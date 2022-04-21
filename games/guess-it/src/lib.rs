extern crate unescape;
use std::fmt;
use std::fmt::Formatter;
use scrypto::prelude::*;
use sbor::{Encode,Decode,TypeId, Describe};
use serde::{Serialize, Deserialize};
use unescape::unescape;
use std::cmp;
mod context;

#[derive(Debug, Copy, Clone, TypeId, Encode, Decode, Describe, PartialEq, Eq, Deserialize, Serialize)]
#[repr(u32)]
pub enum State {
    AcceptingChallenger = 0,
    MakeGuess = 1,
    WinnerSelection = 2,
    Payout = 3,
    Refunding = 4,
    Destroyed = 5,
}
impl FromStr for State {
    type Err = String;

    fn from_str(input: &str) -> Result<State, Self::Err> {
        match input {
            "AcceptingChallenger" => Ok(State::AcceptingChallenger),
            "MakeGuess" => Ok(State::MakeGuess),
            "WinnerSelection" => Ok(State::WinnerSelection),
            "Payout" => Ok(State::Payout),
            "Refunding" => Ok(State::Refunding),
            "Destroyed" => Ok(State::Destroyed),
            _ => Err("Could not parse state".to_string()),
        }
    }
}
impl From<u32> for State {

    fn from(input: u32) -> State {
        match input {
            0 => State::AcceptingChallenger,
            1 => State::MakeGuess,
            2 => State::WinnerSelection,
            3 => State::Payout,
            4 => State::Refunding,
            5 => State::Destroyed,
            _ => State::Refunding,
        }
    }
}
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let outcome = match *self {
            State::AcceptingChallenger => "AcceptingChallenger",
            State::MakeGuess => "MakeGuess",
            State::WinnerSelection => "WinnerSelection",
            State::Payout => "Payout",
            State::Refunding => "Refunding",
            State::Destroyed => "Destroyed",
        };
        write!(f, "{}", outcome)
    }
}

#[derive(TypeId, Decode, Encode)]
pub struct Game {
    name: String,
    state: State,
    bet: String,
    players: HashMap<NonFungibleKey, Player>,
    winner: NonFungibleKey,
    last_roll: u128,
}
impl fmt::Debug for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let formatted = str::replace(&*format!("{:#}", Game::serialize(&self).as_str()), "\\", "");
        write!(f, "{}", formatted)
    }
}
impl Game {
    pub fn new(players: Option<HashMap<NonFungibleKey, Player>>, name: String, bet: String) -> Self {
        Self {
            name,
            state: State::AcceptingChallenger,
            bet,
            players: players.unwrap_or(HashMap::new()),
            winner: NonFungibleKey::from(0),
            last_roll: 0u128,
        }
    }

    pub fn get_random(&self) -> u128 {
        let multiplier = self.players.clone().into_iter()
            .map(|(_,p)| p.guess)
            .reduce(|a,b| a * b).unwrap_or(1);

        Uuid::generate() * multiplier
    }

    fn get_winner(&self, random_number: u128) -> NonFungibleKey {
        let player_ids: Vec<NonFungibleKey> = self.players.clone().into_iter().map(|(k,_)| k).collect();
        let p1 = random_number - self.players.get(&player_ids[0]).unwrap_or(&Player::empty()).guess;
        let p2 = random_number - self.players.get(&player_ids[1]).unwrap_or(&Player::empty()).guess;
        let closest_guess = cmp::min(p1, p2);
        let winner = if p1 == closest_guess { &player_ids[0] } else { &player_ids[1] };

        return winner.to_owned();
    }

    pub fn serialize(game: &Self) -> String {
        let players: HashMap<String, Player> = game.players.clone().into_iter()
            .map(|(k, p)| (k.to_string(), p))
            .collect();

        let serializable = GameSerialized {
            name: game.name.clone(),
            state: game.state.clone(),
            players,
            winner: game.winner.to_string(),
            bet: game.bet.clone(),
            last_roll: game.last_roll.clone(),
        };
        unescape(serde_json::to_string(&serializable).unwrap_or("".to_string())
            .as_mut_str())
            .unwrap_or("".to_string())
    }
}

#[derive(TypeId, Decode, Encode, Clone, Deserialize, Serialize)]
pub struct GameSerialized {
    pub name: String,
    pub state: State,
    pub bet: String,
    pub players: HashMap<String, Player>,
    pub winner: String,
    pub last_roll: u128,
}
impl fmt::Debug for GameSerialized {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let formatted = serde_json::to_string(&self).unwrap().replace("\\", "");
        write!(f, "{}", formatted)
    }
}
impl GameSerialized {
    pub fn empty(players: Option<HashMap<String, Player>>) -> Self {
        Self {
            name: "".to_string(),
            state: State::AcceptingChallenger,
            bet: 0.to_string(),
            players: players.unwrap_or(HashMap::new()),
            winner: NonFungibleKey::from(0).to_string(),
            last_roll: 0u128,
        }
    }
}

#[derive(Debug, TypeId, Describe, Decode, Encode, Clone, Serialize, Deserialize)]
pub struct Player {
    guess: u128,
}
impl Player {
    pub fn empty() -> Player {
        return Self {
            guess: 0u128,
        }
    }

    pub fn get_guess(&self) -> u128 { self.guess }
}

#[derive(TypeId, Describe, Decode, Encode)]
#[derive(NonFungibleData)]
struct PlayerNft {
    name: String,
}

blueprint! {
    struct GuessIt {
        bank: Vault,
        badges: Vault,
        badge_ref: ResourceDef,
        game: Game,
    }

    impl GuessIt {
        pub fn create(name: String, bet: String) -> Component {
            let player_badges: Bucket = ResourceBuilder::new_non_fungible()
                .metadata("name", "Guess-It Game")
                .metadata("symbol", "GIG")
                .initial_supply_non_fungible([
                    (
                        NonFungibleKey::from(Uuid::generate()),
                        PlayerNft {name: "Player 1".to_string()},
                    ),
                    (
                        NonFungibleKey::from(Uuid::generate()),
                        PlayerNft {name: "Player 2".to_string()},
                    ),
                ]);
            let player_badge_def = player_badges.resource_def();

            Self {
                bank: Vault::new(RADIX_TOKEN),
                badges: Vault::with_bucket(player_badges),
                badge_ref: player_badge_def,
                game: Game::new(None, name, bet),
            }.instantiate()
        }

        pub fn join(&mut self, mut payment: Bucket) -> (Bucket, Bucket, String) {
            assert_eq!(self.game.state, State::AcceptingChallenger, "Not accepting challengers!");
            assert!(self.game.players.len() <= 2, "Too many players!");

            let player: Player = Player::empty();
            let badge: Bucket = self.badges.take(Decimal::one());
            let nft_id: NonFungibleKey = badge.get_non_fungible_key();
            // Secure the funds
            self.bank.put(payment.take(Decimal::from(self.game.bet.clone())));
            // Add the player's details
            self.game.players.insert(nft_id.clone(), player.clone());
            self.game.state = if self.game.players.len() == 2 { State::MakeGuess } else { State::AcceptingChallenger };

            return (badge, payment, nft_id.to_string());
        }

        pub fn state(&mut self) -> String {
            Game::serialize(&self.game)
        }

        #[auth(badge_ref)]
        pub fn make_guess(&mut self, guess: u128) -> String {
            assert_eq!(self.game.state, State::MakeGuess, "Not ready to make a guess!");

            let key = auth.get_non_fungible_key();
            let player = self.game.players.get_mut(&key).unwrap();
            player.guess = guess;

            let guesses: Vec<u128> = self.game.players.clone().into_iter()
                .map(|(_key, player)| player.guess)
                .filter(|guess| guess > &0u128)
                .collect();
            self.game.state = if guesses.len() == 2 { State::WinnerSelection } else { State::MakeGuess };
            if self.game.state == State::WinnerSelection {
                let response = self.check_winner();
                if self.game.winner != NonFungibleKey::from(0) {
                    self.game.state = State::Payout;
                }
                format!("Your guess '{}' was accepted\nResponse: {}", guess, response)
            } else {
                format!("Your guess '{}' was accepted", guess)
            }

        }

        pub fn check_winner(&mut self) -> String {
            // Pick random number and store it
            let random_number = (self.game.get_random() % 6) + 1;
            // determine who was closest without going over
            self.game.winner = self.game.get_winner(random_number);
            self.game.last_roll = random_number;

            format!("Dice roll was: '{}' .... winner is: {}",random_number, self.game.winner)
        }

        #[auth(badge_ref)]
        pub fn withdraw_funds(&mut self) -> (Bucket, String) {
            assert_eq!(self.game.state, State::Payout, "It's not time to get paid!");
            let nft_id: NonFungibleKey = auth.get_non_fungible_key();

            if nft_id == self.game.winner {
                let amt = self.bank.amount();
                let payout = self.bank.take(amt);

                (payout, format!("You withdrew {:?} XRD!", amt))
            }
            else { panic!("You cannot withdraw funds!") }
        }

        pub fn get_context(&self) -> (Actor, Address, H256, u64, u128) {
            context::ContextTest::query()
        }
    }
}