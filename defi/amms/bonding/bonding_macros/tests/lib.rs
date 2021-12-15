use scrypto::prelude::*;

use scrypto_traits::blueprint_stub;

// the expected inferface, but declaring a trait doesn't actually matter since it really depends on the ABI, just to be helpful
// Probably generating ABI / stubs from trait declaration is what is needed instead of depending on an existing implementation as is done below
#[blueprint_stub]
pub trait BondingCurve {
    fn get_mint_amount(&self, collateral_amount: Decimal, reserve_amount: Decimal, supply_amount: Decimal) -> Decimal;
    fn get_return_amount(&self, continuous_amount: Decimal, reserve_amount: Decimal, supply_amount: Decimal) -> Decimal;
    fn get_initial_supply(&self, collateral_amount: Decimal) -> Decimal;
}

#[blueprint_stub]
pub trait TestTrait {
    fn method1(&self);
    fn method2(&self) -> u32;
}