use scrypto::prelude::*;


blueprint! {

    struct AirdropWithFixedTokensByRecipient {
        admin_badge: ResourceDef,
        recipients: Vec<Address>,
        tokens_by_recipient : Decimal
    }

    impl AirdropWithFixedTokensByRecipient {
        
        pub fn new(tokens_by_recipient : Decimal) -> (Component, Bucket) {
            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .initial_supply_fungible(1);
            let component = Self {
                admin_badge : admin_badge.resource_def(),
                recipients :  Vec::new(),
                tokens_by_recipient : tokens_by_recipient
            }
            .instantiate();

            return (component, admin_badge)
        }

        #[auth(admin_badge)]
        pub fn add_recipient(&mut self, recipient: Address) {
                self.recipients.push(recipient);    
        }

        #[auth(admin_badge)]
        pub fn perform_airdrop(&mut self, tokens : Bucket) -> Bucket  {
            
            let recipients_count = self.recipients.len();
            println!("perform_airdrop : recipients_count {0}", recipients_count);
            assert!(recipients_count > 0, "You must register at least one recipient before performing an airdrop");

            let required_tokens = self.tokens_by_recipient * Decimal::from(recipients_count);
            assert!(tokens.amount() >=required_tokens, "The tokens quantity is not sufficient"); 

            for i in 0..recipients_count {
                let address = self.recipients.get(i).unwrap();
                Account::from(*address).deposit(tokens.take(self.tokens_by_recipient));
            }
            
            // additionnal tokens ? 
            if !tokens.is_empty() {
                let additionnal_tokens_by_recipient = tokens.amount()/recipients_count;
                for i in 0..recipients_count {
                    //Is there a minimum token quantity to make a deposit ??
                    // if so this part of code will have to be adapted additionnal_tokens_by_recipient must be less
                    // than the minimum token quantity
                    let address = self.recipients.get(i).unwrap();
                    Account::from(*address).deposit(tokens.take(additionnal_tokens_by_recipient));
                }
            }

            return tokens;
        }
    }
}
