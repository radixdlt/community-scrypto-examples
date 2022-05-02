use scrypto::prelude::*;

blueprint! {
    struct Bank {
        bank_account: Vault,
        cash: Vault,
        owner_badge: ResourceAddress,
    }

    impl Bank {
        pub fn instantiate_bank() -> (ComponentAddress, Bucket) {
            let mut initial_cash: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("description", "USD")
                .initial_supply(1000);

            let owner_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "owner authorization")
                .initial_supply(1);

            // Add an access check for the withdraw
            let access_check = AccessRules::new()
                .method("bank_withdraw", rule!(require(owner_badge.resource_address())))
                .default(rule!(allow_all));

            // Fill the bank account and the pocket
            let instantiate_accounts = Self {
                bank_account: Vault::with_bucket(initial_cash.take(dec!(500))),
                cash: Vault::with_bucket(initial_cash),
                owner_badge: owner_badge.resource_address(),
            }
            .instantiate()
            .add_access_check(access_check)
            .globalize();

            (instantiate_accounts, owner_badge)
        }

        pub fn balances(&self) {
            info!("My bank balance is: {}", self.bank_account.amount());
            info!("My cash balance is: {}", self.cash.amount());
        }

        pub fn bank_deposit(&mut self, exchange: Decimal) {
            assert!(exchange < self.cash.amount(),
            "You don't have that much of cash. Your current balance is {}", self.cash.amount());
            self.bank_account.put(self.cash.take(exchange));
            info!("Your Cash balance is now {}", self.cash.amount());
            info!("Your bank balance is now {}", self.bank_account.amount());
        
        }
        
        pub fn bank_withdraw(&mut self, exchange: Decimal) {
            assert!(exchange < self.cash.amount(),
            "You don't have that much of cash. Your current balance is {}", self.cash.amount());
            self.cash.put(self.bank_account.take(exchange));
            info!("Your Cash balance is now {}", self.cash.amount());
            info!("Your bank balance is now {}", self.bank_account.amount());
        
        }
    }
}
