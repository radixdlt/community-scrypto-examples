use scrypto::prelude::*;

import! { r#"
{
  "package": "013fa22e238526e9c82376d2b4679a845364243bf970e5f783d13f",
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
      "mutability": "Mutable",
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
          "name": "scrypto::resource::Bucket",
          "generics": []
        },
        {
          "type": "Custom",
          "name": "scrypto::resource::BucketRef",
          "generics": []
        }
      ],
      "output": {
        "type": "Unit"
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
    }
  ]
}
"# }


#[derive(NonFungibleData)]
pub struct LotteryTicketData {
    // holds lottery ID this ticket belongs to
    lottery_id: u128,
    //use to mark a ticket after the withdraw
    #[scrypto(mutable)]
    checked: bool, 
    //use to mark a winner ticket
    #[scrypto(mutable)]
    winner: bool
}

#[derive(NonFungibleData)]
pub struct LotteryData {
    // fixed price per lottery ticket
    ticket_price: Decimal,
    // current lottery details
    lottery_name: String,
    lottery_epoch: u64,
    // mutable minted tickets array
    #[scrypto(mutable)]
    minted: Vec<NonFungibleKey>,
    // ended flag
    #[scrypto(mutable)]
    ended: bool,
    // reward value after lottery end
    #[scrypto(mutable)]
    reward: Decimal,
    // winner ticket id after lottery end
    #[scrypto(mutable)]
    winner: Option<NonFungibleKey>
}

blueprint! {
    struct Lottery {
        // staking component
        staking: Staking,
        // admin and minting badges
        admin_badge: ResourceDef,
        // vault for minting tickets
        lottery_minter: Vault,
        // address of staking contract
        staking_address: Address,
        // staking user NFT
        staking_token: Vault,
        // track staking amount
        current_staking: Decimal,
        // vault that will be used for withdrawals and reward
        assets_vault: Vault,
        // resource def for minting lottery tickets
        tickets_resource_def: ResourceDef,
        // resource def for the lottery
        lottery_resource_def: ResourceDef,
        // ID counters
        lottery_id_counter: u128,
        ticket_id_counter: u128,
    }

    impl Lottery {
        
      // Initiate new Lottery using address for the staking component
        pub fn new(staking_address: Address) -> (Component, Bucket) {
           
            // main badges for admin and minting
            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
            .metadata("name", "Lottery Admin Badge")
            .initial_supply_fungible(1);

            let lottery_minter = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
            .metadata("name", "Lottery Ticket Minter")
            .initial_supply_fungible(1);

            let admin_resource_def = admin_badge.resource_def();

            // lottery nft
            let lottery_resource_def = ResourceBuilder::new_non_fungible()
            .metadata("name", "Lottery Token")
            .metadata("symbol", "LOT")
            .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
            .badge(lottery_minter.resource_def(), MAY_MINT | MAY_CHANGE_INDIVIDUAL_METADATA)
            .no_initial_supply();

            // lottery ticket nft
            let tickets_resource_def = ResourceBuilder::new_non_fungible()
            .metadata("name", "Lottery Ticket Token")
            .metadata("symbol", "LTT")
            .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
            .badge(lottery_minter.resource_def(), MAY_MINT| MAY_CHANGE_INDIVIDUAL_METADATA)
            .no_initial_supply();

            // staking setup
            let staking: Staking = staking_address.into();
            // register new user and take the staking user nft (STT)
            let staking_nft = staking.new_user();

            let component = Self {
                staking,
                admin_badge: admin_resource_def, 
                lottery_minter: Vault::with_bucket(lottery_minter),
                staking_token: Vault::with_bucket(staking_nft),
                staking_address,
                current_staking: Decimal::zero(),
                assets_vault: Vault::new(RADIX_TOKEN),
                ticket_id_counter: 0,
                lottery_id_counter: 0,
                tickets_resource_def,
                lottery_resource_def
            }
            .instantiate();
            
            (component, admin_badge)
        }

        // initiate new lottery
        #[auth(admin_badge)]
        pub fn start_lottery(&mut self, name: String, epoch: u64, price: Decimal) -> Bucket {
            assert!(epoch > 0, "Epoch cannot be zero");
            assert!(!name.is_empty(), "Name cannot be empty");

            // mint lottery
            let data = LotteryData {
              lottery_name: name,
              lottery_epoch: epoch,
              ended: false,
              minted: Vec::new(),
              ticket_price:price,
              reward: Decimal::zero(),
              winner: None
          };

          let lottery = self.lottery_minter.authorize(|auth| {
              self.lottery_resource_def.mint_non_fungible(&NonFungibleKey::from(self.lottery_id_counter), data, auth)
          });

          // increase lottery count
          self.lottery_id_counter += 1;

          lottery
        }

        // End lottery by specific ID
        #[auth(admin_badge)]
        pub fn end_lottery(&mut self, lottery_id: u128) {

            // find this lottery
            let mut lottery_data: LotteryData = self.lottery_resource_def.get_non_fungible_data(&NonFungibleKey::from(lottery_id));
            assert!(Context::current_epoch() > lottery_data.lottery_epoch, "Lottery epoch is not ended yet");

            debug!("++++++Lottery Finished++++++");

            // pick winner
            // take hash
            let hash = Context::transaction_hash().to_string();
            //take first 5 bits of result hash
            let seed = &hash[0..5];
            //convert 10 hex bits to usize
            let result = usize::from_str_radix(&seed, 16).unwrap();
            // get winner index
            let index = result % lottery_data.minted.iter().count();
            // get winner ticket id
            let winner = lottery_data.minted.get(index).expect("Error while fetching winner");
            debug!("Winner ID: {:?}", winner);

            // save winner
            lottery_data.winner = Some(winner.clone());
            lottery_data.ended = true;

            // withdraw staking
            // get current staking amount
            let staking_amount = self.current_staking;            
            self.staking_token.authorize(|auth| {
                let withdraw = self.staking.withdraw(auth);

                // take the reward from the withdraw amount
                let reward = withdraw.amount() - staking_amount;
                debug!("Reward: {}", reward);

                // update lottery data reward params
                lottery_data.reward = reward;

                // clear staking value
                self.current_staking = Decimal::zero();
                self.assets_vault.put(withdraw);
            });            
                        
            // update lottery NFT
            self.lottery_minter.authorize(|auth| {
                self.lottery_resource_def.update_non_fungible_data(&NonFungibleKey::from(lottery_id), lottery_data, auth);
            });

            // clear current staking
            debug!("=======================");
        }

        // Buy a ticket for a specific lottery
        pub fn buy_ticket(&mut self, lottery_id: u128, mut payment: Bucket) -> (Bucket, Bucket) {
          // find this lottery
          let mut lottery_data: LotteryData = self.lottery_resource_def.get_non_fungible_data(&NonFungibleKey::from(lottery_id));
          assert!(!lottery_data.ended, "Lottery is ended");

          assert!(payment.resource_address() == RADIX_TOKEN, "You can only use radix");
          assert!(payment.amount() >= lottery_data.ticket_price, "Not enough amount");

          // send to stake
          self.staking_token.authorize(|auth| {
              self.staking.stake(payment.take(lottery_data.ticket_price), auth);
          });

          // update staking amount
          self.current_staking += lottery_data.ticket_price;

          // mint ticket
          let data = LotteryTicketData {
              lottery_id,
              checked: false,
              winner: false
          };

          let ticket = self.lottery_minter.authorize(|auth| {
              self.tickets_resource_def.mint_non_fungible(&NonFungibleKey::from(self.ticket_id_counter), data, auth)
          });

          self.ticket_id_counter += 1;

          // save minted ticket id for the lottery
          let id = ticket.get_non_fungible_key();
          debug!("Minted new ticket: {}", id);
          lottery_data.minted.push(id);

          // update lottery NFT
          self.lottery_minter.authorize(|auth| {
            self.lottery_resource_def.update_non_fungible_data(&NonFungibleKey::from(lottery_id), lottery_data, auth);
          });

          (payment, ticket)
        }

        // Withdraw the stake amount (purchased ticket price) and reward (in case of winning) for the lottery
        #[auth(tickets_resource_def)]
        pub fn withdraw(&mut self, lottery_id: u128) -> Bucket {
          assert!(auth.amount() == Decimal::one(), "Only one ticket withdraw at the time");

          // check lottery
          let lottery_data: LotteryData = self.lottery_resource_def.get_non_fungible_data(&NonFungibleKey::from(lottery_id));
          assert!(lottery_data.ended, "Lottery is not ended yet");

          // check the ticket
          let ticket_id = auth.get_non_fungible_key();
          let mut ticket_data: LotteryTicketData = self.tickets_resource_def.get_non_fungible_data(&NonFungibleKey::from(lottery_id));
          assert!(ticket_data.lottery_id == lottery_id, "The ticket doesn't belong to the specified lottery");
          assert!(!ticket_data.checked, "This ticket was already checked for the withdraw");

          // mark this ticket as checked
          ticket_data.checked = true;

          // get the staking amount
          let mut withdraw = self.assets_vault.take(lottery_data.ticket_price);

          // check if this ticket is winner
          match lottery_data.winner {
            Some(winner) => {
              if winner == ticket_id {
                // we found the winner, add the reward to the withdraw
                debug!("This is the winner ticket");
                // mark this ticket as a winner just for the user
                ticket_data.winner = true;
                // add reward to the withdraw
                withdraw.put(self.assets_vault.take(lottery_data.reward));
              }
            },
            _ => (),
          }

          // update ticket data
          self.lottery_minter.authorize(|minter_auth| {
            self.tickets_resource_def.update_non_fungible_data(&NonFungibleKey::from(lottery_id), ticket_data, minter_auth);
          });

          withdraw
        }
    }
}
