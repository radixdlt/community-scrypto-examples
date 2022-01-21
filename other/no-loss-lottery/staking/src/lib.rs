use scrypto::prelude::*;

#[derive(NftData)]
pub struct StakingData {
    #[scrypto(mutable)]
    reward: Decimal,
    #[scrypto(mutable)]
    balance: Decimal,
    #[scrypto(mutable)]
    paid: Decimal
}

blueprint! {
    struct Staking {
        token_minter: Vault,
        token_id_counter: u128,
        token_resource_def: ResourceDef,
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
        // rewards: HashMap<Address, Decimal>,
        // collects balance per account
        // balances: HashMap<Address, Decimal>,
        // collects paid reward per account
        // paid: HashMap<Address, Decimal>
    }

    impl Staking {
        
        // initiate component with some rewards pool
        pub fn new(rewards: Bucket) -> Component {
            
            let rewards_ref = rewards.resource_def();

            let token_minter = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
            .metadata("name", "Staking Token Minter")
            .initial_supply_fungible(1);

            let token = ResourceBuilder::new_non_fungible()
            .metadata("name", "Staking Token")
            .metadata("symbol", "STT")
            .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
            .badge(token_minter.resource_def(), MAY_MINT | MAY_CHANGE_INDIVIDUAL_METADATA)
            .no_initial_supply();

            Self {
                token_minter: Vault::with_bucket(token_minter),
                token_id_counter: 0,
                token_resource_def: token,
                staking_pool: Vault::new(rewards_ref),
                rewards_pool: Vault::with_bucket(rewards),
                reward_value: Decimal::zero(),
                last_update: Context::current_epoch(),
                rate: 2.into(),
                // rewards: HashMap::new(),
                // balances: HashMap::new(),
                // paid: HashMap::new()
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
            // 10k epochs as possible way to calculate the yearly rate to get the rate per epoch
            let epoch_rate = self.rate / 10_000;
            let calculation = Decimal::from(time) * epoch_rate * exp_rate;
            let result = self.reward_value + calculation;
            result
        }

        // calculate earning 
        fn earned(&self, data: &StakingData) -> Decimal {

            let exp = 1e18 as i64;
            let reward_value = self.get_reward_value();

            let balance = data.balance;
            debug!("Account balance: {}", balance);

            let reward = data.reward;
            debug!("Account reward: {}", reward);

            let paid = data.paid;

            // calculate reward per account by balance current reward and collected paid reward
            let calculation = (balance * (reward_value - paid)) / Decimal::from(exp);

            debug!("Earned calculation: {}", calculation);

            let result = reward + calculation;

            debug!("Earned result: {}", result);

            result 
        }
     
        // private method that is used to update rewards during the epoch
        // will be called on each public request
        fn update_reward(&mut self, data: &StakingData) -> Decimal{
            // update per token reward
            self.reward_value = self.get_reward_value();
            // update epoch
            self.last_update = Context::current_epoch();

            let earned = self.earned(&data);

            earned
        }

        /// Registers a new user
        pub fn new_user(&mut self) -> Bucket {
            // mint course NFT
            let token_data = StakingData {
                reward: Decimal::zero(),
                balance: Decimal::zero(),
                paid: Decimal::zero()
            };

            let bucket = self.token_minter.authorize(|auth| {
                self.token_resource_def
                    .mint_nft(self.token_id_counter, token_data, auth)
            });
            self.token_id_counter += 1;
            
            bucket
        }

        // staking for specific account
        #[auth(token_resource_def)]
        pub fn stake(&mut self, staking: Bucket) {
            // let user_id = Self::get_user_id(user_auth);

             // get nft data from bucket
            let id = auth.get_nft_id();
            let mut data: StakingData = self.token_resource_def.get_nft_data(id);

            // update the reward
            let earned = self.update_reward(&data);
            let amount = staking.amount();

            // update rewards
            data.reward = earned;
            data.paid = self.reward_value;
            // update balance after
            data.balance += amount;

            debug!("Account staking balance: {}", data.balance);

            self.token_minter
            .authorize(|auth| self.token_resource_def.update_nft_data(id, data, auth));

            self.staking_pool.put(staking);
        }

        // withdraw the staking amount and the reward
        #[auth(token_resource_def)]
        pub fn withdraw(&mut self) -> Bucket {
            let id = auth.get_nft_id();
            let mut data: StakingData = self.token_resource_def.get_nft_data(id);

            let earned = self.update_reward(&data);
            data.reward = earned;
            data.paid = self.reward_value;
            // take staking amount
            let bucket = self.staking_pool.take(data.balance);
            // add reward from the reward pool
            bucket.put(self.rewards_pool.take(data.reward));
            // reset balance and reward
            data.balance = Decimal::zero();
            data.reward = Decimal::zero();
            
            // update NFT
            self.token_minter
            .authorize(|auth| self.token_resource_def.update_nft_data(id, data, auth));
            
            bucket
        }

    }
}
