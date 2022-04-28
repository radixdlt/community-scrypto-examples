use scrypto::prelude::*;

blueprint! {
    struct Airdrop {
        admin_badge: ResourceAddress,
        recipients: Vec<ComponentAddress>,
    }

    impl Airdrop {
        pub fn new() -> (ComponentAddress, Bucket) {
            let admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);

            let access_rules = AccessRules::new()
                .method("add_recipient", rule!(require(admin_badge.resource_address())))
                .method("perform_airdrop", rule!(require(admin_badge.resource_address())))
                .default(rule!(allow_all));
                
            let component = Self {
                admin_badge: admin_badge.resource_address(),
                recipients: Vec::new()
            }
            .instantiate()
            .add_access_check(access_rules)
            .globalize();

            (component, admin_badge)
        }

        pub fn add_recipient(&mut self, recipient: ComponentAddress) {
            self.recipients.push(recipient);    
        }

        pub fn perform_airdrop(&self, mut tokens: Bucket) -> Bucket {
            let num_recipients = self.recipients.len();
            assert!(num_recipients > 0, "You must register at least one recipient before performing an airdrop");

            // Calculate the amount of tokens each recipient can receive
            let amount_per_recipient = tokens.amount() / Decimal::from(num_recipients as i128);

            // Send that amount to all but the last recipients
            for recipients_address in self.recipients.iter() {
                borrow_component!(*recipients_address).call::<()>("deposit", args![tokens.take(amount_per_recipient)])
            }
            
            // Return a bucket of the remaining tokens back to the caller
            return tokens;
        }
    }
}
