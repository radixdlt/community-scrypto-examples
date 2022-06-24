use scrypto::prelude::*;

#[derive(TypeId, Encode, Decode, Describe)]
pub enum OptionBucket {
    None,
    Some(Bucket)
}

#[derive(NonFungibleData)]
pub struct HelloTK {
}

blueprint! {
    struct Hello {
        // Define what resources and data will be managed by Hello components
        sample_vault: Vault,
        controller: Vault,
        move_badge: ResourceAddress,
        hello: ResourceAddress
    }

    impl Hello {
        // Implement the functions and methods which will manage those resources and data
        
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn new() -> (ComponentAddress, Bucket) {

            let controller = ResourceBuilder::new_fungible()
                .metadata("name", "controller")
                .initial_supply(dec!(1));

            let kyc = ResourceBuilder::new_fungible()
                .metadata("name", "kyc")
                .initial_supply(dec!(1));

            let move_badge = ResourceBuilder::new_fungible()
                .mintable(rule!(require(controller.resource_address())), LOCKED)
                .burnable(rule!(require(controller.resource_address())), LOCKED)
                .metadata("name", "move_badge")
                .no_initial_supply();

            let hello = ResourceBuilder::new_non_fungible()
                .metadata("name", "HelloToken")
                .metadata("symbol", "HT")
                .mintable(rule!(require(controller.resource_address())), LOCKED)
                .burnable(rule!(require(controller.resource_address())), LOCKED)
                .restrict_withdraw(rule!(require(kyc.resource_address()) || require(controller.resource_address())), LOCKED)
                .restrict_deposit(rule!(require(move_badge)), LOCKED)
                .no_initial_supply();

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            (Self {
                sample_vault: Vault::new(hello),
                controller: Vault::with_bucket(controller),
                move_badge: move_badge,
                hello: hello
            }
            .instantiate()
            .globalize(), kyc)
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn free_token(&mut self) -> (OptionBucket, Proof) {

            self.controller.authorize(|| {

                let move_badge = borrow_resource_manager!(self.move_badge)
                    .mint(dec!(1));

                let move_proof = move_badge.create_proof();

                borrow_resource_manager!(self.move_badge)
                    .burn(move_badge);

                let token = borrow_resource_manager!(self.hello)
                    .mint_non_fungible(&NonFungibleId::random(), HelloTK{});

                (OptionBucket::Some(token), move_proof)

            })
            
        }

        pub fn deposit(&mut self, token: OptionBucket) {

            match token {
                OptionBucket::None => {error!("none")}
                OptionBucket::Some(token) => {self.controller.authorize(|| {

                    let move_badge = borrow_resource_manager!(self.move_badge)
                        .mint(dec!(1));
    
                    move_badge.authorize(|| {self.sample_vault.put(token)});
    
                    borrow_resource_manager!(self.move_badge)
                        .burn(move_badge);
    
                })
                }
            }
        }

        pub fn withdraw(&mut self) -> (OptionBucket, Proof) {

            self.controller.authorize(|| {

                let move_badge = borrow_resource_manager!(self.move_badge)
                    .mint(dec!(1));

                let move_proof = move_badge.create_proof();

                borrow_resource_manager!(self.move_badge)
                    .burn(move_badge);

                (OptionBucket::Some(self.sample_vault.take(dec!(1))), move_proof)

            })

        }
    }
}