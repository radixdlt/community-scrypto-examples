extern crate unescape;
use std::fmt;
use std::fmt::Formatter;
use scrypto::prelude::*;
use sbor::{Encode,Decode,TypeId, Describe};
use serde::{Serialize, Deserialize};
use unescape::unescape;
mod context;

#[derive(Debug, Copy, Clone, TypeId, Encode, Decode, Describe, PartialEq, Eq, Deserialize, Serialize)]
#[repr(u32)]
pub enum State {
    AcceptingChallenger = 0,
    MakeGuess = 1,
    SubmitSecret = 2,
    WinnerSelection = 3,
    Payout = 4,
    Refunding = 5,
    Destroyed = 6,
}
impl FromStr for State {
    type Err = String;

    fn from_str(input: &str) -> Result<State, Self::Err> {
        match input {
            "AcceptingChallenger" => Ok(State::AcceptingChallenger),
            "MakeGuess" => Ok(State::MakeGuess),
            "SubmitSecret" => Ok(State::SubmitSecret),
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
            2 => State::SubmitSecret,
            3 => State::WinnerSelection,
            4 => State::Payout,
            5 => State::Refunding,
            6 => State::Destroyed,
            _ => State::Refunding,
        }
    }
}
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let outcome = match *self {
            State::AcceptingChallenger => "AcceptingChallenger",
            State::MakeGuess => "MakeGuess",
            State::SubmitSecret => "SubmitSecret",
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
    state: State,
    pub players: HashMap<NonFungibleKey, Player>,
    winner: Player,
}
impl fmt::Debug for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let formatted = str::replace(&*format!("{:#}", Game::serialize(&self).as_str()), "\\", "");
        write!(f, "{}", formatted)
    }
}
impl Game {
    pub fn new(players: Option<HashMap<NonFungibleKey, Player>>) -> Self {
        Self {
            state: State::AcceptingChallenger,
            players: players.unwrap_or(HashMap::new()),
            winner: Player::empty(),
        }
    }
    pub fn serialize(game: &Self) -> String {
        let players: HashMap<String, Player> = game.players.clone().into_iter()
            .map(|(k, p)| (k.to_string(), p))
            .collect();

        let serializable = GameSerialized { state: game.state.clone(), players, winner: game.winner.clone() };
        unescape(serde_json::to_string(&serializable).unwrap_or("".to_string())
            .as_mut_str())
            .unwrap_or("".to_string())
    }
}

#[derive(TypeId, Decode, Encode, Clone, Deserialize, Serialize)]
pub struct GameSerialized {
    pub state: State,
    pub players: HashMap<String, Player>,
    pub winner: Player,
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
            state: State::AcceptingChallenger,
            players: players.unwrap_or(HashMap::new()),
            winner: Player::empty(),
        }
    }
}

#[derive(Debug, TypeId, Describe, Decode, Encode, Clone, Serialize, Deserialize)]
pub struct Player {
    guess: String,
    secret: String,
    bet: u32,
    share_of_pot: u32,
}
impl Player {
    pub fn empty() -> Player {
        return Self {
            guess: "".into(),
            secret: "".into(),
            bet: 0,
            share_of_pot: 0,
        }
    }

    pub fn get_guess(&self) -> String { self.guess.clone() }
    pub fn make_guess(&mut self, guess: String) { self.guess = guess; }

    pub fn get_secret(&self) -> String { self.secret.clone() }
}

#[derive(TypeId, Describe, Decode, Encode)]
#[derive(NonFungibleData)]
struct PlayerNft {
    name: String,
}

blueprint! {
    struct GuessIt {
        badges: Vault,
        badge_ref: ResourceDef,
        game: Game,
    }

    impl GuessIt {
        pub fn create() -> Component {
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
                badges: Vault::with_bucket(player_badges),
                badge_ref: player_badge_def,
                game: Game::new(None),
            }.instantiate()
        }

        pub fn join(&mut self) -> Bucket {
            assert_eq!(self.game.state, State::AcceptingChallenger, "Not accepting challengers!");
            assert!(self.game.players.len() <= 2, "Too many players!");

            let badge = self.badges.take(1);
            let player = Player::empty();
            let auth = badge.present();
            let key = auth.get_non_fungible_key().clone();
            auth.drop();

            self.game.players.insert(key, player.clone());
            self.game.state = if self.game.players.len() == 2 { State::MakeGuess } else { State::AcceptingChallenger };

            return badge;
        }

        pub fn state(&mut self) -> String {
            Game::serialize(&self.game)
        }

        #[auth(badge_ref)]
        pub fn make_guess(&mut self, guess: String) -> Result<String, ()> {
            assert_eq!(self.game.state, State::MakeGuess, "Not ready to make a guess!");

            let key = auth.get_non_fungible_key();
            let player = self.game.players.get_mut(&key).unwrap();
            player.make_guess(guess.clone());

            let guesses: Vec<String> = self.game.players.clone().into_iter()
                .map(|(_key, player)| player.get_guess())
                .filter(|guess| !guess.is_empty())
                .collect();
            self.game.state = if guesses.len() == 2 { State::WinnerSelection } else { State::MakeGuess };
            if self.game.state == State::WinnerSelection { self.check_winner(); }
            // if self.game.state == State::Payout { self.payout()?; }

            Ok(format!("Your guess '{}' was accepted", guess))
        }

        pub fn check_winner(&mut self) -> String {
            // Pick random number and store it

            // determine who was closest without going over

            // update player's share of pot

            format!("Winner is '{:#?}'", self.game.winner).to_string()
        }

        fn _payout(&mut self) -> Result<String, ()> {
            todo!()
            // Ok(format!("Doing Payout'").to_string())
        }

        #[auth(badge_ref)]
        fn withdraw(&mut self) -> Result<String, ()> {
            todo!()
            // Ok(format!("Doing Payout'").to_string())
        }

        pub fn get_context(&self) -> (Actor, Address, H256, u64, u128) {
            context::ContextTest::query()
        }
    }
}

fn _has_revealed(players_map: &HashMap<NonFungibleKey, Player>) -> bool {
    let players: Vec<Player> = players_map.clone().into_values().collect();
    if players.len() == 0 { return false; }
    let p1: Player = players.get(0).unwrap().to_owned();
    let p2: Player = players.get(1).unwrap().to_owned();

    !p1.get_secret().is_empty() && !p2.get_secret().is_empty()
}