use scrypto::prelude::*;

blueprint! {
    struct LiquidityPool {
        pool: Vault,
        lp_mint_badge: Vault,
        lp_resource_address: ResourceAddress,
        lp_per_asset_ratio:Decimal,
    }

    impl LiquidityPool {
        /// Creates a LiquidityPool component and returns the component address
        /// along with the initial LP tokens.
        pub fn new(
            tokens: Bucket,
            lp_symbol: String,
            lp_name: String,
        ) -> (ComponentAddress,Bucket) {

            // Check arguments
            assert!(
                !tokens.is_empty(),
                "You must pass in an initial supply of token"
            );
            assert_ne!(
                borrow_resource_manager!(tokens.resource_address()).resource_type(),
                ResourceType::NonFungible,
                "A liquidity pool can only have fungible tokens."
            );

            // Mint the badge needed to mint LP Tokens
            let lp_mint_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "LP Token Mint Auth")
                .initial_supply(1);

            //  Define our LP token and mint an initial supply
            let lp_tokens = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("symbol", lp_symbol)
                .metadata("name", lp_name)
                .mintable(rule!(require(lp_mint_badge.resource_address())), LOCKED)
                .burnable(rule!(require(lp_mint_badge.resource_address())), LOCKED)
                .initial_supply(tokens.amount());
            let lp_resource_address = lp_tokens.resource_address();
            
            let liquidity_pool = Self {
                pool: Vault::with_bucket(tokens),

                lp_mint_badge: Vault::with_bucket(lp_mint_badge),
                lp_resource_address,

                lp_per_asset_ratio: Decimal::one(),
            }
            .instantiate()
            .globalize();

            // Return the new LiquidityPool component, as well as the initial supply of LP tokens
            (liquidity_pool,lp_tokens)
        }

        /// Adds liquidity to this pool and return the LP tokens representing pool shares
        /// along with any remainder.
        pub fn add_liquidity(&mut self,
            tokens: Bucket
        ) -> Bucket {

            // Get the resource manager of the lp tokens
            let lp_resource_manager = borrow_resource_manager!(self.lp_resource_address);

            // Mint LP tokens according to the share the provider is contributing 
            let  supply_to_mint = tokens.amount() * self.lp_per_asset_ratio;

            let lp_tokens = self.lp_mint_badge.authorize(|| {
               return lp_resource_manager.mint(supply_to_mint);
            });

            // Add tokens to the liquidity pool
            self.pool.put(tokens);

            // Return the LP tokens along with any remainder
            lp_tokens
        }

        /// Collect fee for liquidity provider. will be added to the pool and LP/Token will be ajusted accordingly
        pub fn add_collected_fee(&mut self, 
            tokens: Bucket
        )  {

            self.pool.put(tokens);

            let lp_resource_manager = borrow_resource_manager!(self.lp_resource_address);
            if lp_resource_manager.total_supply() !=  Decimal::zero() {
                self.lp_per_asset_ratio = lp_resource_manager.total_supply() / self.pool.amount();
            } else {
                self.lp_per_asset_ratio = Decimal::one()
            }
        }

        /// Removes liquidity from this pool.
        pub fn remove_liquidity(&mut self,
             lp_tokens: Bucket
        ) -> Bucket {
            assert!(
                self.lp_resource_address == lp_tokens.resource_address(),
                "Wrong token type passed in"
            );

            // Withdraw the correct amounts of tokens A and B from reserves
            let  to_remove = lp_tokens.amount()/(self.lp_per_asset_ratio);

            // Remain residual liquidity will be withdrawl on the last withdrawal  
            let withdrawn = self.pool.take(std::cmp::min(to_remove, self.pool.amount()));
           
            // Burn the LP tokens received
            self.lp_mint_badge.authorize(|| {
                lp_tokens.burn();
            });

            // Rest lp_per_asset_ratio if the pool is empty
            if self.pool.amount() ==  Decimal::zero() {
                self.lp_per_asset_ratio = Decimal::one()
            }

            // Return the withdrawn tokens
            withdrawn
        }
    }
}
