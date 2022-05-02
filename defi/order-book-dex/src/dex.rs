use crate::trading_pair::*;
use scrypto::prelude::*;

blueprint! {

    /// The Dex component is the central component of this system's architecture.
    /// Admins can use it to register trading pairs and users can use it to discover those trading pairs
    /// and their respective component addresses.
    struct Dex {
        /// A badge used to administer the Dex component
        admin_badge: ResourceAddress,

        /// A map of all trading pairs that are managed by this component.
        /// The keys of the map are tuples where the first value is the address of the base resource
        /// and the second value is the address of the quote resource. The values of the map are
        /// instances of TradingPairInfo. They represent a trading pair.
        trading_pairs: HashMap<(ResourceAddress, ResourceAddress), TradingPairInfo>,
    }

    impl Dex {
        /// Instantiates a new Dex component.
        /// Returns the component and a bucket with the admin badge.
        pub fn instantiate() -> (ComponentAddress, Bucket) {
            let admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Dex admin badge")
                .initial_supply(1);
            let component = Self {
                admin_badge: admin_badge.resource_address(),
                trading_pairs: HashMap::new(),
            }
            .instantiate();

            let access_rules = AccessRules::new()
                .method("add_trading_pair", rule!(require(admin_badge.resource_address())))
                .default(rule!(allow_all));

            (component.add_access_check(access_rules).globalize(), admin_badge)
        }

        /// Adds a trading pair for the given base_resource_address and quote_resource_address.
        /// Checks that the same trading pair cannot be added twice.
        /// Does not check that the reverse of an existing trading pair is not added!
        pub fn add_trading_pair(
            &mut self,
            base_resource_address: ResourceAddress,
            quote_resource_address: ResourceAddress,
        ) {
            // Prevent the same trading pair from being added twice
            assert!(
                !self
                    .trading_pairs
                    .contains_key(&(base_resource_address, quote_resource_address)),
                "Trading pair already exists: {}/{}",
                base_resource_address,
                quote_resource_address
            );

            // Instantiate a new TradingPair component on the ledger
            let trainding_pair_component = TradingPair::instantiate(
                base_resource_address,
                quote_resource_address,
            );

            // Save the info on the newly created trading pair in the trading_pairs HashMap
            self.trading_pairs.insert(
                (base_resource_address, quote_resource_address),
                TradingPairInfo {
                    base_resource_address,
                    quote_resource_address,
                    component_address: trainding_pair_component,
                },
            );
        }

        /// Returns a vector with all trading pairs that are managed by this component
        pub fn get_trading_pairs(&self) -> Vec<TradingPairInfo> {
            self.trading_pairs.values().cloned().collect()
        }

        /// Returns the address for the trading pair with the given base_resource_address and quote_resource_address
        /// or None if no such trading pair exists.
        pub fn get_trading_pair_component_address(
            &self,
            base_resource_address: ResourceAddress,
            quote_resource_address: ResourceAddress,
        ) -> Option<ComponentAddress> {
            self.trading_pairs
                .get(&(base_resource_address, quote_resource_address))
                .map(|trading_pair| trading_pair.component_address)
        }
    }
}

/// Represents a trading pair
#[derive(sbor::TypeId, sbor::Encode, sbor::Decode, sbor::Describe, Clone)]
pub struct TradingPairInfo {
    /// The address of the base resource
    pub base_resource_address: ResourceAddress,

    /// The address of the quote resource
    pub quote_resource_address: ResourceAddress,

    /// The address of TradingPair component that must be used to exchange resources
    pub component_address: ComponentAddress,
}
