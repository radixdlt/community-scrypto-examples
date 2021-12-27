use scrypto::prelude::*;


blueprint! {

    struct AirdropWithFixedTokensByRecipient {
        recipients: Vec<Address>,
        tokens_by_recipient : Decimal
    }

    impl AirdropWithFixedTokensByRecipient {
        
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn new(tokens_by_recipient : Decimal) -> Component {
            Self {
                recipients :  Vec::new(),
                tokens_by_recipient : tokens_by_recipient
            }
            .instantiate()
        }

        pub fn add_recipient(&mut self, recipient: Address) {
                self.recipients.push(recipient);    
        }

        pub fn perform_airdrop(&mut self, tokens : Bucket) -> Bucket  {
            
            let recipients_count = self.recipients.len();
            println!("perform_airdrop : recipients_count {0}", recipients_count);
            assert!(recipients_count > 0, "You must register at least one recipient before performing an airdrop");

            let required_tokens = self.tokens_by_recipient * Decimal::from(recipients_count);
            assert!(tokens.amount() >=required_tokens, "The tokens quantity is not sufficiant"); 

            for i in 0..recipients_count {
                let address = self.recipients.get(i).unwrap();
                Account::from(*address).deposit(tokens.take(self.tokens_by_recipient));
            }
            
            
            if !tokens.is_empty() {
                let additionnal_tokens_by_recipient = tokens.amount()/recipients_count;
                for i in 0..recipients_count {
                    let address = self.recipients.get(i).unwrap();
                    Account::from(*address).deposit(tokens.take(additionnal_tokens_by_recipient));
                }
            }

            return tokens;
        }
    }
}
