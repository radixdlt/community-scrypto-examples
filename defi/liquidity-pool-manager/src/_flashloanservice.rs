// use crate::poolmanager::*;
use scrypto::prelude::*;

// Define the functions on the Radiswap blueprint
external_blueprint! {
    FlashLoanPoolTarget {
        fn instantiate_default(pool_resource_addresse: ResourceAddress, loan_interest:Decimal) -> (ComponentAddress,Bucket);
    }
}

// Define the methods on instantiated components
external_component! {
    FlashLoanPoolComponentTarget {
        fn add_liquidity(&mut self, tokens: Bucket) ;
        fn remove_liquidity(&mut self, tokens: Decimal) -> Bucket;
        fn available_liquidity(&self) -> Decimal;
    }
}

// Define the functions on the Radiswap blueprint
external_blueprint! {
    PoolManagerTarget {
        fn new(lp_symbol: String, lp_name: String) -> (ComponentAddress,Bucket);
    }
}

// Define the methods on instantiated components
external_component! {
    PoolManagerComponentTarget {
        fn add_liquidity(&mut self,tokens: Decimal, pool_amount: Decimal) -> Bucket;
        fn remove_liquidity(&mut self, lp_tokens: Bucket, pool_amount: Decimal  ) -> Decimal;
  }
}

blueprint! {

    struct FlashLoanService {
        flashloanpool: ComponentAddress,
        flashloanpool_admin_badge: Vault,
        poolmanager:ComponentAddress,
        poolmanager_admin_badge: Vault,
    }

    impl FlashLoanService {

        pub fn new(flashloanpool_package_address:PackageAddress,poolmanager_package_address:PackageAddress,pool_resource_addresse: ResourceAddress,lp_symbol: String, lp_name: String) -> ComponentAddress{

            let (flashloanpool,flashloanpool_admin_badge) =   FlashLoanPoolTarget::at(flashloanpool_package_address,"FlashLoanPool").instantiate_default( pool_resource_addresse,dec!("0.01"));

            let (poolmanager,poolmanager_admin_badge) =   PoolManagerTarget::at(poolmanager_package_address,"PoolManager").new(lp_symbol,lp_name);

            Self {
                flashloanpool,
                poolmanager,
                flashloanpool_admin_badge: Vault::with_bucket(flashloanpool_admin_badge),
                poolmanager_admin_badge: Vault::with_bucket(poolmanager_admin_badge),

            }.instantiate().globalize()

        }

        pub fn add_liquidity(&self,tokens:Bucket) -> Bucket{
            ComponentAuthZone::push(self.poolmanager_admin_badge.create_proof());
            ComponentAuthZone::push(self.flashloanpool_admin_badge.create_proof());

            //
            let liquidity_pool_total_amount =  FlashLoanPoolComponentTarget::at(self.flashloanpool).available_liquidity();

            //
            let amount_to_add = tokens.amount();
            FlashLoanPoolComponentTarget::at(self.flashloanpool).add_liquidity(tokens);
            let lp_tokens =  PoolManagerComponentTarget::at(self.poolmanager).add_liquidity(amount_to_add,liquidity_pool_total_amount);

            ComponentAuthZone::pop().drop();
            ComponentAuthZone::pop().drop();

            lp_tokens
        }

        pub fn remove_liquidity(&self,lp_tokens:Bucket) -> Bucket{
            ComponentAuthZone::push(self.poolmanager_admin_badge.create_proof());
            ComponentAuthZone::push(self.flashloanpool_admin_badge.create_proof());

            //
            let liquidity_pool_total_amount =  FlashLoanPoolComponentTarget::at(self.flashloanpool).available_liquidity();

            //
            let amount_to_remove =  PoolManagerComponentTarget::at(self.poolmanager).remove_liquidity(lp_tokens,liquidity_pool_total_amount);
            let tokens = FlashLoanPoolComponentTarget::at(self.flashloanpool).remove_liquidity(amount_to_remove);

            ComponentAuthZone::pop().drop();
            ComponentAuthZone::pop().drop();

            tokens
        }

        pub fn simulate_fee(&self,tokens:Bucket) {

            ComponentAuthZone::push(self.flashloanpool_admin_badge.create_proof());

            FlashLoanPoolComponentTarget::at(self.flashloanpool).add_liquidity(tokens);

            ComponentAuthZone::pop().drop();
        }


    }

}
