use scrypto::prelude::*;
use std::collections::HashMap;
use crate::util::*;
use crate::pool::*;

// Main blueprint
blueprint! {
    struct Exchange {
        // Hashmap stores all the PoolComponent instances
        // that are created in the exchange by their 2 tokens' addresses.
        pools: HashMap<(ResourceAddress, ResourceAddress), PoolComponent>,
        // Another hashmap stores the PoolComponent instances by their LP token's resource address.
        pools_lp: HashMap<ResourceAddress, PoolComponent>
    }

    impl Exchange {
        // Instantiate a krulkswap exchange.
        pub fn new_exchange() -> ComponentAddress {
            Self {
                pools: HashMap::new(),
                pools_lp: HashMap::new()
            }
            .instantiate()
            .globalize()
        }

        // Adds a new pool to the exchange by instantiating a new PoolComponent.
        // Returns a Bucket of LP tokens that represents the pool.
        pub fn add_pool(&mut self, mut a: Bucket, mut b: Bucket) -> Bucket {
            // Sort the two buckets, so that the key for a tokenpair will always be the same.
            (a, b) = bucket_sort(a, b);
            // Check if the pool already exists.
            assert!(
                self.pool_exists(a.resource_address(), b.resource_address()),
                "This pool already exists"
            );
            // Create the key for the pool by combining the resource addresses.
            let pool_key = (a.resource_address(), b.resource_address());
            // Instantiate a new PoolComponent.
            let (pool, lp_tokens) = PoolComponent::new_pool(a, b);
            // Insert the PoolComponent into the hashmap with the pool key.
            self.pools.insert(pool_key, pool);
            lp_tokens
        }

        // Add some liquidity to a pool.
        pub fn add_liquidity(&self, mut a: Bucket, mut b: Bucket) -> Bucket {
            // Sort the two buckets, so that the key for a tokenpair will always be the same.
            (a, b) = bucket_sort(a, b);
            // Create the key for the pool by combining the resource addresses.
            let pool_key = (a.resource_address(), b.resource_address());
            // Get the PoolComponent from the hashmap.
            let pool = self.pools.get(&pool_key).unwrap();
            // Pass the buckets off to the PoolComponent's add_liquidity
            // method and return the LP tokens that come back.
            pool.add_liquidity(a, b)
        }

        // Redeem some liquidity from a pool by passing some LP tokens.
        // Returns two buckets; one of token A and one of token B.
        pub fn take_liquidity(&self, lp: Bucket) -> (Bucket, Bucket) {
            // Check if the pool even exists.
            assert!(
                !self.pool_lp_exists(lp.resource_address()),
                "This pool doesn't exist yet"
            );
            // Create the pool key from the resource address of the lp token.
            let pool_key = lp.resource_address();
            // Get the pool from the hashmap using the pool key.
            let pool = self.pools_lp.get(&pool_key).unwrap();
            // Pass the buckets off to the PoolComponent's take_liquidity
            // method and return the tokens that come back.
            pool.take_liquidity(lp)
        }

        // Swap a bucket of token A into a bucket of the target resource.
        // Returns one bucket of the target token.
        pub fn swap(&self, input: Bucket, target_resource: ResourceAddress) -> Bucket {
            // Check if the pool even exists for this token pair.
            assert!(
                !self.pool_exists(input.resource_address(), target_resource),
                "This pool doesn't exist yet"
            );
            // Create the pool key from the resource address of
            // the input bucket and the target resource address.
            let pool_key = (input.resource_address(), target_resource);
            // Get the PoolComponent from the pool key.
            let pool = self.pools.get(&pool_key).unwrap();
            // Pass the buckets off to the PoolComponent's swap
            // method and return the tokens that come back.
            pool.swap(input, target_resource)
        }

        // Check if a pool exists for a token pair.
        pub fn pool_exists(&self, a: ResourceAddress, b: ResourceAddress) -> bool {
            // Sort the two buckets, so that the key for a tokenpair will always be the same.
            let (a, b) = address_sort(a, b);
            // Return the bool value.
            self.pools.contains_key(&(a, b))
        }

        // // Check if a pool exists for an LP ResourceAddress.
        pub fn pool_lp_exists(&self, lp: ResourceAddress) -> bool {
            // Return the bool value.
            self.pools_lp.contains_key(&lp)
        }
    }
}
