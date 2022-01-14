use scrypto::prelude::*;


blueprint! {

    struct AirdropWithWithdraw {
        admin_badge: Address,
        recipients: HashMap<Address, Decimal>,
        tokens : Vault,
        user_badge_def : ResourceDef,
        minter_badge_vault : Vault
    }

    impl AirdropWithWithdraw {
        
        pub fn new(token : Address) -> (Component, Bucket) {

            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .initial_supply_fungible(1);

            let minter_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                               .initial_supply_fungible(1); 

            let user_badge_def = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .flags(MINTABLE)
                                .badge(minter_badge.resource_address(), MAY_MINT | MAY_TRANSFER )
                                .no_initial_supply(); 
            
            let component = Self {
                admin_badge : admin_badge.resource_address(),
                recipients :  HashMap::new(),
                tokens : Vault::new(token),
                user_badge_def : user_badge_def,
                minter_badge_vault : Vault::with_bucket(minter_badge)
            }
            .instantiate();

            return (component, admin_badge)
        }


        #[auth(admin_badge)]
        pub fn add_recipient(&mut self, recipient: Address, tokens : Bucket)  {
            assert!(tokens.amount() > Decimal::zero(), "tokens quantity cannot be 0"); 
            assert!(!self.recipients.contains_key(&recipient) , "Recipient already exist");
            self.recipients.insert(recipient, tokens.amount());
            self.tokens.put(tokens);
            let user_badge = self.minter_badge_vault.authorize(|auth|
                self.user_badge_def.mint(1, auth)
            );
            Account::from(recipient).deposit(user_badge);
        }


        #[auth(user_badge_def)]
        pub fn withdraw_token(&mut self) -> Bucket
        {   
            let recipient_address  = auth.resource_address(); 
            assert!(self.tokens.amount() > Decimal::zero(), "Impossible withdrawal"); 
            assert!(self.recipients.contains_key(&recipient_address), "Recipient not found");
            let recipient_tokens  = self.recipients[&recipient_address]; 
            self.recipients.remove(&recipient_address); 
            info!("withdraw_token : {}",recipient_tokens);
            return self.tokens.take(recipient_tokens);
           
        }
    }
}
