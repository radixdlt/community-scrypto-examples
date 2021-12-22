use scrypto::prelude::*;

blueprint! {
    struct Staking {
        staking_pool: Vault,
        rewards_pool: Vault,
        reward_value: Decimal,
        last_update: u64,
        rate: Decimal,
        rewards: HashMap<Address, Decimal>,
        balances: HashMap<Address, Decimal>,
        paid: HashMap<Address, Decimal>
    }

    impl Staking {
        
        
        pub fn new(assets: Bucket, rewards: Bucket) -> Component {
            
            Self {
                staking_pool: Vault::with_bucket(assets),
                rewards_pool: Vault::with_bucket(rewards),
                reward_value: Decimal::zero(),
                last_update: Context::current_epoch(),
                rate: 100.into(),
                rewards: HashMap::new(),
                balances: HashMap::new(),
                paid: HashMap::new()
            }
            .instantiate()
        }

        // Calculates reward per token in supply
        fn get_reward_value(&self) -> Decimal {
            if self.staking_pool.is_empty() {
                debug!("Pool is empty");
                return Decimal::zero();
            }

            let time = Context::current_epoch()-self.last_update;
            debug!("Time difference: {}", time);
            let exp = 1e18 as i64;
            let calculation = (Decimal::from(time) * self.rate * Decimal::from(exp)) / self.staking_pool.amount();
            debug!("Period reward value: {}", calculation);
            let result = self.reward_value + calculation;
            debug!("Summarized reward value: {}", result);
            result
        }

        fn earned(&self, account: Address) -> Decimal {
            let exp = 1e18 as i64;
            let reward_value = self.get_reward_value();

            let balance = match self.balances.get(&account) {
                Some(value) => *value,
                None => Decimal::zero()
            };

            debug!("Account balance: {}", balance);

            let reward = match self.rewards.get(&account){
                Some(value) => *value,
                None => Decimal::zero()
            };

            debug!("Account reward: {}", reward);

            let paid = match self.paid.get(&account){
                Some(value) => *value,
                None => Decimal::zero()
            };

            debug!("Account paid: {}", paid);

            let calculation = (balance * reward_value - paid) / Decimal::from(exp);

            debug!("Earned calculation: {}", calculation);

            let result = reward + calculation;

            debug!("Earned result: {}", result);

            result 
        }
     
        fn update_reward(&mut self, account: Address) {

            self.reward_value = self.get_reward_value();
            self.last_update = Context::current_epoch();

            let earned = self.earned(account);
            self.rewards.insert(account, earned);
            self.paid.insert(account, self.reward_value);
        }


        pub fn stake(&mut self, account: Address, bucket: Bucket) {
            self.update_reward(account);
            let amount = bucket.amount();

            let balance = self.balances.entry(account).or_insert(Decimal::zero());
            *balance += amount;

            debug!("Account staking balance: {}", balance);

            self.staking_pool.put(bucket.take(amount));
        }
    
        pub fn withdraw(&mut self, account: Address, amount: Decimal) -> Bucket {
            assert!(self.balances.contains_key(&account), "No staking amount found");            
            self.update_reward(account);

            let balance = self.balances.get_mut(&account).unwrap();
            assert!(*balance >= amount, "Not enough assets in staking");            

            let bucket = self.staking_pool.take(amount);
            
            *balance -= amount;

            debug!("Account withdraw balance: {}", balance);

            bucket
        }
    
        pub fn get_reward(&mut self, account: Address) -> Bucket{
            assert!(self.rewards.contains_key(&account), "No rewards found");            

            self.update_reward(account);

            let reward = self.rewards.get(&account).unwrap();

            debug!("Account reward to send: {}", reward);

            let bucket = self.rewards_pool.take(*reward);
            
            self.rewards.insert(account, Decimal::zero());
            bucket
        }
    }
}
