use scrypto::prelude::*;

import! { r#"
{
    "package": "01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c",
    "name": "Staking",
    "functions": [
      {
        "name": "new",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "scrypto::core::Component",
          "generics": []
        }
      }
    ],
    "methods": [
      {
        "name": "new_user",
        "mutability": "Immutable",
        "inputs": [],
        "output": {
          "type": "Custom",
          "name": "scrypto::resource::Bucket",
          "generics": []
        }
      },
      {
        "name": "stake",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::BucketRef",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Unit"
        }
      },
      {
        "name": "account_reward",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::BucketRef",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "scrypto::types::Decimal",
          "generics": []
        }
      },
      {
        "name": "withdraw",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::BucketRef",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "scrypto::resource::Bucket",
          "generics": []
        }
      },
      {
        "name": "get_reward",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::BucketRef",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "scrypto::resource::Bucket",
          "generics": []
        }
      }
    ]
  }
"# }


#[derive(NftData)]
pub struct LotteryCard {
    lottery_name: String,
    lottery_epoch: u64
}

blueprint! {
    struct Lottery {
        // staking component
        staking: Staking,
        // admin and minting badges
        admin_badge: ResourceDef,
        // vault for minting tickets
        ticket_minter: Vault,
        // address of staking contract
        staking_address: Address,
        // staking user badge
        staking_user: Vault,
        // vault that will be used for withdrawals
        withdraw_vault: Vault,
        // resource def for future minting
        tickets: ResourceDef,
        // players staking amount
        players: HashMap<Address, HashMap<String, Decimal>>,
        // IDs counter
        ticket_id_counter: u128,
        // minted nft ids
        minted: Vec<u128>,
        // fixed price per lottery ticket
        ticket_price: Decimal,
        // current lottery details
        lottery_name: String,
        lottery_epoch: u64,
        // history of lotteries
        ended:HashMap<String, (u128, Vault)>,
    }

    impl Lottery {
        
        pub fn new(staking_address: Address,price: Decimal) -> (Component, Bucket) {
           
            // main badges for admin and minting
            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
            .metadata("name", "Lottery Badge Mint Auth")
            .initial_supply_fungible(1);

            let ticket_minter = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
            .metadata("name", "Lottery Ticket Minter")
            .initial_supply_fungible(1);

            let admin_resource_def = admin_badge.resource_def();

            // token resource def
            let tickets = ResourceBuilder::new_non_fungible()
            .metadata("name", "Lottery Ticket Token")
            .metadata("symbol", "LTT")
            .flags(MINTABLE | BURNABLE)
            .badge(ticket_minter.resource_def(), MAY_MINT | MAY_BURN)
            .no_initial_supply();

            // staking setup
            let staking: Staking = staking_address.into();
            // staking user badge
            let staking_badge = staking.new_user();

            let component = Self {
                staking,
                admin_badge: admin_resource_def, 
                ticket_minter: Vault::with_bucket(ticket_minter),
                staking_user: Vault::with_bucket(staking_badge),
                staking_address,
                withdraw_vault: Vault::new(RADIX_TOKEN),
                players: HashMap::new(),
                ticket_id_counter: 0,
                tickets: tickets,
                minted: Vec::new(),
                ticket_price: price,
                lottery_epoch: 0,
                lottery_name: String::new(),
                ended: HashMap::new(),
            }
            .instantiate();
            
            (component, admin_badge)
        }

        #[auth(admin_badge)]
        pub fn start_lottery(&mut self, name: String, epoch: u64) {
            assert!(
                self.lottery_epoch == 0 || Context::current_epoch() > self.lottery_epoch, 
                "Cannot initiate new lottery when current one is not ended yet"
            );

            assert!(epoch > 0, "Epoch cannot be zero");
            assert!(!name.is_empty(), "Name cannot be empty");

            self.lottery_epoch = epoch;
            self.lottery_name = name
        }

        #[auth(admin_badge)]
        pub fn end_lottery(&mut self) {
            assert!(Context::current_epoch() > self.lottery_epoch, "Lottery is not ended yet");

            debug!("++++++Lottery Finished++++++");

            // withdraw staking
            self.staking_user.authorize(|auth| {
                let withdraw = self.staking.withdraw(auth);
                self.withdraw_vault.put(withdraw);
            });            

            // pick winner
            // take hash
            let hash = Context::transaction_hash().to_string();
            //take first 5 bits of result hash
            let seed = &hash[0..5];
            //convert 10 hex bits to usize
            let result = usize::from_str_radix(&seed, 16).unwrap();
            // get winner index
            let index = result % self.minted.iter().count();
            // get winner ticket id
            let winner = self.minted[index];
            debug!("Winner ID: {}", winner);
            // save result

            self.staking_user.authorize(|auth| {
                let reward = self.staking.get_reward(auth);
                debug!("Reward: {}", reward.amount());
                self.ended.insert(self.lottery_name.to_string(), (winner, Vault::with_bucket(reward)));
            });
                        
            // clear minted array
            self.minted.clear();
            debug!("=======================");
        }

        pub fn buy_ticket(&mut self, player: Address, payment: Bucket) -> (Bucket, Bucket) {
            assert!(Context::current_epoch() < self.lottery_epoch, "Lottery is ended");
            assert!(payment.resource_address() == RADIX_TOKEN, "You can only use radix");
            assert!(payment.amount() >= self.ticket_price, "Not enough amount");

            // mint ticket
            let data = LotteryCard {
                lottery_name: self.lottery_name.to_string(),
                lottery_epoch: self.lottery_epoch
            };

            let ticket = self.ticket_minter.authorize(|auth| {
                self.tickets.mint_nft(self.ticket_id_counter, data, auth)
            });

            self.ticket_id_counter += 1;

            // save minted ticket id for the lottery
            let id = ticket.get_nft_id();
            self.minted.push(id);
            debug!("Minted new ticket: {}", id);

            // send to stake
            self.staking_user.authorize(|auth| {
                self.staking.stake(auth, payment.take(self.ticket_price));
            });

            // save staking for a user
            let player_staking = self.players.entry(player).or_insert(HashMap::new());
            let staking_amount = player_staking.entry(self.lottery_name.to_string()).or_insert(Decimal::zero());
            *staking_amount += self.ticket_price;

            debug!("Total staked for this lottery: {}", staking_amount);

            (payment, ticket)
        }

        pub fn request_reward(&mut self, lottery_name: String, player: Address, ticket: Bucket) {
            assert!(self.ended.contains_key(&lottery_name), "This lottery is not found or finished yet");
            assert!(ticket.resource_def() == self.tickets, "This bucket doesn't match to lottery tickets");
            assert!(ticket.amount() == Decimal::one(), "Checking reward for one ticket at a time");

            // get results of ended lottery
            let result = self.ended.get(&lottery_name).unwrap();

            // compare winner with requested ticket
            if result.0 == ticket.get_nft_id() {
                debug!("Winning ticket!");
                // send the reward
                let reward = result.1.take_all();
                Account::from(player).deposit(reward);
            }else{
                debug!("Not Winning ticket!");
            }

            // burn tickets
            self.ticket_minter.authorize(|auth| {
                ticket.burn_with_auth(auth);
            });
        }

        pub fn withdraw(&mut self, lottery_name: String, player: Address) {
            assert!(self.ended.contains_key(&lottery_name), "This lottery is not found or finished yet");
            assert!(self.players.contains_key(&player), "No staking found for this user");

            // get all staking for this player
            let player_staking = self.players.get_mut(&player).unwrap();
            assert!(player_staking.contains_key(&lottery_name), "No staking found for the lottery");
            // get lottery specific staking
            let lottery_staking = player_staking.get_mut(&lottery_name).unwrap();
            // withdraw 
            let bucket = self.withdraw_vault.take(*lottery_staking);
            Account::from(player).deposit(bucket);
            // reset staking for this lottery
            *lottery_staking = Decimal::zero();
        }
    }
}
