use scrypto::prelude::*;

#[derive(NftData)]
pub struct AirdropWithWithdrawData {
    amount: Decimal,
    token_type : Address, 
    #[scrypto(mutable)]
    is_collected : bool
}

blueprint! {

    
    struct AirdropWithWithdraw {
        admin_badge: Address,
        tokens : Vault,
        recipient_address_by_badge_id : HashMap<u128, Address>,
        recipient_badge_def : ResourceDef,
        minter_badge_vault : Vault
    }

    impl AirdropWithWithdraw {
        
        pub fn new(token_type : Address) -> (Component, Bucket) {

            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .initial_supply_fungible(1);

            let minter_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                               .metadata("name", "minter badge")
                                               .initial_supply_fungible(1); 

            let recipient_badge_def = ResourceBuilder::new_non_fungible()
                                .metadata("name", "recipient badge")
                                .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
                                .badge(minter_badge.resource_address(), MAY_MINT | MAY_CHANGE_INDIVIDUAL_METADATA)
                                .no_initial_supply(); 

            
            
            let component = Self {
                admin_badge : admin_badge.resource_address(),
                tokens : Vault::new(token_type),
                recipient_address_by_badge_id :  HashMap::new(),
                recipient_badge_def ,
                minter_badge_vault : Vault::with_bucket(minter_badge)
            }
            .instantiate();

            return (component, admin_badge)
        }


        #[auth(admin_badge)]
        pub fn add_recipient(&mut self, recipient: Address, tokens : Bucket)  {

            assert!(tokens.amount() > Decimal::zero(), "tokens quantity cannot be 0"); 
            let recipient_badge_id = Uuid::generate();
            let recipient_badge = self.minter_badge_vault.authorize(|auth|
                return self.recipient_badge_def.mint_nft(recipient_badge_id , 
                                                         AirdropWithWithdrawData {  
                                                                                    amount : tokens.amount() , 
                                                                                    token_type : tokens.resource_address(),
                                                                                    is_collected : false
                                                                                }, 
                                                         auth)
            );
         self.recipient_address_by_badge_id.insert(recipient_badge_id, recipient);
           self.tokens.put(tokens);
           Account::from(recipient).deposit(recipient_badge);
        }
        

        #[auth(recipient_badge_def)]
        pub fn withdraw_token(&mut self) -> Bucket
        {  
            let recipient_badge_id  = auth.get_nft_id();  
            assert!(self.recipient_address_by_badge_id.contains_key(&recipient_badge_id), "Recipient not found");
            let mut nft_data : AirdropWithWithdrawData = self.recipient_badge_def.get_nft_data(recipient_badge_id);
            nft_data.is_collected = true; 
            let amount = nft_data.amount; 
            self.minter_badge_vault.authorize({
                |auth| self.recipient_badge_def.update_nft_data(recipient_badge_id, nft_data, auth)
            });
            return self.tokens.take(amount);
        }
    }
}
