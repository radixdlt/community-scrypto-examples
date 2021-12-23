use scrypto::prelude::*;

blueprint! {
    struct Staking {
        // staking and rewards pools
        staking_pool: Vault,
        rewards_pool: Vault,
        // reward value by staking pool amount
        reward_value: Decimal,
        // last update epoch
        last_update: u64,
        // reward rate per staking
        rate: Decimal,
        // collects rewards per account
        rewards: HashMap<Address, Decimal>,
        // collects balance per account
        balances: HashMap<Address, Decimal>,
        // collects paid reward per account
        paid: HashMap<Address, Decimal>
    }

    impl Staking {
        
        // initiate component with some rewards pool
        pub fn new(rewards: Bucket) -> Component {
            
            let rewards_ref = rewards.resource_def();
            Self {
                staking_pool: Vault::new(rewards_ref),
                rewards_pool: Vault::with_bucket(rewards),
                reward_value: Decimal::zero(),
                last_update: Context::current_epoch(),
                rate: 2.into(),
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

            // calculate per token reward depending on the time passed since last epoch
            let time = Context::current_epoch()-self.last_update;
            let exp = 1e18 as i64;
            let exp_rate = Decimal::from(exp) / self.staking_pool.amount();
            let epoch_rate = self.rate / 10000;
            let calculation = Decimal::from(time) * epoch_rate * exp_rate;
            let result = self.reward_value + calculation;
            result
        }

        // calculate earning 
        fn earned(&self, account: Address) -> Decimal {

            let exp = 1e18 as i64;
            let reward_value = self.get_reward_value();

            // get account data
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

            // calculate reward per account by balance current reward and collected paid reward
            let calculation = (balance * (reward_value - paid)) / Decimal::from(exp);

            debug!("Earned calculation: {}", calculation);

            let result = reward + calculation;

            debug!("Earned result: {}", result);

            result 
        }
     
        // private method that is used to update rewards during the epoch
        // will be called on each public request
        fn update_reward(&mut self, account: Address) {
            // update per token reward
            self.reward_value = self.get_reward_value();
            // update epoch
            self.last_update = Context::current_epoch();
            // calculate earnings
            let earned = self.earned(account);
            // store earnings for the account
            self.rewards.insert(account, earned);
            // set updated per token reward value to the account
            self.paid.insert(account, self.reward_value);
        }


        pub fn stake(&mut self, account: Address, bucket: Bucket) -> Bucket {
            self.update_reward(account);
            let amount = bucket.amount();

            let balance = self.balances.entry(account).or_insert(Decimal::zero());
            *balance += amount;

            debug!("Account staking balance: {}", balance);

            self.staking_pool.put(bucket.take(amount));

            bucket
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

        // returns rewards value per account
        pub fn account_reward(&mut self, account: Address) -> Decimal {
            assert!(self.rewards.contains_key(&account), "No rewards found");            

            self.update_reward(account);

            *self.rewards.get(&account).unwrap()
        }
    
        // send rewards to account
        pub fn request_reward(&mut self, account: Address) -> Bucket{
            // get the reward value
            let reward = self.account_reward(account);
            debug!("Account reward to send: {}", reward);
            // take reward from the rewards pool
            let bucket = self.rewards_pool.take(reward);
            // reset rewards for this account
            self.rewards.insert(account, Decimal::zero());
            // send rewards bucket
            bucket
        }
    }
}
