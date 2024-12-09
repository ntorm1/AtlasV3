use atlas_core::error::{AtlasError, AtlasResult};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use log::info;
use reqwest::Client;
use reqwest::Error;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use serde_json::Value;
use std::fmt;
use std::str::FromStr;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};

fn deserialize_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    D: Deserializer<'de>,
    <T as FromStr>::Err: std::fmt::Debug, // for error handling
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<T>()
        .map_err(|e| de::Error::custom(format!("Failed to parse: {:?}", e)))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Price {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub conf: i32,
    pub expo: i32,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub price: i64,
    pub publish_time: i64,
}

impl Price {
    pub fn new(conf: String, expo: i32, price: String, publish_time: i64) -> AtlasResult<Self> {
        let conf_res = conf.parse::<i32>();
        if conf_res.is_err() {
            return Err(AtlasError::CustomError("Invalid conf".to_string()));
        }
        let price_res = price.parse::<i64>();
        if price_res.is_err() {
            return Err(AtlasError::CustomError("Invalid price".to_string()));
        }
        Ok(Price {
            conf: conf_res.unwrap(),
            expo,
            price: price_res.unwrap(),
            publish_time,
        })
    }

    pub fn to_dict(&self) -> serde_json::Value {
        serde_json::json!({
            "conf": self.conf,
            "expo": self.expo,
            "price": self.price,
            "publish_time": self.publish_time,
        })
    }

    pub fn from_dict(price_dict: &PriceDict) -> AtlasResult<Self> {
        if price_dict.conf.is_empty() || price_dict.price.is_empty() {
            return Err(AtlasError::CustomError("Empty conf or price".to_string()));
        }
        Ok(Price {
            conf: price_dict.conf.parse().unwrap(),
            expo: price_dict.expo,
            price: price_dict.price.parse().unwrap(),
            publish_time: price_dict.publish_time,
        })
    }

    pub fn from_json_value(value: &Value) -> AtlasResult<Self> {
        let price_dict: PriceDict = serde_json::from_value(value.clone())?;
        Price::from_dict(&price_dict)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Price(conf={}, expo={}, price={}, publish_time={})",
            self.conf, self.expo, self.price, self.publish_time
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PriceDict {
    pub conf: String,
    pub expo: i32,
    pub price: String,
    pub publish_time: i64,
}

impl PriceDict {
    pub fn to_price(&self) -> AtlasResult<Price> {
        Price::from_dict(self)
    }
}

#[derive(Debug, Deserialize)]
struct PriceFeed {
    id: String,
    price: Price,
    ema_price: Price,
}

const HERMES_ENDPOINT_HTTPS: &str = "https://hermes.pyth.network";
const HERMES_ENDPOINT_WSS: &str = "wss://hermes.pyth.network/ws";

pub struct HermesClient {
    feed_ids: Vec<String>,
    pending_feed_ids: Vec<String>,
    prices_dict: DashMap<String, PriceFeed>,
    endpoint: String,
    ws_endpoint: String,
    feed_batch_size: usize,
}

impl HermesClient {
    pub fn new(
        feed_ids: Vec<String>,
        endpoint: Option<String>,
        ws_endpoint: Option<String>,
        feed_batch_size: Option<usize>,
    ) -> Self {
        HermesClient {
            feed_ids: feed_ids.clone(),
            pending_feed_ids: feed_ids,
            prices_dict: DashMap::new(),
            endpoint: endpoint.unwrap_or_else(|| HERMES_ENDPOINT_HTTPS.to_string()),
            ws_endpoint: ws_endpoint.unwrap_or_else(|| HERMES_ENDPOINT_WSS.to_string()),
            feed_batch_size: feed_batch_size.unwrap_or(100), // default to 100
        }
    }

    fn extract_price_feed_v1(data: &serde_json::Value) -> AtlasResult<PriceFeed> {
        let price = Price::from_json_value(&data["price"])?;
        let ema_price = Price::from_json_value(&data["ema_price"])?;
        let update_data = vec![data["vaa"].clone()]; // Convert to Vec<serde_json::Value>

        Ok(PriceFeed {
            id: data["id"].as_str().unwrap_or_default().to_string(),
            price,
            ema_price,
        })
    }

    fn extract_price_feed_v2(data: &serde_json::Value) -> AtlasResult<Vec<PriceFeed>> {
        let update_data = data["binary"]["data"].clone();
        let mut price_feeds = Vec::new();

        for feed in data["parsed"].as_array().unwrap_or(&vec![]) {
            let price = Price::from_json_value(&feed["price"])?;
            let ema_price = Price::from_json_value(&feed["ema_price"])?;

            let price_feed = PriceFeed {
                id: feed["id"].as_str().unwrap_or_default().to_string(),
                price,
                ema_price,
            };

            price_feeds.push(price_feed);
        }
        Ok(price_feeds)
    }

    pub async fn get_pyth_prices_latest(
        &self,
        feed_ids: Vec<String>,
        version: u8,
    ) -> AtlasResult<Vec<PriceFeed>> {
        let url = match version {
            1 => format!("{}/api/latest_price_feeds", self.endpoint),
            2 => format!("{}/v2/updates/price/latest", self.endpoint),
            _ => {
                return Err(AtlasError::CustomError(format!(
                    "Unsupported version {}",
                    version
                )))
            }
        };

        let params = match version {
            1 => vec![
                ("ids[]", feed_ids.join(",")),
                ("binary", "true".to_string()),
            ],
            2 => vec![
                ("ids[]", feed_ids.join(",")),
                ("encoding", "base64".to_string()),
                ("parsed", "true".to_string()),
            ],
            _ => return Err(AtlasError::CustomError("Unsupported version".to_string())),
        };

        let client = Client::new();
        let response = client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| AtlasError::CustomError(e.to_string()))?;
        let data: Value = response
            .json()
            .await
            .map_err(|e| AtlasError::CustomError(e.to_string()))?;
        if version == 1 {
            let mut results = Vec::new();
            if let Some(arr) = data.as_array() {
                for res in arr {
                    results.push(Self::extract_price_feed_v1(&data)?);
                }
            }
            Ok(results)
        } else if version == 2 {
            Self::extract_price_feed_v2(&data)
        } else {
            Err(AtlasError::CustomError(format!(
                "Unsupported version {}",
                version
            )))
        }
    }

    async fn add_feed_ids(&mut self, feed_ids: Vec<String>) {
        self.pending_feed_ids.extend(feed_ids);
    }

    async fn ws_pyth_prices(&mut self, version: u32) -> Result<(), Box<dyn std::error::Error>> {
        if version != 1 {
            return Err("Unsupported version".into());
        }
        let (mut ws_stream, _) = connect_async(&self.ws_endpoint).await?;
        loop {
            // Add new price feed ids to the ws subscription
            if !self.pending_feed_ids.is_empty() {
                let json_subscribe = json!({
                    "ids": self.pending_feed_ids,
                    "type": "subscribe",
                    "verbose": true,
                    "binary": true,
                });
                ws_stream
                    .send(Message::Text(json_subscribe.to_string()))
                    .await?;
                self.feed_ids.extend(self.pending_feed_ids.clone());
                self.pending_feed_ids.clear();
            }

            // Wait for a message from the websocket
            let msg = ws_stream.next().await.unwrap()?;
            match msg {
                Message::Text(text) => {
                    let msg: Value = serde_json::from_str(&text)?;
                    if msg.get("type") == Some(&Value::String("response".to_string())) {
                        if msg.get("status") != Some(&Value::String("success".to_string())) {
                            return Err("Error subscribing to websocket".into());
                        }
                    }
                    if msg.get("type") == Some(&Value::String("price_update".to_string())) {
                        if let Some(price_feed) = msg.get("price_feed") {
                            if let Some(feed_id) = price_feed.get("id").and_then(|id| id.as_str()) {
                                let new_feed: PriceFeed =
                                    serde_json::from_value(price_feed.clone())?;
                                info!("Received price feed: {:?}", new_feed);
                                self.prices_dict.insert(feed_id.to_string(), new_feed);
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    println!("WebSocket closed");
                    break;
                }
                _ => continue,
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use atlas_core::util::AtlasUtil;

    const ID: &'static str = "0x63f341689d98a12ef60a5cff1d7f85c70a9e17bf1575f0e7c0b2512d48b1c8b3";
    use super::*;

    #[tokio::test]
    async fn test_get_feed_ids() {
        AtlasUtil::setup_logger().unwrap();
        let client = HermesClient::new(vec![], None, None, None);
        let id = "0x63f341689d98a12ef60a5cff1d7f85c70a9e17bf1575f0e7c0b2512d48b1c8b3";
        let feed_ids = client
            .get_pyth_prices_latest(vec![id.to_string()], 2)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_stream_prices() {
        AtlasUtil::setup_logger().unwrap();
        let mut client = HermesClient::new(vec![], None, None, None);
        client.add_feed_ids(vec![ID.to_string()]).await;
        client.ws_pyth_prices(1).await.unwrap();
    }
}
