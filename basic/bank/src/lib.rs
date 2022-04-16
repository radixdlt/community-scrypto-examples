use scrypto::prelude::*;

blueprint! {
    struct Bank {
        // Define what resources and data will be managed by Hello components
        bank_account: Vault,
        cash: Vault,
        owner_badge: ResourceDef,

    }

    impl Bank {
        // Implement the functions and methods which will manage those resources and data
        
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn start() -> (Component, Bucket) {
            // Create a new token called "HelloToken," with a fixed supply of 1000, and put that supply into a bucket
            let mut initial_cash: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("description", "USD")
                .initial_supply_fungible(1000);

            let owner_badge: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "owner authorization")
                .initial_supply_fungible(1);


            // Fill the bank account and the pocket
            let instantiate_accounts = Self {
                bank_account: Vault::with_bucket(initial_cash.take(dec!(500))),
                cash: Vault::with_bucket(initial_cash),
                owner_badge: owner_badge.resource_def(),
            }
            .instantiate();

            (instantiate_accounts, owner_badge)
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn balances(&self) {
            info!("My bank balance is: {}", self.bank_account.amount());
            info!("My cash balance is: {}", self.cash.amount());
            // If the semi-colon is omitted on the last line, the last value seen is automatically returned
            // In this case, a bucket containing 1 HelloToken is returned
            // self.sample_vault.take(1)
        }

        pub fn bank_deposit(&mut self, exchange: Decimal) {
            assert!(exchange < self.cash.amount(),
            "You don't have that much of cash. Your current balance is {}", self.cash.amount());
            self.bank_account.put(self.cash.take(exchange));
            info!("Your Cash balance is now {}", self.cash.amount());
            info!("Your bank balance is now {}", self.bank_account.amount());
        
        }
        
        #[auth(owner_badge)]
        pub fn bank_withdraw(&mut self, exchange: Decimal) {
            assert!(exchange < self.cash.amount(),
            "You don't have that much of cash. Your current balance is {}", self.cash.amount());
            self.cash.put(self.bank_account.take(exchange));
            info!("Your Cash balance is now {}", self.cash.amount());
            info!("Your bank balance is now {}", self.bank_account.amount());
        
        }
    }
}
