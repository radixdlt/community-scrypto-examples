use scrypto::prelude::*;

/* 
This is an implementation of the well known rock, paper, scissors game
The script caller plays agains the component a.k.a the house
Along with delivering the players move, a XRD amount that serves as bet must be deposited 
*/

#[blueprint]
mod rockpaperscissors {
    struct RPS {
        //admin badge to authorize burning and minting of tokens
        admin_badge: Vault,
        //resource representing 'rocks'
        rock: Vault,
        //resource representing 'paper'
        paper: Vault,
        //resource representing 'scissors'
        scissor: Vault,
        //XRD safe 
        xrd_vault: Vault
    }

    impl RPS {
        //this function instantiates the component
        //in addition to the function call, please provide a seed amount of XRD for the house
        pub fn instantiate_rps(xrd_seed: Bucket) -> ComponentAddress {
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Admin badge")
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);

            let rock: ResourceAddress = ResourceBuilder::new_fungible()
                .metadata("name", "Rock")
                .metadata("symbol", "RPS")
                .divisibility(DIVISIBILITY_NONE)
                .mintable(rule!(allow_all), LOCKED,)
                .burnable(rule!(require(admin_badge.resource_address())), LOCKED)
                .create_with_no_initial_supply();

            let paper: ResourceAddress = ResourceBuilder::new_fungible()
                .metadata("name", "Paper")
                .metadata("symbol", "RPS")
                .divisibility(DIVISIBILITY_NONE)
                .mintable(rule!(allow_all), LOCKED,)
                .burnable(rule!(require(admin_badge.resource_address())), LOCKED)
                .create_with_no_initial_supply();
            
            let scissor: ResourceAddress = ResourceBuilder::new_fungible()
                .metadata("name", "Scissor")
                .metadata("symbol", "RPS")
                .divisibility(DIVISIBILITY_NONE)
                .mintable(rule!(allow_all), LOCKED,)
                .burnable(rule!(require(admin_badge.resource_address())), LOCKED)
                .create_with_no_initial_supply();

            Self {
                admin_badge: Vault::with_bucket(admin_badge),
                rock: Vault::new(rock),
                paper: Vault::new(paper),
                scissor: Vault::new(scissor),
                xrd_vault: Vault::with_bucket(xrd_seed)
            }
                .instantiate()
                .globalize()
        }

        //this method mints and distributes the game options (rock, paper or scissor) as resources to the caller
        pub fn get_tokens(&mut self) -> (Bucket, Bucket, Bucket) {
            let rock_seed: Bucket = borrow_resource_manager!(self.rock.resource_address()).mint(1);
            let paper_seed: Bucket = borrow_resource_manager!(self.paper.resource_address()).mint(1);
            let scissor_seed: Bucket = borrow_resource_manager!(self.scissor.resource_address()).mint(1);

            (rock_seed, paper_seed, scissor_seed)
        }

        //this method enables the game
        //please provide your move, plus a XRD amount as bet to this method
        pub fn play_rps(&mut self, rps: Bucket, mut xrd_stake: Bucket) -> (Bucket, Bucket) {

                // assessess whether the resources in range are provided
                assert!(
                    rps.resource_address() == self.rock.resource_address()
                    ||
                    rps.resource_address() == self.paper.resource_address()
                    ||
                    rps.resource_address() == self.scissor.resource_address()
                    , "Please provide either a rock, paper or scissor token"
                );

                // assesses if XRD is provided as bet
                assert!(xrd_stake.resource_address() == self.xrd_vault.resource_address(), 
                    "Please provide radix as stake"
                );

                // random number generator (0..3) that determines components move
                let opponent = spin_wheel();

                // translate random number to a move
                let opponent_rps: ResourceAddress = match opponent {
                    1 => self.rock.resource_address(),
                    2 => self.paper.resource_address(),
                    3 => self.scissor.resource_address(),
                    // in case of min or max value, players move is returned to force a tie
                    _ => rps.resource_address(),
                };

                // retrieve player and components move as stored in metadata of resource
                let player_move = borrow_resource_manager!(rps.resource_address()).get_metadata("name".to_string()).unwrap();
                let house_move = borrow_resource_manager!(opponent_rps).get_metadata("name".to_string()).unwrap();

                // declare players bet in XRD as stake
                let stake = xrd_stake.amount();

                // determine rules that result in a win, loss or tie for the player
                // and reflect this in the XRD bucket as provided by player
                let xrd_stake: Bucket = 
                    if 
                        (rps.resource_address() == self.rock.resource_address()
                        && opponent_rps == self.scissor.resource_address())
                        ||
                        (rps.resource_address() == self.scissor.resource_address()
                        && opponent_rps == self.paper.resource_address())                        
                        ||
                        (rps.resource_address() == self.paper.resource_address()
                        && opponent_rps == self.rock.resource_address())
                        { 
                        info!("You win! You played {}, and your opponent played {}."
                            , player_move
                            , house_move
                        );
                        // take players betted amount and return from component vault                     
                        xrd_stake.put(self.xrd_vault.take(stake))  ;
                        xrd_stake           
                    }
                    else if 
                        (rps.resource_address() == self.rock.resource_address()
                        && opponent_rps == self.paper.resource_address())
                        ||
                        (rps.resource_address() == self.paper.resource_address()
                        && opponent_rps == self.scissor.resource_address())                        
                        ||
                        (rps.resource_address() == self.scissor.resource_address()
                        && opponent_rps == self.rock.resource_address())
                    {
                        info!("You lose ... You played {}, and your opponent played {}."
                            , player_move
                            , house_move
                        );   
                        // take players bet and store in component vault                     
                        self.xrd_vault.put(xrd_stake.take(stake));
                        xrd_stake
                    }
                    else {
                        info!("Its a tie! You played {}, and your opponent played {}."
                            , player_move
                            , house_move
                        );   
                        // return XRD bet                      
                        xrd_stake
                    };
                   
                // return players rock/paper/scissor token plus the XRD bucket
                (rps, xrd_stake)
        } 
    }
}

// implement random number generator
pub fn spin_wheel() -> i32 {
    let item = Runtime::generate_uuid() % 3;
    dbg!("Generated number: {}", item);
    return i32::try_from(item).unwrap();
}