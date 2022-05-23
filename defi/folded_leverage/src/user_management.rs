use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct User {
    #[scrypto(mutable)]
    deposit_balance: HashMap<ResourceAddress, Decimal>,
    #[scrypto(mutable)]
    borrow_balance: HashMap<ResourceAddress, Decimal>,
}


blueprint! {
    struct UserManagement {
        // Vault that holds the authorization badge
        user_badge_vault: Vault,
        // Collects User Address
        user_address: ResourceAddress,
        // A counter for ID generation
        user_id: u64,
        //

    }

    impl UserManagement {
        pub fn new() -> ComponentAddress {
            // Badge that will be stored in the component's vault to update user state.
            let lending_protocol_user_badge = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .metadata("user", "Lending Protocol User Badge")
            .initial_supply(1);

            // NFT description for user identification. 
            let user_address = ResourceBuilder::new_non_fungible()
            .metadata("user", "Lending Protocol User")
            .mintable(rule!(require(lending_protocol_user_badge.resource_address())), LOCKED)
            .burnable(rule!(require(lending_protocol_user_badge.resource_address())), LOCKED)
            .updateable_non_fungible_data(rule!(require(lending_protocol_user_badge.resource_address())), LOCKED)
            .no_initial_supply();
            
            return Self {
                user_badge_vault: Vault::with_bucket(lending_protocol_user_badge),
                user_address: user_address,
                user_id: 0,
            }
            .instantiate()
            .globalize()
        }

        pub fn user_exists(&self, user_auth: Proof) -> bool {
            return self.user_address.contains_key(Proof);
       }

        pub fn assert_user_exists(&self, user_auth: Proof, label: String) {
           assert!( self.user_exists(user_auth),
           "[{}]: No user exists", label);
       }

        pub fn assert_user_doesnt_exists(&self, user_auth: Proof, label: String) {
           assert!(!self.user_exists(user_auth),
           "[{}]: User exists", label);
       }

        fn users(&self) -> Vec<ResourceAddress> {
        return self.user_address.keys().cloned().collect::<Vec<ResourceAddress>>();
        }

        // Creates a new user for the lending protocol. Need to deposit a token to register as a user.
        // User is created to track the deposit balance, borrow balance, and the token of each.
        // Token is registered by extracting the resource address of the token they deposited.
        // Users are not given a badge. Badge is used by the protocol to update the state. Users are given an NFT to identify as a user.

        pub fn new_user(&mut self) -> Bucket {

            // Mint NFT to give to users as identification 
            let user = self.user_badge_vault.authorize(|| {
                let resource_manager = borrow_resource_manager!(self.user_address);
                resource_manager.mint_non_fungible(
                    // The User id
                    self.user_id,
                    // The User data
                    User {
                        borrow_balance: HashMap::new(),
                        deposit_balance: HashMap::new(),
                    },
                )
            });
            // Returns NFT to user
            return user
        }

        fn add_deposit_balance(&mut self, user_auth: Proof, address: ResourceAddress, amount: Decimal) -> Bucket {
            
            let user = self.user_badge_vault.authorize(|auth| {
                let resource_manager = borrow_resource_manager!(self.user_address);
                resource_manager.update_non_fungible_data(
                    self.user_id,
                    UserMut{
                        borrow_balance,
                        deposit balance: deposit_balance.get_mut(&address).unwrap() += amount,
                    },
                    auth
                )
        });

            return user
        }

    }
}