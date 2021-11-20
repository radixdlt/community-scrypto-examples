use scrypto::prelude::*;

blueprint! {
    struct CandyShop {
        // Define what resources and data will be the Candy shop
        gumball_vault: Vault,
        lollipop_vault: Vault,
        jawbreaker_vault: Vault
    }

    impl CandyShop {
        // Implement the functions and methods which will manage those resources and data
        
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn new() -> Component {
            // Create some tokens
            let gumball_bucket: Bucket = ResourceBuilder::new()
                .metadata("name", "Gumball")
                .metadata("symbol", "GUM")
                .new_token_fixed(1000);
            let lolli_bucket: Bucket = ResourceBuilder::new()
                .metadata("name", "Lollipop")
                .metadata("symbol", "LOL")
                .new_token_fixed(1000);
            let jaw_bucket: Bucket = ResourceBuilder::new()
                .metadata("name", "Jawbreaker")
                .metadata("symbol", "JAW")
                .new_token_fixed(1000);
            // Instantiate CandyShop component, populating its vault with our supply of 1000 HelloToken
            Self {
                gumball_vault: Vault::with_bucket(gumball_bucket),
                lollipop_vault: Vault::with_bucket(lolli_bucket),
                jawbreaker_vault: Vault::with_bucket(jaw_bucket),
            }
            .instantiate()
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn free_candy(&mut self) -> Bucket {
            info!("My balance is: {} Gumball Token. Now giving away a gumball!", self.gumball_vault.amount());
            // If the semi-colon is omitted on the last line, the last value seen is automatically returned
            // In this case, a bucket containing 1 HelloToken is returned
            self.gumball_vault.take(1)
        }
    }
}
