use scrypto::prelude::*;

use bonding_macros::blueprint_stub; // fancy proc macro that generates a "hidden" empty blueprint so the stubs can be used

// the expected inferface, but for the moment "impl Trait...." cannot be done inside a blueprint
// use blueprint_stub to generate an empty blueprint so we can get the ABI stubs for use from other blueprints
#[blueprint_stub]
pub trait BondingCurve {
    fn get_mint_amount(
        &self,
        collateral_amount: Decimal,
        reserve_amount: Decimal,
        supply_amount: Decimal,
    ) -> Decimal;
    fn get_return_amount(
        &self,
        continuous_amount: Decimal,
        reserve_amount: Decimal,
        supply_amount: Decimal,
    ) -> Decimal;
    fn get_initial_supply(&self, collateral_amount: Decimal) -> Decimal;
    fn get_price(&self, reserve_amount: Decimal, supply_amount: Decimal) -> Decimal;
}
