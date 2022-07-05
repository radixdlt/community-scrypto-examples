use scrypto::prelude::*;

use num_traits::Zero;

use crate::number::*;

fn calculate_price(r: Number, s: Number, reserve_ratio_n: u32, reserve_ratio_d: u32) -> Number {
    //  Continuous Token Price = Reserve Token Balance / (Continuous Token Supply x Reserve Ratio)
    let n = number_from_decimal(Decimal::try_from(reserve_ratio_n).unwrap(), 0);
    let d = number_from_decimal(Decimal::try_from(reserve_ratio_d).unwrap(), 0);
    r / s / n * d
}

fn calculate_initial_supply(
    collateral_amount: Number,
    reserve_ratio_n: u32,
    reserve_ratio_d: u32,
) -> Number {
    //collateral_amount * reserve_ratio_d / reserve_ratio_n
    scaled_power(
        &collateral_amount,
        &number_from_decimal(Decimal::try_from(reserve_ratio_d).unwrap(), 0),
        &number_from_decimal(Decimal::try_from(reserve_ratio_n).unwrap(), 0),
        1,
        1,
    )
}

fn calculate_curve_mint(
    c: Number,
    r: Number,
    s: Number,
    reserve_ratio_n: u32,
    reserve_ratio_d: u32,
) -> Number {
    // PurchaseReturn = ContinuousTokenSupply * ((1 + ReserveTokensReceived / ReserveTokenBalance) ^ (ReserveRatio) - 1)
    // = s * (((1 + c / r) ^ rr) - 1)

    //let result = s * (n / d - 1); // worse for precision
    //let result = s.clone() * n / d - s; // better for precision
    // scaled_power used here for arbitrary precision way to do: s * (1 + c/r)^rr
    let result = scaled_power(&s, &(c + &r), &r, reserve_ratio_n, reserve_ratio_d) - s;

    result
}

fn calculate_curve_return(
    c: Number,
    r: Number,
    s: Number,
    reserve_ratio_n: u32,
    reserve_ratio_d: u32,
) -> Number {
    // SaleReturn = ReserveTokenBalance * (1 - (1 - ContinuousTokensReceived / ContinuousTokenSupply) ^ (1 / (ReserveRatio)))
    // = r * (1 - (1 - c / s) ^ (1/rr))
    // = r * (1 - ([s - c] / s)^rri) = r * (1-(n/d))

    // let result = r * (1 - n / d); // bad for precision
    // let result = r - r*(n/d); // better for precision
    // let result = r - ((r * n) / d); // best precision
    // scaled_power used here for arbitrary precision way to do: r * (1 - c/s)^(1/rr)
    let result = &r - scaled_power(&r, &(&s - c), &s, reserve_ratio_d, reserve_ratio_n);

    result
}

fn get_initial_supply(
    collateral_amount: Decimal,
    reserve_ratio_n: u32,
    reserve_ratio_d: u32,
    precision_bits: u16,
) -> Decimal {
    assert!(!collateral_amount.is_negative());
    assert!(reserve_ratio_d != 0);

    if collateral_amount.is_zero() {
        return Decimal::zero();
    }

    let collateral_amount = number_from_decimal(collateral_amount, precision_bits);

    let result = calculate_initial_supply(collateral_amount, reserve_ratio_n, reserve_ratio_d);

    decimal_from_number(result, precision_bits).unwrap()
}

fn get_mint_amount(
    collateral_amount: Decimal,
    reserve_amount: Decimal,
    supply_amount: Decimal,
    reserve_ratio_n: u32,
    reserve_ratio_d: u32,
    precision_bits: u16,
) -> Decimal {
    assert!(!collateral_amount.is_negative());
    assert!(!reserve_amount.is_negative());
    assert!(!supply_amount.is_negative());
    assert!(reserve_ratio_d != 0);

    if collateral_amount.is_zero() {
        return Decimal::zero();
    }

    let c = number_from_decimal(collateral_amount, precision_bits);
    let r = number_from_decimal(reserve_amount, precision_bits);
    let s = number_from_decimal(supply_amount, precision_bits);

    let result = calculate_curve_mint(c, r, s, reserve_ratio_n, reserve_ratio_d);

    assert!(result >= Number::zero(), "Calculated negative mint amount");

    decimal_from_number(result, precision_bits).unwrap()
}

fn get_return_amount(
    continuous_amount: Decimal,
    reserve_amount: Decimal,
    supply_amount: Decimal,
    reserve_ratio_n: u32,
    reserve_ratio_d: u32,
    precision_bits: u16,
) -> Decimal {
    assert!(!continuous_amount.is_negative());
    assert!(!reserve_amount.is_negative());
    assert!(!supply_amount.is_negative());
    assert!(reserve_ratio_d != 0);

    if continuous_amount.is_zero() {
        return Decimal::zero();
    }

    let c = number_from_decimal(continuous_amount, precision_bits);
    let r = number_from_decimal(reserve_amount, precision_bits);
    let s = number_from_decimal(supply_amount, precision_bits);

    let result = calculate_curve_return(c, r, s, reserve_ratio_n, reserve_ratio_d);

    assert!(
        result >= Number::zero(),
        "Calculated negative return amount"
    );

    decimal_from_number(result, precision_bits).unwrap()
}

blueprint! {
    struct RatioBondingCurve {
        reserve_ratio_n: u32,
        reserve_ratio_d: u32,
        precision_bits: u16,
    }

    impl RatioBondingCurve {
        pub fn new(reserve_ratio_n: u32, reserve_ratio_d: u32, precision_bits: u16) -> ComponentAddress {
            debug!(
                "RatioBondingCurve::new called with {} / {} @ {} bits",
                reserve_ratio_n, reserve_ratio_d, precision_bits
            );
            assert_ne!(
                reserve_ratio_d, 0,
                "reserve radio denominator cannot be zero"
            );
            // simplify and verify simple case of rr = 0 or 1
            let reserve_ratio_d = if reserve_ratio_n == 0 {
                1
            } else {
                reserve_ratio_d
            };
            Self {
                reserve_ratio_n,
                reserve_ratio_d,
                precision_bits,
            }
            .instantiate()
            .globalize()
        }

        // can't do trait impl's within the blueprint.  Would be ncie to get those compile time checks...
        // impl BondingCurve for blueprint::RatioBondingCurve {

        pub fn get_initial_supply(&self, collateral_amount: Decimal) -> Decimal {
            get_initial_supply(
                collateral_amount,
                self.reserve_ratio_n,
                self.reserve_ratio_d,
                self.precision_bits,
            )
        }

        pub fn get_mint_amount(
            &self,
            collateral_amount: Decimal,
            reserve_amount: Decimal,
            supply_amount: Decimal,
        ) -> Decimal {
            debug!("RatioBondingCurve::get_mint_amount called with:\ncollateral_amount: {}\nreserve_amount: {}\nsupply_amount: {}", 
                collateral_amount, reserve_amount, supply_amount);
            get_mint_amount(
                collateral_amount,
                reserve_amount,
                supply_amount,
                self.reserve_ratio_n,
                self.reserve_ratio_d,
                self.precision_bits,
            )
        }

        pub fn get_return_amount(
            &self,
            continuous_amount: Decimal,
            reserve_amount: Decimal,
            supply_amount: Decimal,
        ) -> Decimal {
            debug!("RatioBondingCurve::get_return_amount called with:\ncontinuous_amount: {}\nreserve_amount: {}\nsupply_amount: {}", 
                continuous_amount, reserve_amount, supply_amount);
            get_return_amount(
                continuous_amount,
                reserve_amount,
                supply_amount,
                self.reserve_ratio_n,
                self.reserve_ratio_d,
                self.precision_bits,
            )
        }

        pub fn get_price(&self, reserve_amount: Decimal, supply_amount: Decimal) -> Decimal {
            let r = number_from_decimal(reserve_amount, self.precision_bits);
            let s = number_from_decimal(supply_amount, self.precision_bits);
            let p = calculate_price(r, s, self.reserve_ratio_n, self.reserve_ratio_d);
            decimal_from_number(p, self.precision_bits).unwrap()
        }

        // }
    }
}

// -------- Testing

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_1_buy() {
        let precision_bits = 384;
        let collateral_amount = 300.into();
        let reserve_amount = 60000.into();
        let supply_amount = 300000.into();
        let reserve_ratio_n = 1;
        let reserve_ratio_d = 5;
        let to_mint = get_mint_amount(
            collateral_amount,
            reserve_amount,
            supply_amount,
            reserve_ratio_n,
            reserve_ratio_d,
            precision_bits,
        );
        let expected: i128 = 299401793723844635041; // 299.401793723844635041 // the right answer to 18 decimal places (the default for Decimal)
        let expected: Decimal = Decimal(expected);
        assert_eq!(to_mint, expected);

        // buy more
        let collateral_amount = 700.into();
        let reserve_amount = 60300.into();
        let supply_amount = Decimal(300000000000000000000000i128 + 299401793723844635041i128);
        let reserve_ratio_n = 1;
        let reserve_ratio_d = 5;
        let to_mint = get_mint_amount(
            collateral_amount,
            reserve_amount,
            supply_amount,
            reserve_ratio_n,
            reserve_ratio_d,
            precision_bits,
        );
        let expected: i128 = 693997438220660073726; // 693.997438220660073726  // the right answer to 18 decimal places (the default for Decimal
        let expected: Decimal = Decimal(expected);
        assert_eq!(to_mint, expected);
    }

    #[test]
    fn test_2_buy_sell() {
        let precision_bits = 384;
        let collateral_amount = 300.into();
        let reserve_amount = 60000.into();
        let supply_amount = 300000.into();
        let reserve_ratio_n = 1;
        let reserve_ratio_d = 5;
        let to_mint = get_mint_amount(
            collateral_amount,
            reserve_amount,
            supply_amount,
            reserve_ratio_n,
            reserve_ratio_d,
            precision_bits,
        );
        let expected: i128 = 299401793723844635041; // 299.401793723844635041 // the right answer to 18 decimal places (the default for Decimal)
        let expected: Decimal = Decimal(expected); // TODO use decimal_from_bigint instead
        assert_eq!(to_mint, expected);

        // sell back same
        let continuous_amount = expected;
        let reserve_amount = 60300.into();
        let supply_amount = Decimal(300000000000000000000000i128 + 299401793723844635041i128);
        let reserve_ratio_n = 1;
        let reserve_ratio_d = 5;
        let to_return = get_return_amount(
            continuous_amount,
            reserve_amount,
            supply_amount,
            reserve_ratio_n,
            reserve_ratio_d,
            precision_bits,
        );
        let expected: i128 = 300000000000000000000; // 300.0  // the right answer to 18 decimal places (the default for Decimal
        let expected: Decimal = Decimal(expected); // TODO use decimal_from_bigint instead
        assert_eq!(to_return, expected);
    }

    #[test]
    fn test_3_sell() {
        // sell some
        let precision_bits = 384;
        let continuous_amount = 100.into();
        let reserve_amount = 61000.into();
        let supply_amount = Decimal(
            300000000000000000000000i128 + 299401793723844635041i128 + 693997438220660073726i128,
        ); // amount after test_1
        let reserve_ratio_n = 1;
        let reserve_ratio_d = 5;
        let to_return = get_return_amount(
            continuous_amount,
            reserve_amount,
            supply_amount,
            reserve_ratio_n,
            reserve_ratio_d,
            precision_bits,
        );
        let expected: i128 = 101263817029251588263; // 101.263817029251588263 // the right answer to 18 decimal places (the default for Decimal
        let expected: Decimal = Decimal(expected); // TODO use decimal_from_bigint instead
        assert_eq!(to_return, expected);
    }
}
