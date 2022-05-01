extern crate unescape;
use std::fmt;
use std::fmt::Formatter;
use scrypto::prelude::*;
use sbor::{Encode,Decode,TypeId, Describe};
use serde::{Serialize, Deserialize};
use unescape::unescape;
use std::cmp;

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

#[derive(TypeId, Decode, Encode, Describe)]
pub struct Game {
    name: String,
    state: State,
    bet: i128,
    players: HashMap<NonFungibleId, Player>,
    winner: NonFungibleId,
    last_roll: u128,
}
impl fmt::Debug for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let formatted = str::replace(&*format!("{:#}", Game::serialize(&self).as_str()), "\\", "");
        write!(f, "{}", formatted)
    }
}
impl Game {
    pub fn new(players: Option<HashMap<NonFungibleId, Player>>, name: String, bet: u128) -> Self {
        Self {
            name,
            state: State::AcceptingChallenger,
            bet: bet as i128,
            players: players.unwrap_or(HashMap::new()),
            winner: NonFungibleId::from_u64(0),
            last_roll: 0u128,
        }
    }

    // This is a pseudorandom function and not a true random number function.
    pub fn get_random(&self) -> u128 {
        let multiplier = self.players.clone().into_iter()
            .map(|(_,p)| p.guess)
            .reduce(|a,b| a * b).unwrap_or(1);

        Runtime::generate_uuid() / multiplier
    }

    pub fn get_winner(&self, random_number: u128) -> NonFungibleId {
        let player_ids: Vec<NonFungibleId> = self.players.clone().into_iter().map(|(k,_)| k).collect();
        let p1 = random_number - self.players.get(&player_ids[0]).unwrap_or(&Player::empty()).guess;
        let p2 = random_number - self.players.get(&player_ids[1]).unwrap_or(&Player::empty()).guess;
        let closest_guess = cmp::min(p1, p2);

        // Deal with a tie
        if p1 == p2 { return NonFungibleId::from_u64(0); }
        let winner = if p1 == closest_guess { &player_ids[0] } else { &player_ids[1] };

        return winner.to_owned();
    }

    pub fn reset_game(&mut self) {
        self.players.values_mut().for_each(|p| p.guess = 0u128 );
        self.state = State::MakeGuess;
    }

    pub fn check_winner(&mut self) {
        assert_eq!(self.state, State::WinnerSelection, "Not time to choose winner!");
        // Pick random number and store it
        let random_number = (self.get_random() % 6) + 1;
        // determine who was closest without going over
        self.winner = self.get_winner(random_number);
        self.last_roll = random_number;

        self.update_state();
        if self.state == State::WinnerSelection { self.reset_game(); }
    }

    pub fn has_guessed(&self) -> bool {
        let guesses: Vec<u128> = self.players.clone().into_iter()
            .map(|(_key, player)| player.guess)
            .filter(|guess| guess > &0u128)
            .collect();

        guesses.len() == 2
    }

    pub fn update_state(&mut self) {
        match self.state {
            State::AcceptingChallenger => if self.players.len() == 2 { self.state = State::MakeGuess; },
            State::MakeGuess => if self.has_guessed() { self.state = State::WinnerSelection; self.check_winner(); },
            State::WinnerSelection => if self.winner != NonFungibleId::from_u64(0) { self.state = State::Payout; },
            State::Payout => { self.state = State::Destroyed; }
            _ => ()
        }
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
            bet: game.bet,
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
    pub bet: i128,
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
            bet: 0i128,
            players: players.unwrap_or(HashMap::new()),
            winner: NonFungibleId::from_u64(0).to_string(),
            last_roll: 0u128,
        }
    }
}

#[derive(Debug, TypeId, Describe, Decode, Encode, Clone, Serialize, Deserialize)]
pub struct Player {
    pub guess: u128,
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
        badge_address: ResourceAddress,
        game: Game,
    }

    impl GuessIt {
        pub fn create(name: String, bet: u128) -> ComponentAddress {
            let player_badges: Bucket = ResourceBuilder::new_non_fungible()
                .metadata("name", "Guess-It Game")
                .metadata("symbol", "GIG")
                .initial_supply([
                    (
                        NonFungibleId::random(),
                        PlayerNft {name: "Player 1".to_string()},
                    ),
                    (
                        NonFungibleId::random(),
                        PlayerNft {name: "Player 2".to_string()},
                    ),
                ]);
            let player_badge_address = player_badges.resource_address();

            Self {
                bank: Vault::new(RADIX_TOKEN),
                badges: Vault::with_bucket(player_badges),
                badge_address: player_badge_address,
                game: Game::new(None, name, bet),
            }
            .instantiate()
            .globalize()
        }

        pub fn join(&mut self, mut payment: Bucket) -> (Bucket, Bucket, String) {
            assert_eq!(self.game.state, State::AcceptingChallenger, "Not accepting challengers!");
            assert!(self.game.players.len() <= 2, "Too many players!");

            let player: Player = Player::empty();
            let badge: Bucket = self.badges.take(Decimal::one());
            let nft_id: NonFungibleId = badge.non_fungible::<PlayerNft>().id();
            // Secure the funds
            self.bank.put(payment.take(Decimal::from(self.game.bet)));
            // Add the player's details
            self.game.players.insert(nft_id.clone(), player);
            self.game.update_state();

            return (badge, payment, nft_id.to_string());
        }

        pub fn state(&mut self) -> String {
            Game::serialize(&self.game)
        }

        pub fn make_guess(&mut self, guess: u128, auth: Proof) -> String {
            assert_eq!(auth.resource_address(), self.badge_address, "Invalid badge provided");
            assert_eq!(auth.amount(), dec!("1"), "Invalid badge amount provided");

            assert_eq!(self.game.state, State::MakeGuess, "Not ready to make a guess!");
            assert!(guess <= 6 && guess >= 1, "You can only guess 1-6!");

            let key = auth.non_fungible::<PlayerNft>().id();
            let player = self.game.players.get_mut(&key).unwrap();
            player.guess = guess;

            self.game.update_state();
            format!("Your guess '{}' was accepted", guess)
        }

        pub fn withdraw_funds(&mut self, auth: Proof) -> (Bucket, String) {
            assert_eq!(auth.resource_address(), self.badge_address, "Invalid badge provided");
            assert_eq!(auth.amount(), dec!("1"), "Invalid badge amount provided");
            assert_eq!(self.game.state, State::Payout, "It's not time to get paid!");

            let nft_id: NonFungibleId = auth.non_fungible::<PlayerNft>().id();
            assert!(nft_id == self.game.winner, "You cannot withdraw funds!");

            let amt = self.bank.amount();
            let payout = self.bank.take(amt);

            self.game.update_state();
            (payout, format!("You withdrew {:?} XRD!", amt))
        }
    }
}