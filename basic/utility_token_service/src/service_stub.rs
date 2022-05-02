use scrypto::prelude::*;

/*
This blueprint is an example of wrapping another component - UtilityTokenFactory.

No actual service is provided but two stub services (regular and premium) are demoed.
*/

use crate::util_token_fac::UtilityTokenFactory;

blueprint! {
    struct ServiceStub {
        utf: UtilityTokenFactory,
        used_tokens: Vault,
        simple_service_count: u32,
        premium_service_count: u32
    }

    impl ServiceStub {

        // Create a UtilityTokenFactory using the simulator and pass it into this constructor.
        // See the README.md file for an example of how to do this.
        //
        pub fn new(comp: ComponentAddress) -> ComponentAddress {
            let my_utf: UtilityTokenFactory = UtilityTokenFactory::from(comp);
            let my_utf_address = my_utf.address();
            Self {  
                utf: my_utf,
                used_tokens: Vault::new(my_utf_address),
                simple_service_count: 0,
                premium_service_count: 0
            }
            .instantiate()
            .globalize()
        }

        // Send the collected UT coins out to be burned.
        // Not required to do this often but letting the vault get too big is a mild safety concern.
        //
        fn maybe_redeem(&mut self) {
            if self.used_tokens.amount() > 100.into() {
                self.utf.redeem( self.used_tokens.take_all())
            };
        }

        // Show how many services have been delivered.
        //
        pub fn show(&self) { // Ideally this function would be secured by a badge here.
            info!("Simple Services performed: {}", self.simple_service_count);
            info!("Premium Services performed: {}", self.premium_service_count);
            // Ideally we could safely call show() on the UTF too.
        }

        // Now we define our services
        pub fn simple_service(&mut self, mut payment: Bucket) -> Bucket {
            assert!(payment.resource_address() == self.utf.address(), "Simple service requires 1 util token");
            assert!(payment.amount() >= Decimal::one(), "Simple service requires 1 util token");
            assert!(self.utf.address() == self.used_tokens.resource_address(), "Mismatch in Vault setup.");
            self.used_tokens.put(payment.take(1));
            info!("Performing Simple Service now.");
            self.simple_service_count += 1;
            self.maybe_redeem();
            payment // return the user's change (if any)
        }

        pub fn premium_service(&mut self, mut payment: Bucket) -> Bucket {
            assert!(payment.resource_address() == self.utf.address(), "Premium service requires 3 util tokens");
            assert!(payment.amount() >= 3.into(), "Premium service requires 3 util tokens");
            self.used_tokens.put(payment.take(3));
            info!("Performing Premium Service now.");
            self.premium_service_count += 1;
            self.maybe_redeem();
            payment // return the user's change (if any)
        }

        
        // The original constructur shown below used these hard-coded values.
        // const MAX_BUY: u32 = 100;    // Limit users to purchasing 100 tokens at a time.
        // const UTIL_TOKEN_NAME: &str = "My Service";
        // const UTIL_TOKEN_SYMBOL: &str = "STUB";
        // const UTIL_TOKEN_DESCRIPTION: &str = "Do nothing service";

        /* This was the original constructor that was used to bootstrap this bluerint.
        // It worked fine but it is not particularly flexible and was deemd to be inferior.
        // Left here for the sake of posterity.
        // 
        pub fn new() -> Component {
            // The new func takes: name, symbol, description, price (in XRD), mint_size, max_buy
            let my_utf: UtilityTokenFactory = UtilityTokenFactory::new(UTIL_TOKEN_NAME.to_string(), UTIL_TOKEN_SYMBOL.to_string(), UTIL_TOKEN_DESCRIPTION.to_string(), 5, 500, MAX_BUY).into();
            let my_utf_address = my_utf.address();
            Self {  
                utf: my_utf,
                used_tokens: Vault::new(ResourceDef::from(my_utf_address)),
                simple_service_count: 0,
                premium_service_count: 0
            }
            .instantiate()
        }
        */
    }
}
