use atlas_core::error::AtlasError;
use serde::{Deserialize, Serialize};
use std::fs;

//==========================================================================
#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    https_url: String,
    wss_url: String,
}

//==========================================================================
#[derive(Deserialize, Debug)]
pub struct BotConfig {
    address: String,
    private_key: String,
    identity_key: String,
}

//==========================================================================
#[derive(Deserialize, Debug)]
pub struct TelegramConfig {
    token: String,
    chat_id: String,
}

//==========================================================================
#[derive(Deserialize, Debug)]
pub struct Settings {
    use_alert: bool,
    debug: bool,
}

//==========================================================================
#[derive(Deserialize, Debug)]
pub struct Environment {
    rust_backtrace: u32,
}

//==========================================================================
#[derive(Deserialize, Debug)]
pub struct AtlasEnv {
    server: ServerConfig,
    bot: BotConfig,
    telegram: TelegramConfig,
    settings: Settings,
    environment: Environment,
}

//==========================================================================
impl AtlasEnv {
    //==========================================================================
    pub fn new(config_path: &str) -> Result<Self, AtlasError> {
        Ok(toml::from_str(&(fs::read_to_string(config_path)?))?)
    }
}

//==========================================================================
#[cfg(test)]
mod tests {
    use atlas_core::util::AtlasUtil;

    use super::*;
    use crate::test::DEV_CONFIG;

    //==========================================================================
    #[test]
    fn test_env() {
        let env = AtlasEnv::new(DEV_CONFIG);
        if !env.is_ok() {
            assert!(false, "{}", format!("{:?}", env.err()));
        }
    }

    //==========================================================================
    #[test]
    fn test_logger() {
        AtlasUtil::setup_logger().unwrap();
    }
}
