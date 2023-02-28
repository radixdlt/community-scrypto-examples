use scrypto::prelude::*;

blueprint! {
    struct PoolManager {
        lp_mint_badge: Vault,
        lp_resource_address: ResourceAddress,
        lp_per_asset_ratio:Decimal,
    }

    impl PoolManager {
        /// Creates a PoolManager component and returns the component address
        pub fn new(
            lp_symbol: String,
            lp_name: String,
        ) -> (ComponentAddress,Bucket) {

            // Mint the badge needed to mint LP Tokens
            let lp_mint_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "LP Token Mint Auth")
                .initial_supply(1);

            // Mint the badge needed to mint LP Tokens
            let admin_badge = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .metadata("name", "PoolManager admin bage")
            .initial_supply(1);

            //  Define our LP token and mint an initial supply
            let lp_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("symbol", lp_symbol)
                .metadata("name", lp_name)
                .mintable(rule!(require(lp_mint_badge.resource_address())), LOCKED)
                .burnable(rule!(require(lp_mint_badge.resource_address())), LOCKED)
                .no_initial_supply();


            let mut liquidity_pool = Self {
                lp_mint_badge: Vault::with_bucket(lp_mint_badge),
                lp_resource_address,
                lp_per_asset_ratio: Decimal::one(),
            }
            .instantiate();


            let access_rules = AccessRules::new()
            .default(rule!(require(admin_badge.resource_address())), AccessRule::DenyAll);

             liquidity_pool.add_access_check(access_rules);


            // Return the new LiquidityPool component, as well as the initial supply of LP tokens
            (liquidity_pool.globalize(),admin_badge)
        }

        /// Adds liquidity to this pool and return the LP tokens representing pool shares
        /// along with any remainder.
        pub fn add_liquidity(&mut self, tokens: Decimal, pool_amount: Decimal ) -> Bucket {

            self.ajust_lp_per_asset_ratio(pool_amount);


            // Get the resource manager of the lp tokens
            let lp_resource_manager = borrow_resource_manager!(self.lp_resource_address);

            // Mint LP tokens according to the share the provider is contributing
            let  supply_to_mint = tokens * self.lp_per_asset_ratio;

            let lp_tokens = self.lp_mint_badge.authorize(|| {
               return lp_resource_manager.mint(supply_to_mint);
            });


            // self.ajust_lp_per_asset_ratio1(pool_amount);


            // Return the LP tokens along with any remainder
            lp_tokens
        }


        /// Removes liquidity from this pool.
        pub fn remove_liquidity(&mut self,
             lp_tokens: Bucket, pool_amount: Decimal
        ) -> Decimal {

            self.ajust_lp_per_asset_ratio(pool_amount);

            assert!(
                self.lp_resource_address == lp_tokens.resource_address(),
                "Wrong token type passed in"
            );

            // self.ajust_lp_per_asset_ratio1(pool_amount);

            // Withdraw the correct amounts of tokens from the pool
            let  to_remove = lp_tokens.amount()/(self.lp_per_asset_ratio);

            // Remain residual liquidity will be withdraw on the last withdrawal to deal with rounding issue
            let withdrawn =  std::cmp::min(to_remove, pool_amount);

            // Burn the LP tokens received
            self.lp_mint_badge.authorize(|| {
                lp_tokens.burn();
            });

            // Rest lp_per_asset_ratio if the pool is empty
            if pool_amount ==  Decimal::zero() {
                self.lp_per_asset_ratio = Decimal::one()
            }



            // Return the withdrawn tokens
            withdrawn
        }

        // Collect fee for liquidity provider. will be added to the pool and LP per asset ratio will be ajusted to reflectthe new pool size (Liquidity + Fee)
       pub fn ajust_lp_per_asset_ratio(&mut self,
            pool_amount: Decimal
        )  {



            let lp_resource_manager = borrow_resource_manager!(self.lp_resource_address);

            let m = lp_resource_manager.total_supply();

            if m !=  Decimal::zero() {
                self.lp_per_asset_ratio = m / pool_amount;
            } else {
                self.lp_per_asset_ratio = Decimal::one()
            }
        }

                // // / Collect fee for liquidity provider. will be added to the pool and LP/Token will be ajusted accordingly
                // pub fn add_collected_fee(&mut self,
                //     tokens: Decimal,pool_amount:Decimal
                // )  {

                //     // let diff = tokens - self.fee_colledted;

                //     // if diff > dec!(0) {

                //         let lp_resource_manager = borrow_resource_manager!(self.lp_resource_address);
                //         if lp_resource_manager.total_supply() !=  Decimal::zero() {
                //             self.lp_per_asset_ratio = lp_resource_manager.total_supply() / pool_amount;
                //         } else {
                //             self.lp_per_asset_ratio = Decimal::one()
                //         }

                //         self.fee_colledted = tokens;
                //     // }

                // }


    }
}
