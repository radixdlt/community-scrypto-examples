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
        lp_token: ResourceDef
    }

    impl SingleTokenLiquidityPool {
        pub fn new(initial_funds: Bucket, lp_initial_supply: Decimal) -> Component {
            let funds_resource_def = initial_funds.resource_def();

            // Create badge that will be used to mint and burn LP tokens
            let lp_minter: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .initial_supply_fungible(1);

            // Create the LP token definition
            let lp_token = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "LP Token")
                .metadata("symbol", "LP")
                .flags(MINTABLE | BURNABLE)
                .badge(lp_minter.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            // Mint initial LP tokens
            lp_token.mint(lp_initial_supply, lp_minter.present());

            Self {
                pool: Vault::with_bucket(initial_funds),
                fees: Vault::new(funds_resource_def),
                lp_minter_badge: Vault::with_bucket(lp_minter),
                lp_token: lp_token

            }
            .instantiate()
        }

        // Will be called by other components
        pub fn add_fees(&self, fees: Bucket) {
            self.fees.put(fees);
        }

        // Contribute tokens to the pool in exchange of an LP token
        pub fn add_liquidity(&self, liquidity: Bucket) -> Bucket {
            let pool_share = liquidity.amount() / self.pool.amount();
            self.pool.put(liquidity);

            // Return newly minted lp_tokens
            self.lp_minter_badge.authorize(|badge| {
                self.lp_token
                    .mint(self.lp_token.total_supply() * pool_share, badge)
            })
        }

        // Give LP token back to get portion of the pool and fees
        pub fn remove_liquidity(&self, lp_tokens: Bucket) -> Bucket {
            assert!(lp_tokens.resource_def() == self.lp_token, "Wrong LP token !");

            let share = lp_tokens.amount() / self.lp_token.total_supply();
            
            // Burn the provided LP token
            self.lp_minter_badge.authorize(|badge| {
                lp_tokens.burn_with_auth(badge);
            });

            // Return the share of fees and pool
            let return_bucket = self.fees.take(self.fees.amount() * share);
            return_bucket.put(self.pool.take(self.pool.amount() * share));
            return_bucket
        }
    }
}
