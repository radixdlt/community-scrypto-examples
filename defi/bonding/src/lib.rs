//! scrypto-bonding: A Scrypto package for creating and using bonding curves as an automated market maker on Radix
//!
//! The main blueprint to interact with is `BondingAMM`.  It has buy/sell functionality against a single
//! bonding curve defined by a plugable second component.  It will instantiate a component from the `RatioBondingCurve`
//! blueprint if no other curve component is provided.
//!
//! There is also an extrememly simple `BasicBondingCurve` blueprint (more like a flat line) which could be used to implement
//! a simple "wrapped" or "virtual" token from another.  Or it's a a good template for making your own curve component.
//!
//! Some useful utilities are included too.  An internal crate is used to provide the `blueprint_stub` macro which
//! automates creating stub functions from a trait so calling another component is ergonomic.  Also included and
//! used for the `RatioBondingCurve` is a reusable arbitrary precision number implmentation that converts to/from Decimal
//! It is precise but not yet optimized.  Bounded (BigInt) or unbounded (BigRational) precision is configurable with a feature flag.
//!
//! # Bonuses:
//!
//! * This package is well tested (though not completely) with both unit and integration tests.  Integration
//! testing is not as easy as it should be, but using the [`scrypto-unit`] library helps a whole lot
//!
//! * Key portions of `BondingAMM` are implemented using [`scrypto_statictypes`] which provides compile-time resource
//! checking and additional runtime checking for enhanced producitivity and safety.
//!
//! [`scrypto-unit`]: https://github.com/plymth/scrypto-unit
//! [`scrypto_statictypes`]: https://github.com/devmannic/scrypto_statictypes
//!
//! # References:
//!
//! If you want to read more about bonding curves here some things that might be useful:
//!
//! Reading List
//! * https://blog.relevant.community/bonding-curves-in-depth-intuition-parametrization-d3905a681e0a?gi=8df5128ab916
//! * https://blog.relevant.community/how-to-make-bonding-curves-for-continuous-token-models-3784653f8b17
//! * https://billyrennekamp.medium.com/converting-between-bancor-and-bonding-curve-price-formulas-9c11309062f5
//! * https://yos.io/2017/11/10/bonding-curves/
//! * https://medium.com/linum-labs/intro-to-bonding-curves-and-shapes-bf326bc4e11a
//! * https://discourse.sourcecred.io/t/bonding-curve-references/271
//!
//! Code
//! * https://github.com/oed/bonding-curves/blob/master/contracts/PolynomialCurvedToken.sol
//! * https://github.com/OpenZeppelin/openzeppelin-contracts/blob/7b7ff729b82ea73ea168e495d9c94cb901ae95ce/contracts/math/Power.sol
//! * https://github.com/bancorprotocol/contracts-solidity/blob/c9adc95e82fdfb3a0ada102514beb8ae00147f5d/solidity/contracts/converter/BancorFormula.sol
//!
//! # Notes:
//!
//! The initial release is pre-Alexandria with expected (minimal) API incompatabilities
//!
mod basic_curve; // a simple flat "curve" 1:1 implementation as a reference
mod bonding_curve;
mod default_curve; // a complete non-production bonding curve implementation parametrizable by "curve weight" aka "reserve ratio".  Max precision within Decimal. (ie. precise, but unoptimized)
mod number; // arbitrary precision math used in default_curve // the trait for cross-blueprint calls for plugable curve math

use scrypto::prelude::*;
use scrypto_statictypes::prelude::*; // Use https://github.com/devmannic/scrypto_statictypes

declare_resource!(RESERVE); // resource type used for the reserve pool, collateral for buys
declare_resource!(CONTINUOUS); // resource type minted/burned by the bonding curve
declare_resource!(AUTH); // resource type for authority (badges) for mint/burn and authentication

blueprint! {
    struct BondingAMM {
        reserve: VaultOf<RESERVE>,
        continuous: VaultOf<CONTINUOUS>,
        continuous_auth: VaultOf<AUTH>,
        bonding_curve: ComponentAddress, // plugable, does the math
    }

    impl BondingAMM {
        // Convenient Constructor with default curve - also makes it easy to call from integratino tests -- passing the bonding_curve = None is not possible: FailedToBuildArgs(UnsupportedType(3, Option { value: Custom { name: "scrypto::core::Component", generics: [] } }))
        pub fn new_default(
            initial_reserve: BucketOf<RESERVE>,
            continuous_name: String,
            continuous_symbol: String,
        ) -> (ComponentAddress, BucketOf<CONTINUOUS>) {
            BondingAMM::new(initial_reserve, continuous_name, continuous_symbol, None)
        }

        // Convenient Constructor with specified curve - also makes it easy to call from integratino tests -- passing the bonding_curve = None is not possible: FailedToBuildArgs(UnsupportedType(3, Option { value: Custom { name: "scrypto::core::Component", generics: [] } }))
        pub fn new_with_curve(
            initial_reserve: BucketOf<RESERVE>,
            continuous_name: String,
            continuous_symbol: String,
            bonding_curve: ComponentAddress,
        ) -> (ComponentAddress, BucketOf<CONTINUOUS>) {
            BondingAMM::new(
                initial_reserve,
                continuous_name,
                continuous_symbol,
                Some(bonding_curve.into()),
            )
        }

        // Main constructor easily called from other blueprints
        pub fn new(
            initial_reserve: BucketOf<RESERVE>,
            continuous_name: String,
            continuous_symbol: String,
            bonding_curve: Option<ComponentAddress>,
        ) -> (ComponentAddress, BucketOf<CONTINUOUS>) {
            // initial_reserve cannot be empty
            assert!(!initial_reserve.is_empty());

            // get the curve Component for the math, or create a default_curve with ratio 1:5 and 384 bit precision (which is plenty for the 1e-18 precision of a Decimal)
            // the 384 bit precision is not needed and ignored when built with feature=use_rationals
            let bonding_curve =
                bonding_curve.unwrap_or_else(|| default_curve::RatioBondingCurve::new(1, 5, 384));

            // use the generated stubs for calling methods on the Component (kind of like a virtual call aka dynamic dispatch, but it happens via the kernel)
            let curve: crate::bonding_curve::BondingCurve = bonding_curve.clone().into();

            // calculate the initial_supply for the initial_reserve
            let initial_supply = curve.get_initial_supply(initial_reserve.amount());

            // setup auth/badges
            let continuous_auth: BucketOf<AUTH> = ResourceBuilder::new_fungible().divisibility(DIVISIBILITY_NONE)
                .initial_supply(1)
                .into(); // only this Component can mint/burn CONTINUOUS.  No failsafe for better or worse

            // setup continuous resource
            let continuous_def: ResourceOf<CONTINUOUS> =
                ResourceBuilder::new_fungible()
                    .metadata("name", continuous_name)
                    .metadata("symbol", continuous_symbol)
                    .mintable(rule!(require(continuous_auth.resource_address())), LOCKED)
                    .burnable(rule!(require(continuous_auth.resource_address())), LOCKED)
                    .no_initial_supply()
                    .into();

            // mint the initial supply
            let mut continuous = continuous_auth
                .authorize(|| continuous_def.mint(initial_supply));

            // store and instantiate
            let component = Self {
                reserve: VaultOf::with_bucket(initial_reserve),
                continuous: VaultOf::with_bucket(continuous.take(0)),
                continuous_auth: VaultOf::with_bucket(continuous_auth),
                bonding_curve,
            }
            .instantiate()
            .globalize();

            (component, continuous)
        }

        pub fn buy(
            &mut self,
            collateral: BucketOf<RESERVE>,
            minimum_to_receive: Decimal,
        ) -> (BucketOf<CONTINUOUS>, BucketOf<RESERVE>) {
            // mint the right amount of CONTINOUS and return it
            assert!(!minimum_to_receive.is_negative());

            debug!("buy  with RESERVE    amount: {}", collateral.amount());

            if collateral.is_empty() {
                // fast path, don't panic to allow better composability within a single transaction
                debug!("returning CONTINOUS amount: {}", 0);
                debug!("returning RESERVE amount: {}", collateral.amount());
                return (self.continuous.take(0), collateral);
            }

            // calculate the amount to mint
            let mint_amount = self.get_buy_quote_amount(collateral.amount());

            debug!("will mint CONTINOUS amount: {}", mint_amount);

            if mint_amount.is_zero() || mint_amount < minimum_to_receive {
                // not enough for even a little CONTINUOUS
                // return empty bucket of CONTINUOUS and the sent in RESERVE instead of failing the tx incase other instructions want to do something else with the RESERVE
                debug!("returning CONTINOUS amount: {}", mint_amount);
                debug!("returning RESERVE amount: {}", collateral.amount());
                return (self.continuous.take(0), collateral);
            }

            // keep the collateral
            self.reserve.put(collateral);

            // mint for return
            let continuous = self.continuous_auth.authorize(|| {
                self.continuous
                    .resource_manager()
                    .mint(mint_amount)
            });

            debug!("returning CONTINOUS amount: {}", continuous.amount());
            debug!("returning RESERVE amount: {}", 0);

            // return the minted CONTINUOUS and empty RESERVE bucket
            (continuous, self.reserve.take(0))
        }

        pub fn sell(
            &mut self,
            continuous: BucketOf<CONTINUOUS>,
            minimum_to_receive: Decimal,
        ) -> (BucketOf<RESERVE>, BucketOf<CONTINUOUS>) {
            // burn the CONTINUOUS and return the right amount of RESERVE
            assert!(!minimum_to_receive.is_negative());

            debug!("sell with CONTINUOUS amount: {}", continuous.amount());

            if continuous.is_empty() {
                // fast path, don't panic to allow better composability within a single transaction
                debug!("returning RESERVE    amount: {}", 0);
                debug!("returning CONTINUOUS amount: {}", continuous.amount());
                return (self.reserve.take(0), continuous);
            }
            // calculate the amount to return
            let return_amount = self.get_sell_quote_amount(continuous.amount());

            debug!("will return RESERVE amount: {}", return_amount);

            if return_amount.is_zero() || return_amount < minimum_to_receive {
                // not enough for even a little RESERVE
                // return empty bucket of RESERVE and the sent in CONTINUOUS instead of failing the tx incase other instructions want to do something else with the RESERVE
                debug!("returning RESERVE amount: {}", 0);
                debug!("returning CONTINOUS amount: {}", continuous.amount());
                return (self.reserve.take(0), continuous);
            }

            // burn the CONTINUOUS
            self.continuous_auth
                .authorize(|| continuous.burn());

            // return from reserve vault, and empty CONTINUOUS bucket
            (self.reserve.take(return_amount), self.continuous.take(0))
        }

        pub fn get_price(&self) -> Decimal {
            // use the generated stubs for calling methods on the Component (kind of like a virtual call aka dynamic dispatch, but it happens via the kernel)
            let curve: crate::bonding_curve::BondingCurve = self.bonding_curve.clone().into();
            curve.get_price(
                self.reserve.amount(),
                self.continuous.resource_manager().total_supply(),
            )
        }

        pub fn get_sell_quote(&mut self, continuous_amount: Decimal) -> ProofOf<RESERVE> {
            // This variant returns a Proof.  Only possible with sell because
            // for buying, we can't return "proof" because the amount isn't minted yet

            // use the generated stubs for calling methods on the Component (kind of like a virtual call aka dynamic dispatch, but it happens via the kernel)
            let curve: crate::bonding_curve::BondingCurve = self.bonding_curve.clone().into();
            // calculate the amount that would be returned
            let return_amount = curve.get_return_amount(
                continuous_amount,
                self.reserve.amount(),
                self.continuous.resource_manager().total_supply(),
            );
            // return a placeholder for the amount as proof we have it in reserve
            let bucket = self.reserve.take(return_amount);
            bucket.create_proof() // return proof of amount, but don't give it away
        }

        pub fn get_buy_quote_amount(&self, collateral_amount: Decimal) -> Decimal {
            // use the generated stubs for calling methods on the Component (kind of like a virtual call aka dynamic dispatch, but it happens via the kernel)
            let curve: crate::bonding_curve::BondingCurve = self.bonding_curve.clone().into();
            // calculate amount that would be minted
            curve.get_mint_amount(
                collateral_amount,
                self.reserve.amount(),
                self.continuous.resource_manager().total_supply(),
            )
        }

        pub fn get_sell_quote_amount(&self, continuous_amount: Decimal) -> Decimal {
            // interestingly, trying to reuse self.get_sell_quote to get a BucketRef and then look at the amount leads to failure with dangling buckets no matter what I've tried
            // instead just calculate the amount directly
            // use the generated stubs for calling methods on the Component (kind of like a virtual call aka dynamic dispatch, but it happens via the kernel)
            let curve: crate::bonding_curve::BondingCurve = self.bonding_curve.clone().into();
            // calculate the amount that would be returned
            curve.get_return_amount(
                continuous_amount,
                self.reserve.amount(),
                self.continuous.resource_manager().total_supply(),
            )
        }
    }
}
