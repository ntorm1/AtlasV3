use alloy_primitives::U256;
use rand::Rng;

//==============================================================================
struct AtlasEthUtil {}

impl AtlasEthUtil {
    //==========================================================================
    // Calculates the next block base fee given the previous block's gas usage / limits
    // Refer to: https://www.blocknative.com/blog/eip-1559-fees
    pub fn calculate_next_block_base_fee(
        gas_used: U256,
        gas_limit: U256,
        base_fee_per_gas: U256,
    ) -> U256 {
        let elasticity_multiplier = U256::from(2);
        let base_fee_change_denominator = U256::from(8);
        let target_gas_used = gas_limit / elasticity_multiplier;
        if gas_used == target_gas_used {
            return base_fee_per_gas;
        }
        let gas_delta = if gas_used > target_gas_used {
            gas_used - target_gas_used
        } else {
            target_gas_used - gas_used
        };
        let base_fee_adjustment =
            base_fee_per_gas * gas_delta / target_gas_used / base_fee_change_denominator;
        if gas_used > target_gas_used {
            base_fee_per_gas + base_fee_adjustment
        } else {
            base_fee_per_gas.saturating_sub(base_fee_adjustment)
        }
    }
}

//==========================================================================
#[test]
fn test_calculate_next_block_base_fee() {
    let gas_used = U256::from(15_000_000);
    let gas_limit = U256::from(30_000_000);
    let base_fee_per_gas = U256::from(100_000_000_000u64);
    let next_base_fee =
        AtlasEthUtil::calculate_next_block_base_fee(gas_used, gas_limit, base_fee_per_gas);
    assert_eq!(next_base_fee, U256::from(100_000_000_000u64));
}
