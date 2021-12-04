use scrypto::prelude::*;


blueprint! {
    struct TimeLock {
        // Collected fees in XRD.
        collected_fees: Vault,
        // locks map
        locks_map: HashMap<Address, (Decimal,u64)>,
        
        // Locked XRD
        locked_xrd: Vault,

        // fee in percents
        fee_percent: Decimal,
    }

    impl TimeLock {
       
        /// Setup the Lock contract, assign an amount of XRD and the end time.
        pub fn make_timelock(fee: Decimal) -> Component {

            // Instantiate the Time Lock component.
            Self {
                collected_fees: Vault::new(RADIX_TOKEN),
                locks_map: HashMap::new(),
                locked_xrd: Vault::new(RADIX_TOKEN),
                fee_percent: fee
            }
            .instantiate()
        }

        /// Registers a new user
        pub fn new_user(&self) -> Bucket {
            ResourceBuilder::new()
                .metadata("name", "TimeLock User Badge")
                .new_badge_fixed(1)
        }

        /// Parse user id from a bucket ref.
        fn get_user_id(user_auth: BucketRef) -> Address {
            scrypto_assert!(user_auth.amount() > 0.into(), "Invalid user proof");
            let user_id = user_auth.resource_address();
            user_auth.drop();
            user_id
        }
        
        /// Lock XRD for a certain time.
        pub fn lock(&mut self, user_auth: BucketRef, lock_tokens: Bucket, duration: u64) {

            let user_id = Self::get_user_id(user_auth);
            let amount = lock_tokens.amount();

            scrypto_assert!(amount != Decimal::zero(), "You cannot lock zero amount");
            scrypto_assert!(duration != 0, "You cannot lock with a zero duration");

            //check if you already have locked amount
            let locked = match self.locks_map.get(&user_id) {
                Some(&value) => {
                    info!("current epoch {}", Context::current_epoch());
                   value
                }
                _ => (Decimal::zero(),0)
            };
            scrypto_assert!(locked.0 == Decimal::zero(), "You already locked tokens. Please use ADD method");            

            // fees calculation
            let fee_amount = amount * self.fee_percent/100;
            // setup fees to be taken from the payment
            let fee_tokens = lock_tokens.take(fee_amount);

            // Setup the end time.
            let end_time = Context::current_epoch() + duration;
            // Add lock with the rest tokens 
            self.locks_map.insert(user_id, (lock_tokens.amount(), end_time));
            // Put fees in collected XRD.
            self.collected_fees.put(fee_tokens);
            // put the rest amount of tokens to the locked vault
            self.locked_xrd.put(lock_tokens)
        }

        /// Add more XRD to the locked amount
        pub fn add(&mut self, user_auth: BucketRef, lock_tokens: Bucket) {
            let user_id = Self::get_user_id(user_auth);
            let amount = lock_tokens.amount();

            scrypto_assert!(amount != Decimal::zero(), "You cannot add zero amount");

            //check your locked amount
            let locked = match self.locks_map.get(&user_id) {
                Some(&value) => {
                    info!("current epoch {}", Context::current_epoch());
                   value
                }
                _ => (Decimal::zero(),0)
            };
            scrypto_assert!(locked.1 != 0, "No lock found for user. You can only add more tokens to an existing lock, please use LOCK method");  
            scrypto_assert!(Context::current_epoch() < locked.1, "Release time is over, there's no point to lock more tokens"); 

            // fees calculation
            let fee_amount = amount * self.fee_percent/100;
            // setup fees to be taken from the payment
            let fee_tokens = lock_tokens.take(fee_amount);
            // Calculate new lock amount
            let new_lock_amount = locked.0 + lock_tokens.amount();
            // Add lock with the rest tokens 
            self.locks_map.insert(user_id, (new_lock_amount, locked.1));
            // Put fees in collected XRD.
            self.collected_fees.put(fee_tokens);
            // put the rest amount of tokens to the locked vault
            self.locked_xrd.put(lock_tokens)
        }

        /// release locked XRD
        pub fn release(&mut self, user_auth: BucketRef,  amount: Decimal) -> Bucket {
            scrypto_assert!(amount != Decimal::zero(), "You cannot release zero amount");

            let user_id = Self::get_user_id(user_auth);

            let returns = Bucket::new(RADIX_TOKEN);
            match self.locks_map.get(&user_id) {
                Some(&value) => {
                info!("current epoch {}", Context::current_epoch());
                scrypto_assert!(Context::current_epoch() > value.1, "Release time not yet over, wait for a bit longer"); 
                scrypto_assert!(amount <= value.0, "Release amount is higher than the locked one");

                // update the locked amount
                self.locks_map.insert(user_id, (value.0 - amount, value.1));

                returns.put(self.locked_xrd.take(amount));
                },
                _ => error!("No locks found for this user")
            }

            returns
        }
        
        /// current locked XRD of a user
        pub fn locked(&mut self, user_auth: BucketRef) -> (Decimal, u64) {
            let user_id = Self::get_user_id(user_auth);

            let data = match self.locks_map.get(&user_id) {
                Some(&value) => {
                    info!("current epoch {}", Context::current_epoch());
                   value
                },
                _ => {
                    error!("No locks found for this user");
                    (Decimal::zero(),0)
                }
            };

            data
        }
    }
}
