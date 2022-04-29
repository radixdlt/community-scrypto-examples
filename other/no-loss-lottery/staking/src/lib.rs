use scrypto::prelude::*;

#[derive(NonFungibleData)]
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
        token_id_counter: u64,
        token_resource_address: ResourceAddress,
        // staking and rewards pools
        staking_pool: Vault,
        rewards_pool: Vault,
        // reward value by staking pool amount
        reward_value: Decimal,
        // last update epoch
        last_update: u64,
        // reward rate per staking
        rate: Decimal,
    }

    impl Staking {
        
        // initiate component with some rewards pool
        pub fn new(rewards: Bucket) -> ComponentAddress {
            let token_minter = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Staking Token Minter")
                .initial_supply(1);

            let token = ResourceBuilder::new_non_fungible()
                .metadata("name", "Staking Token")
                .metadata("symbol", "STT")
                .mintable(rule!(require(token_minter.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(token_minter.resource_address())), LOCKED)
                .no_initial_supply();

            Self {
                token_minter: Vault::with_bucket(token_minter),
                token_id_counter: 0u64,
                token_resource_address: token,
                staking_pool: Vault::new(rewards.resource_address()),
                rewards_pool: Vault::with_bucket(rewards),
                reward_value: Decimal::zero(),
                last_update: Runtime::current_epoch(),
                rate: dec!("2"),
            }
            .instantiate()
            .globalize()
        }

        // Calculates reward per token in supply
        fn get_reward_value(&self) -> Decimal {

            if self.staking_pool.is_empty() {
                debug!("Pool is empty");
                return Decimal::zero();
            }
            
            // calculate per token reward depending on the time passed since last epoch
            let time = Runtime::current_epoch()-self.last_update;
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
            self.last_update = Runtime::current_epoch();

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

            let bucket = self.token_minter.authorize(|| {
                borrow_resource_manager!(self.token_resource_address)
                    .mint_non_fungible(&NonFungibleId::from_u64(self.token_id_counter), token_data)
            });
            self.token_id_counter += 1;
            
            bucket
        }

        // staking for specific account
        pub fn stake(&mut self, staking: Bucket, auth: Proof) {
            assert_eq!(auth.resource_address(), self.token_resource_address, "Invalid badge provided");
            assert_eq!(auth.amount(), dec!("1"), "Invalid badge amount provided");

            // get nft data from bucket        
            let mut data: StakingData = auth.non_fungible().data();

            // update the reward
            let earned = self.update_reward(&data);
            let amount = staking.amount();

            // update rewards
            data.reward = earned;
            data.paid = self.reward_value;
            // update balance after
            data.balance += amount;

            debug!("Account staking balance: {}", data.balance);

            self.token_minter.authorize(|| auth.non_fungible().update_data(data));

            self.staking_pool.put(staking);
        }

        // withdraw the staking amount and the reward
        pub fn withdraw(&mut self, auth: Proof) -> Bucket {
            assert_eq!(auth.resource_address(), self.token_resource_address, "Invalid badge provided");
            assert_eq!(auth.amount(), dec!("1"), "Invalid badge provided");

            let mut data: StakingData = auth.non_fungible().data();

            let earned = self.update_reward(&data);
            data.reward = earned;
            data.paid = self.reward_value;
            // take staking amount
            let mut bucket = self.staking_pool.take(data.balance);
            // add reward from the reward pool
            bucket.put(self.rewards_pool.take(data.reward));
            // reset balance and reward
            data.balance = Decimal::zero();
            data.reward = Decimal::zero();
            
            // update NFT
            self.token_minter.authorize(|| auth.non_fungible().update_data(data));
            
            bucket
        }

    }
}
