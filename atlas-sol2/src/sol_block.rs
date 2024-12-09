use atlas_core::{error::AtlasResult, util::AtlasUtil};
use futures::{SinkExt, StreamExt};
use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::{bs58::encode, clock::Slot};
use solana_transaction_status_client_types::EncodedConfirmedBlock;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const CHAINSTAKE_SOLONA_HTTPS: &str =
    "https://solana-mainnet.core.chainstack.com/2fa69914c087050cc7b9887511cd7d35";
const CHAINSTAKE_SOLONA_WS: &str =
    "wss://solana-mainnet.core.chainstack.com/2fa69914c087050cc7b9887511cd7d35";

//==============================================================================
pub struct SolanaRpcWrapper {
    client: RpcClient,
}

//==============================================================================
#[derive(Debug, Deserialize)]
pub struct BlockResponse {
    pub jsonrpc: String,
    pub method: String,
    pub params: BlockParams,
}

//==============================================================================
impl BlockResponse {
    pub fn has_error(&self) -> bool {
        self.params.result.value.err.is_some()
    }

    pub fn get_encoded_block(&self) -> Option<&EncodedConfirmedBlock> {
        self.params.result.value.block.as_ref()
    }
}

//==============================================================================
#[derive(Debug, Deserialize)]
pub struct BlockParams {
    pub result: BlockResult,
    pub subscription: u64,
}

//==============================================================================
#[derive(Debug, Deserialize)]
pub struct BlockResult {
    pub context: BlockContext,
    pub value: BlockWrapper,
}

//==============================================================================
#[derive(Debug, Deserialize)]
pub struct BlockContext {
    pub slot: u64,
}

//==============================================================================
#[derive(Debug, Deserialize)]
pub struct BlockWrapper {
    pub slot: u64,
    pub err: Option<String>,
    pub block: Option<EncodedConfirmedBlock>,
}

//==============================================================================
impl SolanaRpcWrapper {
    //==============================================================================
    pub fn new() -> Self {
        AtlasUtil::setup_logger().unwrap();
        let client = RpcClient::new(CHAINSTAKE_SOLONA_HTTPS.to_string());
        SolanaRpcWrapper { client }
    }

    //==============================================================================
    async fn get_slot(&self) -> Result<Slot, String> {
        match self
            .client
            .get_slot_with_commitment(CommitmentConfig::finalized())
            .await
        {
            Ok(slot) => Ok(slot),
            Err(err) => Err(format!("Error fetching slot: {}", err)),
        }
    }

    //==============================================================================
    async fn stream_block(&self) -> AtlasResult<()> {
        info!("WebSocket connected!");
        let (ws_stream, _) = connect_async(CHAINSTAKE_SOLONA_WS)
            .await
            .expect("Failed to connect");
        let (mut write, mut read) = ws_stream.split();
        let subscription_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "blockSubscribe",
            "params": [
                "all",
                {
                    "commitment": "confirmed",
                    "encoding": "base64",
                    "showRewards": true,
                    "transactionDetails": "full",
                    "maxSupportedTransactionVersion": 0

                }
            ]
        });
        write
            .send(Message::Text(subscription_request.to_string()))
            .await
            .expect("Failed to send subscription request");
        info!("Subscription request sent!");
        while let Some(msg) = read.next().await {
            let now = chrono::Utc::now();
            info!("Received message at {}", now.to_rfc3339());
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        let value: Result<BlockResponse, serde_json::Error> =
                            serde_json::from_value(parsed.clone());
                        if value.is_err() {
                            error!("Error parsing block: {:?}", value.err());
                            continue;
                        }
                        let block = value.unwrap();
                        if block.has_error() {
                            error!("Error fetching block");
                            continue;
                        }
                        let now_end = chrono::Utc::now();
                        let elapsed = now_end - now;
                        let encoded_block = block.get_encoded_block().unwrap();
                        info!(
                            "Received block: {} at {}, Elapsed time: {:?}",
                            encoded_block.blockhash,
                            now.to_rfc3339(),
                            elapsed
                        );
                    }
                }
                Ok(Message::Ping(payload)) => {
                    info!("Received ping: {:?}", payload);
                    if let Err(e) = write.send(Message::Pong(payload)).await {
                        error!("Failed to send Pong: {:?}", e);
                    }
                }
                Ok(_) => {
                    info!("Received non-text message: {:?}", msg);
                }
                Err(e) => {
                    log::error!("Error receiving message: {:?}", e);
                    break;
                }
            }
        }
        info!("WebSocket disconnected!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[tokio::test]
    async fn test_block_stream() {
        let wrapper = SolanaRpcWrapper::new();
        wrapper.stream_block().await.unwrap();
    }

    #[tokio::test]
    async fn test_fetch_token_accounts() {}
}
