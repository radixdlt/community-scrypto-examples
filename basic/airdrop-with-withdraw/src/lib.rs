use scrypto::prelude::*;


blueprint! {

    struct AirdropWithWithdraw {
        admin_badge: Address,
        recipients: HashMap<Address, Decimal>,
        tokens : Vault
    }

    impl AirdropWithWithdraw {
        
        pub fn new(token : Address) -> (Component, Bucket) {

            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .initial_supply_fungible(1);


            
            let component = Self {
                admin_badge : admin_badge.resource_address(),
                recipients :  HashMap::new(),
                tokens : Vault::new(token),
            }
            .instantiate();

            return (component, admin_badge)
        }

        fn get_required_tokens_internal(&self) -> Decimal {
            let mut required_tokens  = Decimal::zero();
            for val in self.recipients.values() {
                required_tokens = required_tokens + *val ;  
            }
            return required_tokens;
        }

        #[auth(admin_badge)]
        pub fn get_required_tokens(&self) -> Decimal {
            return self.get_required_tokens_internal();
        }

        #[auth(admin_badge)]
        pub fn add_recipient(&mut self, recipient: Address, tokens : Decimal)  {
            assert!(!self.recipients.contains_key(&recipient) , "Recipient already exist");
            self.recipients.insert(recipient, tokens);    
        }

        #[auth(admin_badge)]
        pub fn put_airdrop_tokens(&mut self, tokens : Bucket)   {

            let recipients_count = self.recipients.len();
            assert!(recipients_count > 0, "You must register at least one recipient before put tokens");

            let required_tokens = self.get_required_tokens_internal();
            assert!(tokens.amount() == required_tokens, "The tokens quantity must be {}", required_tokens); 

            info!("put_airdrop_tokens with recipients_count {} required_tokens {}", recipients_count,  required_tokens);

            self.tokens.put(tokens); 
        }

        pub fn  get_recipient_available_tokens(&mut self, recipient_address : Address) -> Decimal
        {
            assert!(self.recipients.contains_key(&recipient_address), "Recipient not found");
            let recipient_tokens  = self.recipients[&recipient_address]; 
            info!("available_tokens : {}",recipient_tokens);
            return recipient_tokens;
        }

        pub fn withdraw_token(&mut self, recipient_address : Address) -> Bucket
        {   
            assert!(self.tokens.amount() > Decimal::zero(), "Impossible withdrawal"); 
            assert!(self.recipients.contains_key(&recipient_address), "Recipient not found");
            let recipient_tokens  = self.recipients[&recipient_address]; 
            self.recipients.remove(&recipient_address); 
            info!("withdraw_token : {}",recipient_tokens);
            return self.tokens.take(recipient_tokens);
           
        }
    }
}
