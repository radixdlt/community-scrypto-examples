use scrypto::prelude::*;
use crate::lending_pool::*;

#[derive(NonFungibleData)]
pub struct User {

    deposit_balance: HashMap<ResourceAddress, Decimal>,

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

        tracking_token_address_mapping: HashMap<ResourceAddress, ResourceAddress>,
        //Flash loan
        auth_vault: Vault,
        transient_resource_address: ResourceAddress,

        // Vault that holds the authorization badge
        user_badge_vault: Vault,
        // Collects User Address
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
                tracking_token_address_mapping: HashMap::new(),
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

        fn add_deposit_balance(&mut self, user_auth: Bucket, address: ResourceAddress, amount: Decimal) -> Bucket {

            let mut non_fungible_data: User = user_auth.non_fungible().data();

            *non_fungible_data.deposit_balance.get_mut(&address).unwrap() += amount;

            return user_auth
        }

        fn add_borrow_balance(&mut self, user_auth: Bucket, address: ResourceAddress, amount: Decimal) -> Bucket {

            let mut non_fungible_data: User = user_auth.non_fungible().data();

            *non_fungible_data.borrow_balance.get_mut(&address).unwrap() += amount;

            return user_auth
        }


        pub fn new_lending_pool(&mut self, deposit: Bucket) -> Bucket {
            // Checking if a liquidity pool already exists between these two tokens
            self.assert_pool_doesnt_exists(
                deposit.resource_address(), 
                String::from("New Liquidity Pool")
            );

            // Checking if user exists


            let address: ResourceAddress = deposit.resource_address();
            let (lending_pool, tracking_tokens): (ComponentAddress, Bucket) = LendingPool::new(
                deposit
            );
            // Inserts into lending pool hashmap
            self.lending_pools.insert(
                address,
                lending_pool.into()
            );
            // Inserts tracking token hashmap
            self.tracking_token_address_mapping.insert(
                tracking_tokens.resource_address(),
                address
            );

            return tracking_tokens;
        }

        pub fn deposit(&mut self, user_auth: Bucket, deposit_amount: Bucket) -> Bucket 
        {
            // Checks if user exists


            // Checks resource address of the deposit
            let address: ResourceAddress = deposit_amount.resource_address(); 

            let amount = deposit_amount.amount();


            // Attempting to get the lending pool component associated with the provided address pair.
            let optional_lending_pool: Option<&LendingPool> = self.lending_pools.get(&address);
            match optional_lending_pool {
                Some (lending_pool) => { // If it matches it means that the liquidity pool exists.
                    info!("[Lending Protocol Supply Tokens]: Pool for {:?} already exists. Adding supply directly.", address);
                    let returns: Bucket = lending_pool.deposit(deposit_amount); // Does this need auth?
                    // Update user state
                    self.add_deposit_balance(user_auth, address, amount);
                    returns
                }
                None => { // If this matches then there does not exist a liquidity pool for this token pair
                    // In here we are creating a new liquidity pool for this token pair since we failed to find an 
                    // already existing liquidity pool. The return statement below might seem somewhat redundant in 
                    // terms of the two empty buckets being returned, but this is done to allow for the add liquidity
                    // method to be general and allow for the possibility of the liquidity pool not being there.
                    info!("[DEX Add Liquidity]: Pool for {:?} doesn't exist. Creating a new one.", address);
                    self.new_lending_pool(deposit_amount)
                }
            }
        }

        pub fn borrow_supply(&mut self, user_auth: Bucket, deposit_amount: Bucket) -> Bucket 
        {
            // Checks if user exists


            // Checks resource address of the deposit
            let address: ResourceAddress = deposit_amount.resource_address(); 

            let amount = deposit_amount.amount();


            // Attempting to get the lending pool component associated with the provided address pair.
            let optional_lending_pool: Option<&LendingPool> = self.lending_pools.get(&address);
            match optional_lending_pool {
                Some (lending_pool) => { // If it matches it means that the liquidity pool exists.
                    info!("[Lending Protocol Supply Tokens]: Pool for {:?} already exists. Adding supply directly.", address);
                    let returns: Bucket = lending_pool.deposit(deposit_amount); // Does this need auth?
                    // Update user state
                    self.add_borrow_balance(user_auth, address, amount);
                    returns
                }
                None => { // If this matches then there does not exist a liquidity pool for this token pair
                    // In here we are creating a new liquidity pool for this token pair since we failed to find an 
                    // already existing liquidity pool. The return statement below might seem somewhat redundant in 
                    // terms of the two empty buckets being returned, but this is done to allow for the add liquidity
                    // method to be general and allow for the possibility of the liquidity pool not being there.
                    info!("[DEX Add Liquidity]: Pool for {:?} doesn't exist. Creating a new one.", address);
                    self.new_lending_pool(deposit_amount)
                }
            }
        }
        
    }
}