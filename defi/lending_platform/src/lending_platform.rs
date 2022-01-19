use sbor::*;
use scrypto::prelude::*;

blueprint! {
    struct LendingPlatform {
        assets: LazyMap<Address, Vault>,
        asset_ltv_ratios: LazyMap<Address, Decimal>,
        loan_balances: LazyMap<Address, Decimal>,
        users: LazyMap<Address, User>,
        admin_badge: ResourceDef
    }

    impl LendingPlatform {
        fn calculate_total_collateral(&self, user: &User) -> Decimal {
            /*
            calculate_total_collateral calculates the total value a user has of each deposited
            asset type in terms of XRD, then applies an LTV modifier based on each asset type to
            calculate the total collateral a user has.

            Total Collateral =
                quantity_asset_1 * xrd_price_asset_1 * ltv_asset_1 +
                quantity_asset_2 * xrd_price_asset_2 * ltv_asset_2 +
                ...
                quantity_asset_n * xrd_price_asset_n * ltv_asset_n
             */
            let user_address = user.user_address;

            info!("[LendingPlatform][USER:{}] Calculating Total Collateral.", user_address);

            // Total collateral a user has across all their assets
            let mut user_collateral_sum : Decimal = 0.into();

            // Iterate over each asset and calculate the amount of collateral available from each
            for (asset_address, asset_amount) in &user.deposit_balances {

                // TODO: Pull this data from an oracle - right now assuming all assets have a 1:1 ratio with the price of radix
                // We have to do this wonky behavior to create a Decimal. This will be improved in the future
                // let cost_of_asset_in_terms_of_xrd = Decimal::from(54564)/10000;
                let cost_of_asset_in_terms_of_xrd = Decimal::from(10000)/10000;

                let ltv = self.asset_ltv_ratios.get(asset_address).unwrap();

                let asset_value_in_xrd = *asset_amount * cost_of_asset_in_terms_of_xrd;
                let asset_collateral = asset_value_in_xrd * ltv;
                user_collateral_sum += asset_collateral;

                info!(
                    "[LendingPlatform][USER:{}] Asset={}, Amount={}, Value in XRD={}, LTV={}, Collateral Amount={}",
                    user_address,
                    asset_address,
                    asset_amount,
                    asset_value_in_xrd,
                    ltv,
                    asset_collateral
                );
            }
            user_collateral_sum.into()
        }

        fn calculate_total_loan_balance(&self, user: &User) -> Decimal {

            let user_address = user.user_address;

            info!("[LendingPlatform][USER:{}] Calculating Total Loan Balance.", user_address);

            // Total loan balance a user has across all their borrowed assets
            let mut total_loan_balance : Decimal = 0.into();

            // Iterate over each asset and sum the total loan balance
            for (asset_address, asset_amount) in &user.borrow_balances {

                // TODO: Pull this data from an oracle - right now assuming all assets have a 1:1 ratio with the price of radix
                let cost_of_asset_in_terms_of_xrd = Decimal::from(10000)/10000;

                let loan_balance_in_terms_of_xrd = *asset_amount * cost_of_asset_in_terms_of_xrd;

                info!(
                    "[LendingPlatform][USER:{}] Asset={}, Amount={}, Value in XRD={}",
                    user_address,
                    asset_address,
                    asset_amount,
                    loan_balance_in_terms_of_xrd
                );

                total_loan_balance += loan_balance_in_terms_of_xrd;
            }
            total_loan_balance.into()
        }

        fn calculate_available_collateral(&self, user: &User) -> Decimal {
            let user_address = user.user_address;

            let users_total_collateral = self.calculate_total_collateral(user);
            let users_loan_balance = self.calculate_total_loan_balance(user);
            let available_collateral = users_total_collateral - users_loan_balance;
            info!(
                "[LendingPlatform][USER:{}] Total collateral of {} XRD. Current loan balance of \
                {} XRD. Available collateral of {} XRD.",
                user_address,
                users_total_collateral,
                users_loan_balance,
                available_collateral
            );
            available_collateral
        }

        pub fn new() -> (Component, Bucket) {
            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Lending Platform Admin Badge")
                .initial_supply_fungible(1);

            let lending_platform = Self {
                assets: LazyMap::new(),
                asset_ltv_ratios: LazyMap::new(),
                loan_balances: LazyMap::new(),
                users: LazyMap::new(),
                admin_badge: admin_badge.resource_def()
            }
            .instantiate();
            (lending_platform, admin_badge)
        }

        pub fn new_user(&self) -> Bucket { // return a badge
            let user_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Lending Platform Badge")
                .initial_supply_fungible(1);

            let user = User {
                    user_address: user_badge.resource_address(),
                    deposit_balances: HashMap::new(),
                    borrow_balances: HashMap::new()
                };
            self.users.insert(user_badge.resource_address(), user);
            info!(
                "[LendingPlatform][POOL] Created new user with an address of {}.",
                user_badge.resource_address()
            );
            user_badge
        }

        #[auth(admin_badge)]
        pub fn new_asset(&self, asset_address: Address, ltv_ratio: Decimal) {
            assert! (
                ltv_ratio >= 0.into() && ltv_ratio <= 1.into(),
                "[LendingPlatform][POOL] LTV Ratio must be between 0.0 & 1.0 (inclusive)."
                );
            match self.asset_ltv_ratios.get(&asset_address) {
                Some(..) =>  {
                    panic!("[LendingPlatform][POOL] Asset {} already exists in lending pool.", asset_address);
                },
                None => {
                    self.asset_ltv_ratios.insert(asset_address, ltv_ratio);
                    info!(
                        "[LendingPlatform][POOL] Added asset {} with LTV ratio of {} to the lending pool.",
                        asset_address,
                        ltv_ratio
                    )
                }
            };
        }

        pub fn deposit_asset(&self, user_auth: BucketRef, asset: Bucket) {
            let asset_address = asset.resource_address();
            let amount = asset.amount();

            let mut user = self.users.get(&user_auth.resource_address()).unwrap();
            let user_address = user.user_address;

            match self.assets.get(&asset_address) {
                Some(x) =>  x.put(asset),
                None => {
                    let vault = Vault::with_bucket(asset);
                    self.assets.insert(asset_address, vault);
                }
            };

            // Update User Deposit Balance
            user.increase_deposit_balance(asset_address, amount);
            self.users.insert(user_auth.resource_address(), user);
            info!(
                "[LendingPlatform][USER:{}] Depositing {} of {}.",
                user_address,
                amount,
                asset_address
            );

            info!(
                "[LendingPlatform][POOL] New liquidity balance for {} is {}.",
                asset_address,
                self.assets.get(&asset_address).unwrap().amount()
            );
            user_auth.drop();
        }

        pub fn withdrawal_asset(
            &self,
            user_auth: BucketRef,
            asset_address: Address,
            amount: Decimal
        ) -> Bucket {
            let mut user = self.users.get(&user_auth.resource_address()).unwrap();
            let user_address = user.user_address;

            let removed_asset = match self.assets.get(&asset_address) {
                Some(x) =>  {
                    assert! (
                        x.amount() >= amount,
                        "[LendingPlatform][POOL] Pool only has {} of asset {} but {} was requested",
                        x.amount(),
                        asset_address,
                        amount
                        );
                    x.take(amount)
                },
                None => {
                     panic!(
                        "[LendingPlatform][POOL] Cannot decrease balance of asset {}. \
                        No liquidity exists yet for it.",
                        asset_address
                    )
                }
            };

            // Update User Deposit Balance
            user.decrease_deposit_balance(asset_address, amount);
            self.users.insert(user_auth.resource_address(), user);
            info!(
                "[LendingPlatform][USER:{}] Removing {} of {}.",
                user_address,
                amount,
                asset_address
            );

            info!(
                "[LendingPlatform][POOL] New liquidity balance for {} is {}.",
                asset_address,
                self.assets.get(&asset_address).unwrap().amount()
            );
            user_auth.drop();
            removed_asset
        }

        pub fn borrow_asset(
            &self,
            user_auth: BucketRef,
            asset_address: Address,
            amount: Decimal
        ) -> Bucket {
            let mut user = self.users.get(&user_auth.resource_address()).unwrap();
            let user_address = user.user_address;

            let borrowed_asset = match self.assets.get(&asset_address) {
                Some(x) =>  {
                    assert! (
                        x.amount() >= amount,
                        "[LendingPlatform][POOL] Pool only has {} of asset {} but {} was requested",
                        x.amount(),
                        asset_address,
                        amount
                        );
                    x.take(amount) // This only occurs if the assertion does not fail
                },
                None => {
                    panic!(
                        "[LendingPlatform][POOL] No asset of type {} are currently in the liquidity pool",
                        asset_address
                    );
                }
            };

            // Calculate the XRD value of the asset the user is attempting to borrow
            // TODO: Pull this data from an oracle - right now assuming all assets have a 1:1 ratio with the price of radix
            let cost_of_asset_in_terms_of_xrd = Decimal::from(10000)/10000;
            let borrow_amount_in_terms_of_xrd = amount * cost_of_asset_in_terms_of_xrd;

            // Check if user has enough collateral available for the loan (this takes into account LTV)
            let user_available_collateral = self.calculate_available_collateral(&user);
            info!(
                "[LendingPlatform][USER:{}] Available collateral in terms of XRD: {}",
                user_address,
                user_available_collateral
            );
            assert! (
                user_available_collateral >= borrow_amount_in_terms_of_xrd,
                "[LendingPlatform][POOL] User does not have enough collateral. Requested loan with \
                value of {} XRD but only has {} XRD of available collateral.",
                borrow_amount_in_terms_of_xrd,
                user_available_collateral
            );

            // Update User Borrowed Balance
            user.increase_borrowed_balance(asset_address, amount);
            self.users.insert(user_auth.resource_address(), user);
            info!(
                "[LendingPlatform][USER:{}] Borrowing {} of {}.",
                user_address,
                amount,
                asset_address
            );

            let updated_asset_amount = self.assets.get(&asset_address).unwrap().amount();
            info!(
                "[LendingPlatform][POOL] New liquidity balance for {} is {}.",
                asset_address,
                updated_asset_amount
            );
            user_auth.drop();
            borrowed_asset
        }

        pub fn repay_asset(&self, user_auth: BucketRef, asset: Bucket) {
            let asset_address = asset.resource_address();
            let amount = asset.amount();

            let mut user = self.users.get(&user_auth.resource_address()).unwrap();
            let user_address = user.user_address;

            info!(
                "[LendingPlatform][USER:{}] Repaying {} of {}.",
                user_address,
                amount,
                asset_address
            );

            match self.assets.get(&asset_address) {
                Some(x) =>  {
                    x.put(asset);
                    let updated_asset_amount = x.amount();

                    info!(
                        "[LendingPlatform][POOL] New liquidity balance for {} is {}.",
                        asset_address,
                        updated_asset_amount
                        );
                },
                None => {
                    panic!("[LendingPlatform] No asset of type {} are currently in the liquidity pool", asset_address);
                }
            }

            // Update User Borrowed Balance
            let updated_borrow_balance = user.decrease_borrowed_balance(asset_address, amount);
            self.users.insert(user_auth.resource_address(), user);
            info!(
                "[LendingPlatform][USER:{}] Borrow balance for asset {} is {}.",
                user_address,
                asset_address,
                updated_borrow_balance
            );
            user_auth.drop();
        }
    }
}

#[derive(TypeId, Encode, Decode, Describe)]
pub struct User {
    pub user_address: Address,
    pub deposit_balances: HashMap<Address, Decimal>,
    pub borrow_balances: HashMap<Address, Decimal>,
}

impl User {
    pub fn increase_deposit_balance(&mut self, address: Address, amount: Decimal) {
        match self.deposit_balances.get(&address) {
            Some(current_balance) => {
                let new_balance = *current_balance + amount;

                self.deposit_balances.insert(address, new_balance);
                info!(
                    "[User][USER:{}] Updated deposit balance for asset {} is {}",
                    self.user_address, address, new_balance
                );
            }
            None => {
                self.deposit_balances.insert(address, amount);
                info!(
                    "[User][USER:{}] No existing balance - New deposit balance for asset {} is {}",
                    self.user_address, address, amount
                );
            }
        };
    }

    pub fn decrease_deposit_balance(&mut self, address: Address, amount: Decimal) {
        match self.deposit_balances.get(&address) {
            Some(current_balance) => {
                assert!(
                    current_balance >= &amount,
                    "[User] Cannot create a negative balance"
                );
                let new_balance = *current_balance - amount;
                self.deposit_balances.insert(address, new_balance);
                info!(
                    "[User][USER:{}] Updated deposit balance for asset {} is {}",
                    self.user_address, address, new_balance
                );
            }
            None => {
                panic!(
                    "[User] Cannot decrease balance of asset {} as the user has no record of said asset.",
                    address
                );
            }
        };
    }

    pub fn increase_borrowed_balance(&mut self, address: Address, amount: Decimal) {
        match self.borrow_balances.get(&address) {
            Some(current_balance) => {
                let new_balance = *current_balance + amount;
                info!(
                    "[User][USER:{}] Updated deposit balance for asset {} is {}",
                    self.user_address, address, new_balance
                );
                self.borrow_balances.insert(address, new_balance)
            }
            None => self.borrow_balances.insert(address, amount),
        };
    }

    pub fn decrease_borrowed_balance(&mut self, address: Address, amount: Decimal) -> Decimal {
        return match self.borrow_balances.get(&address) {
            Some(current_balance) => {
                assert!(
                    *current_balance >= amount,
                    "[User] Cannot create a negative balance"
                );
                let new_balance = *current_balance - amount;
                info!(
                    "[User][USER:{}] Updated deposit balance for asset {} is {}",
                    self.user_address, address, new_balance
                );
                self.borrow_balances.insert(address, new_balance);
                new_balance
            }
            None => {
                self.borrow_balances.insert(address, amount);
                amount
            }
        };
    }
}
