use num_traits::{Signed, Zero};
use scrypto::prelude::Decimal;

#[cfg(not(feature = "use_rationals"))]
mod details {
    pub type Number = num_bigint::BigInt;

    #[inline(always)]
    pub fn bigint_to_number(b: num_bigint::BigInt, precision_bits: u16) -> Number {
        b << precision_bits
    }

    #[inline(always)]
    pub fn bigint_from_number(b: Number, precision_bits: u16) -> num_bigint::BigInt {
        // let b = b >> precision_bits; // truncate isn't great, but it's easy
        // round instead (could probably be optimized)

        if precision_bits > 0 {
            let mut b = b >> (precision_bits - 1);
            if (b.clone() % 2) == 1.into() {
                b += 1;
            }
            b >> 1
        } else {
            b
        }
    }

    #[inline(always)]
    pub fn pow_nd(base: &Number, n: u32, d: u32) -> Number {
        base.pow(n).nth_root(d)
    }
}

#[cfg(feature = "use_rationals")]
mod details {
    use num_rational::BigRational;
    pub type Number = num_rational::BigRational;

    #[inline(always)]
    pub fn bigint_to_number(b: num_bigint::BigInt, _precision_bits: u16) -> Number {
        BigRational::new(b, 1_000_000_000_000_000_000u128.into())
    }

    #[inline(always)]
    pub fn bigint_from_number(b: Number, _precision_bits: u16) -> num_bigint::BigInt {
        let multiple = BigRational::new(1_000_000_000_000_000_000u128.into(), 1.into());
        // well, to_integer() docs say " /// Converts to an integer, rounding towards zero."
        // but really want half-round away from zero
        // (b * multiple).to_integer()
        // so use .round() instead
        (b * multiple).round().numer().clone()
    }

    pub fn pow_nd(base: &Number, n: u32, d: u32) -> Number {
        let r = base.pow(n as i32);
        // nth_root will round so we still need to explicitly add floating point precision here
        // let's use 1e-10 ** x for both numerator and denominator
        // experimentally checked
        let multiple: num_bigint::BigInt = 1_000_000_000_000_000_000u128.into();
        let multiple = multiple.pow(d + 2); // extra bits 60 * (d + 2) enough precision
        let numer = r.numer() * &multiple;
        let denom = r.denom() * multiple;
        let numer_root = numer.nth_root(d);
        let denom_root = denom.nth_root(d);
        BigRational::new(numer_root, denom_root)
    }
}

pub use details::Number;

pub fn decimal_from_number(b: Number, precision_bits: u16) -> Option<Decimal> {
    // convert from BigInt with Decimal precision
    let b = details::bigint_from_number(b, precision_bits);

    // faster, but assumes Decimal implementation details which may not always be pub
    // let i: i128 = b.to_i128()?;
    // return Some(Decimal(i));

    // slower, uses the canonical 'try_from' API instead
    let mut bytes = b.to_signed_bytes_le();
    let non_padded_len = bytes.len();
    let padding = (16 - non_padded_len % 16) % 16;
    if padding > 0 {
        let pad_byte = if b.is_negative() { 0xFF } else { 0x00 };
        bytes.resize(non_padded_len + padding, pad_byte);
    }
    Decimal::try_from(&bytes[..16]).ok()
}

pub fn number_from_decimal(d: Decimal, precision_bits: u16) -> Number {
    // faster, but assumes Decimal implementation details which may not always be pub
    //let b: BigInt = d.0.into();

    // slower, uses the canonical 'to_vec' API instead
    let b: num_bigint::BigInt = num_bigint::BigInt::from_signed_bytes_le(&d.to_vec());

    details::bigint_to_number(b, precision_bits)
}

pub fn scaled_power(
    scale: &Number,
    base_n: &Number,
    base_d: &Number,
    exp_n: u32,
    exp_d: u32,
) -> Number {
    let n = details::pow_nd(base_n, exp_n, exp_d);
    let d = details::pow_nd(base_d, exp_n, exp_d);

    assert!(d != Number::zero(), "scaled_power divide by zero"); // nicer error message
    scale * n / d
}

#[cfg(test)]
mod test {
    use super::details::bigint_from_number;
    use super::details::bigint_to_number;
    use super::*;
    use num_traits::ToPrimitive;
    use scrypto::prelude::*;

    #[test]
    fn test_decimal_from_bigint() {
        let precision_bits = 10;
        let i: i128 = -25;
        let b: num_bigint::BigInt = i.into();
        let b = bigint_to_number(b, precision_bits);
        let maybe_d = decimal_from_number(b, precision_bits);
        assert_eq!(maybe_d, Decimal::from_str("-0.000000000000000025").ok())
    }

    #[test]
    fn test_bigint_from_decimal() {
        let precision_bits = 0;
        let i: i128 = -25;
        let d = Decimal::from_str("-0.000000000000000025").ok().unwrap();
        let b = number_from_decimal(d, precision_bits);
        let b = bigint_from_number(b, precision_bits);
        assert_eq!(b.to_i128().unwrap(), i);

        let precision_bits = 2;
        let i: i128 = -25;
        let d = Decimal::from_str("-0.000000000000000025").ok().unwrap();
        let b = number_from_decimal(d, precision_bits);
        let b = bigint_from_number(b, precision_bits);
        assert_eq!(b.to_i128().unwrap(), i);
    }
}
