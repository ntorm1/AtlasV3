use atlas_core::error::AtlasError;
use serde::{Deserialize, Serialize};
use std::fs;
use toml;

pub static WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
pub static WETH_BALANCE_SLOT: i32 = 3;
pub static WETH_DECIMALS: u8 = 18;
pub static FB_COINBASE: &str = "0xDAFEA492D9c6733ae3d56b7Ed1ADB60692c98Bc5"; // Flashbots Builder

