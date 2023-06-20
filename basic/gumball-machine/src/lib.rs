use scrypto::prelude::*;

#[blueprint]
mod gumball_machine {
    struct GumballMachine {
        gumballs: Vault,
        collected_xrd: Vault,
        collected_accepted_token: Vault,
        admin_badge: ResourceAddress,
        price: Decimal,
    }

    impl GumballMachine {
        // given a price in XRD, creates a ready-to-use gumball machine
        pub fn instantiate_gumball_machine(
            accepted_token: Bucket,
            price: Decimal,
        ) -> (ComponentAddress, Bucket) {
            // creating admin badge, need this for withdraw XRD
            let admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "admin badge")
                .metadata("description", "Admin Badge for withdrawal of XRD")
                .mint_initial_supply(1);

            // create a new Gumball resource, with a fixed quantity of 100
            let bucket_of_gumballs = ResourceBuilder::new_fungible()
                .metadata("name", "Gumball")
                .metadata("symbol", "GUM")
                .metadata("description", "A delicious gumball")
                .mint_initial_supply(100);

            // Setting up the access rules of the component
            let rules = AccessRulesConfig::new()
                // The third parameter here specifies the authority allowed
                // to update the rule.
                .method(
                    "withdraw_xrd",
                    rule!(require(admin_badge.resource_address())),
                    LOCKED,
                )
                // The second parameter here specifies the authority allowed
                // to update the rule.
                .default(rule!(allow_all), LOCKED);

            // populate a GumballMachine struct and instantiate a new component
            let component = Self {
                gumballs: Vault::with_bucket(bucket_of_gumballs),
                collected_xrd: Vault::new(RADIX_TOKEN),
                collected_accepted_token: Vault::with_bucket(accepted_token),
                admin_badge: admin_badge.resource_address(),
                price: price,
            }
            .instantiate();

            let component_address = component.globalize_with_access_rules(rules);

            // Return the instantiated component and the admin badge we just
            // minted
            (component_address, admin_badge)
        }

        pub fn get_price(&self) -> Decimal {
            self.price
        }

        pub fn buy_gumball(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {
            assert!(
                self.collected_xrd.resource_address() == payment.resource_address()
                    || self.collected_accepted_token.resource_address()
                        == payment.resource_address(),
                "Wrong token type passed in"
            );
            // take our price in XRD out of the payment
            // if the caller has sent too few, or sent something other than XRD, they'll get a runtime error
            let our_share = payment.take(self.price);

            if self.collected_accepted_token.resource_address() == payment.resource_address() {
                self.collected_accepted_token.put(our_share)
            } else {
                self.collected_xrd.put(our_share)
            };

            // we could have simplified the above into a single line, like so:
            // self.collected_xrd.put(payment.take(self.price));

            // return a tuple containing a gumball, plus whatever change is left on the input payment (if any)
            // if we're out of gumballs to give, we'll see a runtime error when we try to grab one
            (self.gumballs.take(1), payment)
        }

        pub fn withdraw_xrd(&mut self, token: Bucket) -> Bucket {
            let withdraw_amount =
                if self.collected_accepted_token.resource_address() == token.resource_address() {
                    self.collected_accepted_token.take(token.amount())
                } else if self.collected_xrd.resource_address() == token.resource_address() {
                    self.collected_xrd.take(token.amount())
                } else {
                    panic!("Withdrawing Invalid Token")
                };

            withdraw_amount
        }
    }
}
