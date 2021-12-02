use scrypto::prelude::*;

blueprint! {
    struct Airdrop {
        recipients: Vec<Address>,
    }

    impl Airdrop {
        pub fn new() -> Component {
            Self {
                recipients: Vec::new(),
            }
            .instantiate()
        }

        pub fn add_recipient(&mut self, recipient: Address) {
            self.recipients.push(recipient);    
        }

        pub fn perform_airdrop(&self, tokens: Bucket) {
            let num_recipients = self.recipients.len();
            assert!(num_recipients > 0, "You must register at least one recipient before performing an airdrop");

            // Calculate the amount of tokens each recipient can receive
            let amount_per_recipient = tokens.amount() / Decimal::from(num_recipients as i128);

            // Send that amount to all but the last recipients
            for i in 0..(num_recipients - 1) {
                let address = self.recipients.get(i).unwrap();
                Account::from(*address).deposit(tokens.take(amount_per_recipient));
            }

            // Send the remaining tokens to the last recipient.
            // This should be almost exactly the calculated amount except for rounding errors. This way we can be sure
            // not to loose any tokens.
            let last_address = self.recipients.get(num_recipients-1).unwrap();
            Account::from(*last_address).deposit(tokens);
        }
    }
}
