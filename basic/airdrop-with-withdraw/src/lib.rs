use scrypto::prelude::*;


blueprint! {

    struct AirdropWithWithdraw {
        admin_badge: Address,
        recipients: HashMap<Address, Vault>,
        user_badge_def : ResourceDef,
        minter_badge_vault : Vault
    }

    impl AirdropWithWithdraw {
        
        pub fn new() -> (Component, Bucket) {

            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .initial_supply_fungible(1);

            let minter_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                               .metadata("name", "minter badge")
                                               .initial_supply_fungible(1); 

            let user_badge_def = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
                                .badge(minter_badge.resource_address(), MAY_MINT | MAY_TRANSFER | MAY_CHANGE_INDIVIDUAL_METADATA)
                                .no_initial_supply(); 
            
            let component = Self {
                admin_badge : admin_badge.resource_address(),
                recipients :  HashMap::new(),
                user_badge_def ,
                minter_badge_vault : Vault::with_bucket(minter_badge)
            }
            .instantiate();

            return (component, admin_badge)
        }


        #[auth(admin_badge)]
        pub fn add_recipient(&mut self, recipient: Address, tokens : Bucket)  {
            assert!(tokens.amount() > Decimal::zero(), "tokens quantity cannot be 0"); 
            assert!(!self.recipients.contains_key(&recipient) , "Recipient already exist");
            self.recipients.insert(recipient, Vault::with_bucket(tokens));
            let user_badge = self.minter_badge_vault.authorize(|auth|
                return self.user_badge_def.mint(1, auth) 
            );
            Account::from(recipient).deposit(user_badge);
        }
        

        #[auth(user_badge_def)]
        pub fn withdraw_token(&mut self) -> Bucket
        {   
            info!("test");
            let recipient_address  = auth.resource_address(); 
            assert!(self.recipients.contains_key(&recipient_address), "Recipient not found");
            let recipient_tokens  = &self.recipients[&recipient_address]; 
            info!("withdraw_token : {}",recipient_tokens.amount());
            return recipient_tokens.take_all();
           
        }
    }
}
