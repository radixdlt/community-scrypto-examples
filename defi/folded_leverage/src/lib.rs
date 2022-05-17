use sbor::*;
use scrypto::prelude::*;

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
    struct AutoLend {
        /// The liquidity pool
        liquidity_pool: Vault,
        /// The min collateral ratio that a user has to maintain
        min_collateral_ratio: Decimal,
        /// The max percent of debt one can liquidate
        max_liquidation_percent: Decimal,
        /// Liquidation bonus
        liquidation_bonus: Decimal,
        /// User state
        users: LazyMap<ResourceAddress, User>,
        /// The interest rate of deposits, per epoch
        deposit_interest_rate: Decimal,
        /// The (stable) interest rate of loans, per epoch
        borrow_interest_rate: Decimal,
        auth_vault: Vault,
        transient_resource_address: ResourceAddress,
    }

    impl AutoLend {
        /// Creates a lending pool, with single collateral.
        pub fn instantiate_autolend(reserve_address: ResourceAddress) -> ComponentAddress {

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
            

            Self {
                liquidity_pool: Vault::new(reserve_address),
                min_collateral_ratio: dec!("0.0"),
                max_liquidation_percent: dec!("0.5"),
                liquidation_bonus: dec!("0.05"),
                users: LazyMap::new(),
                deposit_interest_rate: dec!("0.01"),
                borrow_interest_rate: dec!("0.02"),
                auth_vault: Vault::with_bucket(auth_token),
                transient_resource_address: address,
            }
            .instantiate()
            .globalize()
        }

        /// Registers a new user
        pub fn new_user(&self) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "AutoLend User Badge")
                .initial_supply(1)
        }

        /// Deposits into the liquidity pool and start earning interest.
        pub fn deposit(&mut self, user_auth: Proof, reserve_tokens: Bucket) {
            let user_id = Self::get_user_id(user_auth);
            let amount = reserve_tokens.amount();

            // Update user state
            let deposit_interest_rate = self.deposit_interest_rate;
            let user = match self.users.get(&user_id) {
                Some(mut user) => {
                    user.on_deposit(amount, deposit_interest_rate);
                    user
                }
                None => User {
                    deposit_balance: amount,
                    borrow_balance: Decimal::zero(),
                    deposit_interest_rate,
                    borrow_interest_rate: Decimal::zero(),
                    deposit_last_update: Runtime::current_epoch(),
                    borrow_last_update: Runtime::current_epoch(),
                },
            };

            // Commit state changes
            self.users.insert(user_id, user);
            self.liquidity_pool.put(reserve_tokens);
        }

        /// Redeems the underlying assets, partially or in full.
        pub fn redeem(&mut self, user_auth: Proof, amount: Decimal) -> Bucket {
            let user_id = Self::get_user_id(user_auth);

            // Update user state
            let mut user = self.get_user(user_id);
            let to_return_amount = user.on_redeem(amount);
            user.check_collateral_ratio(self.min_collateral_ratio);

            debug!(
                "LP balance: {}, redeemded: {}",
                self.liquidity_pool.amount(),
                to_return_amount
            );

            // Commit state changes
            self.users.insert(user_id, user);
            self.liquidity_pool.take(to_return_amount)
        }

        /// Borrows the specified amount from lending pool
        pub fn borrow(&mut self, user_auth: Proof, requested: Decimal) -> (Bucket, Bucket) {
            let user_id = Self::get_user_id(user_auth);

            // Update user state
            let borrow_interest_rate = self.borrow_interest_rate;
            let mut user = self.get_user(user_id);
            let amount_due = requested;
            user.on_borrow(requested, borrow_interest_rate);
            user.check_collateral_ratio(self.min_collateral_ratio);

            let loan_terms = self.auth_vault.authorize(|| {
                borrow_resource_manager!(self.transient_resource_address).mint_non_fungible(
                    &NonFungibleId::random(),
                    LoanDue {
                        amount_due: amount_due,
                    },
                )
            });

            // Commit state changes
            self.users.insert(user_id, user);
            (self.liquidity_pool.take(requested), loan_terms) //Why don't you add ";" here?
            
        }

        /// Repays a loan, partially or in full.
        pub fn repay(&mut self, user_auth: Proof, mut repaid: Bucket, loan_terms: Bucket) -> Bucket {
            let user_id = Self::get_user_id(user_auth);

            assert!(
                loan_terms.resource_address() == self.transient_resource_address, 
                "Incorrect resource passed in for loan terms"
            );

            // Was working before inputting this
            let terms: LoanDue = loan_terms.non_fungible().data();
            assert!(
                repaid.amount() >= terms.amount_due,
                "Insufficient repayment given for your loan!"
            );

            self.auth_vault.authorize(|| loan_terms.burn());
 
            // Update user state
            let mut user = self.get_user(user_id);
            let to_return_amount = user.on_repay(repaid.amount());
            let to_return = repaid.take(to_return_amount);

            // Commit state changes
            self.users.insert(user_id, user);
            self.liquidity_pool.put(repaid);
            to_return
            
        }

        /// Liquidates one user's position, if it's under collateralized.
        pub fn liquidate(&mut self, user_id: ResourceAddress, repaid: Bucket) -> Bucket {
            let mut user = self.get_user(user_id);

            // Check if the user is under collateralized
            let collateral_ratio = user.get_collateral_ratio();
            if let Some(ratio) = collateral_ratio {
                assert!(
                    ratio <= self.min_collateral_ratio,
                    "Liquidation not allowed."
                );
            } else {
                panic!("No borrow from the user");
            }

            // Check liquidation size
            assert!(
                repaid.amount() <= user.borrow_balance * self.max_liquidation_percent,
                "Max liquidation percent exceeded."
            );

            // Update user state
            let to_return_amount = user.on_liquidate(repaid.amount(), self.max_liquidation_percent);
            let to_return = self.liquidity_pool.take(to_return_amount);

            // Commit state changes
            self.users.insert(user_id, user);
            to_return
        }

        /// Returns the current state of a user.
        pub fn get_user(&self, user_id: ResourceAddress) -> User {
            match self.users.get(&user_id) {
                Some(user) => user,
                _ => panic!("User not found"),
            }
        }

        /// Returns the deposit interest rate per epoch
        pub fn set_deposit_interest_rate(&mut self, rate: Decimal) {
            self.deposit_interest_rate = rate;
        }

        /// Returns the borrow interest rate per epoch
        pub fn set_borrow_interest_rate(&mut self, rate: Decimal) {
            self.borrow_interest_rate = rate;
        }

        /// Parse user id from a proof.
        fn get_user_id(user_auth: Proof) -> ResourceAddress {
            assert!(user_auth.amount() > dec!("0"), "Invalid user proof");
            user_auth.resource_address()
        }
    }
}

#[derive(Debug, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
pub struct User {
    /// The user's deposit balance
    pub deposit_balance: Decimal,
    /// The interest rate of deposits
    pub deposit_interest_rate: Decimal,
    /// Last update timestamp
    pub deposit_last_update: u64,

    /// The user's borrow balance
    pub borrow_balance: Decimal,
    /// The (stable) interest rate of loans
    pub borrow_interest_rate: Decimal,
    /// Last update timestamp
    pub borrow_last_update: u64,
}

impl User {
    pub fn get_collateral_ratio(&self) -> Option<Decimal> {
        if self.borrow_balance.is_zero() {
            None
        } else {
            let collateral = self.deposit_balance
                + self.deposit_balance * self.deposit_interest_rate * self.deposit_time_elapsed();

            let loan = self.borrow_balance
                + self.borrow_balance * self.borrow_interest_rate * self.borrow_time_elapsed();

            Some(collateral / loan)
        }
    }

    pub fn check_collateral_ratio(&self, min_collateral_ratio: Decimal) {
        let collateral_ratio = self.get_collateral_ratio();
        if let Some(ratio) = collateral_ratio {
            assert!(
                ratio >= min_collateral_ratio,
                "Min collateral ratio does not meet"
            );
        }
    }

    pub fn on_deposit(&mut self, amount: Decimal, interest_rate: Decimal) {
        // Increase principle balance by interests accrued
        let interest =
            self.deposit_balance * self.deposit_interest_rate * self.deposit_time_elapsed();
        self.deposit_balance += interest;
        self.deposit_last_update = Runtime::current_epoch();

        // Calculate the aggregated interest of previous deposits & the new deposit
        self.deposit_interest_rate = (self.deposit_balance * self.deposit_interest_rate
            + amount * interest_rate)
            / (self.deposit_balance + amount);

        // Increase principle balance by the amount.
        self.deposit_balance += amount;
    }

    pub fn on_redeem(&mut self, amount: Decimal) -> Decimal {
        // Deduct withdrawn amount from principle
        self.deposit_balance -= amount;

        // Calculate the amount to return
        amount + amount * self.deposit_interest_rate * self.deposit_time_elapsed()
    }

    pub fn on_borrow(&mut self, amount: Decimal, interest_rate: Decimal) {
        // Increase borrow balance by interests accrued
        let interest = self.borrow_balance * self.borrow_interest_rate * self.borrow_time_elapsed();
        self.borrow_balance += interest;
        self.borrow_last_update = Runtime::current_epoch();

        // Calculate the aggregated interest of previous borrows & the new borrow
        self.borrow_interest_rate = (self.borrow_balance * self.borrow_interest_rate
            + amount * interest_rate)
            / (self.borrow_balance + amount);

        // Increase principle balance by the amount.
        self.borrow_balance += amount;
    }

    pub fn on_repay(&mut self, amount: Decimal) -> Decimal {
        // Increase borrow balance by interests accrued
        let interest = self.borrow_balance * self.borrow_interest_rate * self.borrow_time_elapsed();
        self.borrow_balance += interest;
        self.borrow_last_update = Runtime::current_epoch();

        // Repay the loan
        if self.borrow_balance < amount {
            let to_return = amount - self.borrow_balance;
            self.borrow_balance = Decimal::zero();
            self.borrow_interest_rate = Decimal::zero();
            to_return
        } else {
            self.borrow_balance -= amount;
            Decimal::zero()
        }
    }

    pub fn on_liquidate(&mut self, amount: Decimal, bonus_percent: Decimal) -> Decimal {
        let changes = self.on_repay(amount);
        assert!(changes == 0.into());

        // TODO add exchange rate here when collaterals and borrows are different

        let to_return = amount * (bonus_percent + 1);
        self.deposit_balance -= to_return;
        to_return
    }

    fn deposit_time_elapsed(&self) -> u64 {
        // +1 is for demo purpose only
        Runtime::current_epoch() - self.deposit_last_update + 1
    }

    fn borrow_time_elapsed(&self) -> u64 {
        // +1 is for demo purpose only
        Runtime::current_epoch() - self.borrow_last_update + 1
    }
}
