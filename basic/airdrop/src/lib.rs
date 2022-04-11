use scrypto::prelude::*;

blueprint! {
    struct Airdrop {
        admin_badge: ResourceDef,
        recipients: Vec<Address>,
    }

    impl Airdrop {
        pub fn new() -> (Component, Bucket) {
            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .initial_supply_fungible(1);

            let component = Self {
                admin_badge: admin_badge.resource_def(),
                recipients: Vec::new()
            }
            .instantiate();

            (component, admin_badge)
        }

        #[auth(admin_badge)]
        pub fn add_recipient(&mut self, recipient: Address) {
            self.recipients.push(recipient);    
        }

        #[auth(admin_badge)]
        pub fn perform_airdrop(&self, mut tokens: Bucket) {
            let num_recipients = self.recipients.len();
            assert!(num_recipients > 0, "You must register at least one recipient before performing an airdrop");

            // Calculate the amount of tokens each recipient can receive
            let amount_per_recipient = tokens.amount() / Decimal::from(num_recipients as i128);

            // Send that amount to all but the last recipients
            for i in 0..(num_recipients - 1) {
                let address = self.recipients.get(i).unwrap();
                let bucket_to_send = tokens.take(amount_per_recipient);
                Component::from(*address).call::<()>("deposit", vec![scrypto_encode(&bucket_to_send)]);
            }

            // Send the remaining tokens to the last recipient.
            // This should be almost exactly the calculated amount except for rounding errors. This way we can be sure
            // not to loose any tokens.
            let last_address = self.recipients.get(num_recipients-1).unwrap();

            Component::from(*last_address).call::<()>("deposit", vec![scrypto_encode(&tokens)]);
        }
    }
}
