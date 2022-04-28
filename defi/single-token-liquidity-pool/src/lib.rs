use scrypto::prelude::*;

/*
 * Liquidity pool with a single token and fees collection mechanism.
 * This component is not meant to be used by itself. 
 * You should use it inside another component.
 */
blueprint! {
    struct SingleTokenLiquidityPool {
        pool: Vault,
        fees: Vault,
        lp_minter_badge: Vault,
        lp_token: ResourceAddress
    }

    impl SingleTokenLiquidityPool {
        pub fn new(initial_funds: Bucket, lp_initial_supply: Decimal) -> ComponentAddress {
            let funds_resource_def = initial_funds.resource_address();

            // Create badge that will be used to mint and burn LP tokens
            let lp_minter: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);

            // Create the LP token definition
            let mut lp_token = ResourceBuilder::new_fungible()
                .metadata("name", "LP Token")
                .metadata("symbol", "LP")
                .mintable(rule!(require(lp_minter.resource_address())), LOCKED)
                .burnable(rule!(require(lp_minter.resource_address())), LOCKED)
                .no_initial_supply();

            // Mint initial LP tokens
            lp_minter.authorize(|| {
                borrow_resource_manager!(lp_token).mint(lp_initial_supply);
            });

            Self {
                pool: Vault::with_bucket(initial_funds),
                fees: Vault::new(funds_resource_def),
                lp_minter_badge: Vault::with_bucket(lp_minter),
                lp_token: lp_token

            }
            .instantiate().globalize()
        }

        // Will be called by other components
        pub fn add_fees(&mut self, fees: Bucket) {
            self.fees.put(fees);
        }

        // Contribute tokens to the pool in exchange of an LP token
        pub fn add_liquidity(&mut self, liquidity: Bucket) -> Bucket {
            let pool_share = liquidity.amount() / self.pool.amount();
            self.pool.put(liquidity);

            // Return newly minted lp_tokens
            self.lp_minter_badge.authorize(|| {
                borrow_resource_manager!(self.lp_token)
                    .mint(borrow_resource_manager!(self.lp_token).total_supply() * pool_share)
            })
        }

        // Give LP token back to get portion of the pool and fees
        pub fn remove_liquidity(&mut self, lp_tokens: Bucket) -> Bucket {
            assert!(lp_tokens.resource_address() == self.lp_token, "Wrong LP token !");

            let share = lp_tokens.amount() / borrow_resource_manager!(self.lp_token).total_supply();
            
            // Burn the provided LP token
            self.lp_minter_badge.authorize(|| {
                lp_tokens.burn();
            });

            // Return the share of fees and pool
            let mut return_bucket = self.fees.take(self.fees.amount() * share);
            return_bucket.put(self.pool.take(self.pool.amount() * share));
            return_bucket
        }
    }
}
