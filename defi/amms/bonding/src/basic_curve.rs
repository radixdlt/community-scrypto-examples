use scrypto::prelude::*;

blueprint! {
    struct BasicBondingCurve {}

    impl BasicBondingCurve {
        pub fn new() -> ComponentAddress {
            BasicBondingCurve {}.instantiate().globalize()
        }

        // can't do trait impl's within the blueprint.  Would be ncie to get those compile time checks...
        //trait BondingCurve {
        pub fn get_mint_amount(
            &self,
            collateral_amount: Decimal,
            _reserve_amount: Decimal,
            _supply_amount: Decimal,
        ) -> Decimal {
            collateral_amount
        }
        pub fn get_return_amount(
            &self,
            continuous_amount: Decimal,
            _reserve_amount: Decimal,
            _supply_amount: Decimal,
        ) -> Decimal {
            continuous_amount
        }
        pub fn get_initial_supply(&self, collateral_amount: Decimal) -> Decimal {
            collateral_amount
        }
        pub fn get_price(&self, _reserve_amount: Decimal, _supply_amount: Decimal) -> Decimal {
            1.into()
        }
        pub fn get_sale_quote(
            &self,
            continuous_amount: Decimal,
            _reserve_amount: Decimal,
            _supply_amount: Decimal,
        ) -> Decimal {
            continuous_amount
        }
        pub fn get_buy_quote(
            &self,
            collateral_amount: Decimal,
            _reserve_amount: Decimal,
            _supply_amount: Decimal,
        ) -> Decimal {
            collateral_amount
        }
        //}
    }
}
