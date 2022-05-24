use scrypto::prelude::*;
use crate::lending_pool::*;

#[derive(NonFungibleData, Describe, Encode, Decode, TypeId)]
pub struct User {
    // NFT to record user data
    #[scrypto(mutable)]
    deposit_balance: HashMap<ResourceAddress, Decimal>,
    #[scrypto(mutable)]
    borrow_balance: HashMap<ResourceAddress, Decimal>,
}

// Refactor to main component
#[derive(NonFungibleData)]
pub struct LoanDue {
    pub amount_due: Decimal,
}

// I took the Auto-Lend blueprint example from official Scrypto examples and added a transient token to explore concepts around "folded leverage"
// This would allow user to:
// 1. Deposit collateral to borrow
// 2. Borrow monies
// 3. Deposit borrowed monies
// 4. Borrow additional monies with added borrowed collateral
// 5. Repeat until desired leverage
// 6. Repay loans

// Currently, it's kinda basic and this is a working prototype. But I want to explore additional use-case with this concept.

// This is a barebone implementation of Lending protocol.
//
// Following features are missing:
// * Fees
// * Multi-collateral with price oracle
// * Variable interest rate
// * Authorization
// * Interest dynamic adjustment strategy
// * Upgradability

blueprint! {
    struct FoldedLeverage {

        lending_pools: HashMap<ResourceAddress, LendingPool>,
        //Flash loan
        auth_vault: Vault,
        transient_resource_address: ResourceAddress,
        // Vault that holds the authorization badge
        user_badge_vault: Vault,
        // Tracks NFT created by this component
        user_address: ResourceAddress,
    }

    impl FoldedLeverage {
        /// Creates a lending pool, with single collateral.
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

            // Creates badge to authorizie to mint/burn flash loan
            let auth_token = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Admin authority for BasicFlashLoan")
                .initial_supply(1);

            // Define a "transient" resource which can never be deposited once created, only burned
            let address = ResourceBuilder::new_non_fungible()
                .metadata(
                    "name",
                    "Promise token for BasicFlashLoan - must be returned to be burned!",
                )
                .mintable(rule!(require(auth_token.resource_address())), LOCKED)
                .burnable(rule!(require(auth_token.resource_address())), LOCKED)
                .restrict_deposit(rule!(deny_all), LOCKED)
                .no_initial_supply();
            
            // Difference between using return Self and just Self?
            return Self {
                lending_pools: HashMap::new(),
                auth_vault: Vault::with_bucket(auth_token),
                transient_resource_address: address,
                user_badge_vault: Vault::with_bucket(lending_protocol_user_badge),
                user_address: user_address,
            }
            .instantiate()
            .globalize()
        }

        /// Checks if a liquidity pool for the given pair of tokens exists or not.

        pub fn pool_exists(&self, address: ResourceAddress) -> bool {
            return self.lending_pools.contains_key(&address);
        }

        /// Asserts that a liquidity pool for the given address pair exists on the DEX.

        pub fn assert_pool_exists(&self, address: ResourceAddress, label: String) {
            assert!(
                self.pool_exists(address), 
                "[{}]: No lending pool exists for the given address pair.", 
                label
            );
        }
        
        /// Asserts that a liquidity pool for the given address pair doesn't exist on the DEX.
        
        pub fn assert_pool_doesnt_exists(&self, address: ResourceAddress, label: String) {
            assert!(
                !self.pool_exists(address), 
                "[{}]: A lending pool exists with the given address.", 
                label
            );
        }

        pub fn deposit_resource_exists(&self, user_auth: Proof, address: ResourceAddress) -> bool {
            let user_badge_data: User = user_auth.non_fungible().data();
            return user_badge_data.deposit_balance.contains_key(&address);
        }

        pub fn assert_deposit_resouce_exists(&self, user_auth: Proof, address: ResourceAddress, label: String) {
            assert!(self.deposit_resource_exists(user_auth, address), "[{}]: No resource exists for user.", label);
        }

        pub fn assert_deposit_resouce_doesnt_exists(&self, user_auth: Proof, address: ResourceAddress, label: String) {
            assert!(!self.deposit_resource_exists(user_auth, address), "[{}]: Resource exists for user.", label);
        }

        pub fn borrow_resource_exists(&self, user_auth: Proof, address: ResourceAddress) -> bool {
            let user_badge_data: User = user_auth.non_fungible().data();
            return user_badge_data.borrow_balance.contains_key(&address);
        }
        
        pub fn new_user(&mut self) -> Bucket {

            // Mint NFT to give to users as identification 
            let user = self.user_badge_vault.authorize(|| {
                let resource_manager = borrow_resource_manager!(self.user_address);
                resource_manager.mint_non_fungible(
                    // The User id
                    &NonFungibleId::random(),
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

        // Check if the user belongs to this lending protocol
        pub fn check_user_exist(&self, user_auth: Proof) -> bool {
            return user_auth.contains(self.user_address);
        }

        // Adds the deposit balance
        // Checks if the user already a record of the resource or not
        fn add_deposit_balance(&mut self, user_auth: Proof, address: ResourceAddress, amount: Decimal) {

            let mut non_fungible_data: User = user_auth.non_fungible().data();
                if non_fungible_data.deposit_balance.contains_key(&address) {
                    *non_fungible_data.deposit_balance.get_mut(&address).unwrap() += amount;
                }     
                else {
                    self.insert_deposit(user_auth, address, amount);
                    
                }
            }
        

        fn insert_deposit(&mut self, user_auth: Proof, address: ResourceAddress, amount: Decimal) {

            let mut non_fungible_data: User = user_auth.non_fungible().data();
            non_fungible_data.deposit_balance.insert(address, amount);
        }

        fn insert_borrow(&mut self, user_auth: Proof, address: ResourceAddress, amount: Decimal) {

            let mut non_fungible_data: User = user_auth.non_fungible().data();
            non_fungible_data.borrow_balance.insert(address, amount);
        }

        fn add_borrow_balance(&mut self, user_auth: Proof, address: ResourceAddress, amount: Decimal) {

            let mut non_fungible_data: User = user_auth.non_fungible().data();
                if non_fungible_data.borrow_balance.contains_key(&address) {
                    *non_fungible_data.borrow_balance.get_mut(&address).unwrap() += amount;
                }     
                else {
                    self.insert_borrow(user_auth, address, amount);
                }
            }


        pub fn new_lending_pool(&mut self, user_auth: Proof, deposit: Bucket) {
            // Checking if a liquidity pool already exists between these two tokens
            self.assert_pool_doesnt_exists(
                deposit.resource_address(), 
                String::from("New Liquidity Pool")
            );

            // Checking if user exists


            let address: ResourceAddress = deposit.resource_address();
            let lending_pool: ComponentAddress = LendingPool::new(user_auth, deposit);
            // Inserts into lending pool hashmap
            self.lending_pools.insert(
                address,
                lending_pool.into()
            );

        }

        pub fn deposit(&mut self, user_auth: Proof, deposit_amount: Bucket) 
        {
            // Checks resource address of the deposit
            let address: ResourceAddress = deposit_amount.resource_address(); 

            let amount = deposit_amount.amount();

            let clone_user_auth: Proof = user_auth.clone();

            // Checks if user NFT contains resources
            self.assert_deposit_resouce_doesnt_exists(clone_user_auth, address, 
            String::from("Adding resource to account"));

            // Attempting to get the lending pool component associated with the provided address pair.
            let optional_lending_pool: Option<&LendingPool> = self.lending_pools.get(&address);
            match optional_lending_pool {
                Some (lending_pool) => { // If it matches it means that the liquidity pool exists.
                    info!("[Lending Protocol Supply Tokens]: Pool for {:?} already exists. Adding supply directly.", address);
                    lending_pool.deposit(user_auth, deposit_amount); // Does this need auth?
                    // Update user state
                    self.add_deposit_balance(user_auth, address, amount);

                }
                None => { // If this matches then there does not exist a liquidity pool for this token pair
                    // In here we are creating a new liquidity pool for this token pair since we failed to find an 
                    // already existing liquidity pool. The return statement below might seem somewhat redundant in 
                    // terms of the two empty buckets being returned, but this is done to allow for the add liquidity
                    // method to be general and allow for the possibility of the liquidity pool not being there.
                    info!("[DEX Add Liquidity]: Pool for {:?} doesn't exist. Creating a new one.", address);
                    self.new_lending_pool(user_auth, deposit_amount)
                }
            }
        }

        pub fn borrow(&mut self, user_auth: Proof, token_requested: ResourceAddress, borrow_amount: Bucket) 
        {
            // Checks if user exists


            // Checks resource address of the deposit
            let address: ResourceAddress = borrow_amount.resource_address(); 
            
            let amount = borrow_amount.amount();

            
            assert_eq!(token_requested, address, "Amount sent must be the same token as the token requested");

            // Attempting to get the lending pool component associated with the provided address pair.
            let optional_lending_pool: Option<&LendingPool> = self.lending_pools.get(&address);
            match optional_lending_pool {
                Some (lending_pool) => { // If it matches it means that the liquidity pool exists.
                    info!("[Lending Protocol Supply Tokens]: Pool for {:?} already exists. Adding supply directly.", address);
                    lending_pool.deposit(user_auth, borrow_amount); // Does this need auth?
                    // Update user state
                    self.add_borrow_balance(user_auth, address, amount);

                }
                None => { // If this matches then there does not exist a liquidity pool for this token pair
                    // In here we are creating a new liquidity pool for this token pair since we failed to find an 
                    // already existing liquidity pool. The return statement below might seem somewhat redundant in 
                    // terms of the two empty buckets being returned, but this is done to allow for the add liquidity
                    // method to be general and allow for the possibility of the liquidity pool not being there.
                    info!("[DEX Add Liquidity]: Pool for {:?} doesn't exist. Creating a new one.", address);
                    self.new_lending_pool(user_auth, borrow_amount)
                }
            }
        }

        // Still need to finish
        pub fn redeem(&mut self, user_auth: Proof, token_reuqested: ResourceAddress, amount: Decimal) {
            // Check if deposit withdrawal request has no lien
            let user_badge_data: User = user_auth.non_fungible().data();
            assert_eq!(user_badge_data.borrow_balance.get(&token_requested) > amount, "Need to fully repay loan");
            let optional_lending_pool: Option<&LendingPool> = self.lending_pools.get(&token_reuqested);
            match optional_lending_pool {
                Some (lending_pool) => { // If it matches it means that the liquidity pool exists.
                    info!("[Lending Protocol Supply Tokens]: Pool for {:?} already exists. Adding supply directly.", token_reuqested);
                    lending_pool.redeem(user_auth, token_reuqested, amount); // Does this need auth?
                    // Update user state
                    self.add_borrow_balance(user_auth, token_reuqested, amount);

                }
                None => { // If this matches then there does not exist a liquidity pool for this token pair
                    // In here we are creating a new liquidity pool for this token pair since we failed to find an 
                    // already existing liquidity pool. The return statement below might seem somewhat redundant in 
                    // terms of the two empty buckets being returned, but this is done to allow for the add liquidity
                    // method to be general and allow for the possibility of the liquidity pool not being there.
                    info!("[DEX Add Liquidity]: Pool for {:?} doesn't exist. Creating a new one.", token_reuqested);
                }
            }

        }
        
    }
}