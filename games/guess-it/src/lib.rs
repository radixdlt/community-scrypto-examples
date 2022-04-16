extern crate unescape;

use std::fmt;
use std::fmt::Formatter;
use scrypto::prelude::*;
use sbor::{Encode,Decode,TypeId, Describe};
use serde::{Serialize, Deserialize};
use serde_json::to_string;
use unescape::unescape;

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
}

#[derive(TypeId, Decode, Encode, Clone, Deserialize, Serialize)]
pub struct GameSerialized {
    pub players: HashMap<String, Player>
}
impl fmt::Debug for GameSerialized {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let formatted = serde_json::to_string(&self).unwrap().replace("\\", "");
        write!(f, "{}", formatted)
    }
}
impl GameSerialized {
    pub fn empty(players: Option<HashMap<String, Player>>) -> Self {
        Self { players: players.unwrap_or(HashMap::new()) }
    }
}

#[derive(TypeId, Decode, Encode)]
pub struct Game {
    pub players: HashMap<NonFungibleKey, Player>,
}
impl fmt::Debug for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let formatted = str::replace(&*format!("{:#}", Game::serialize(&self).as_str()), "\\", "");
        write!(f, "{}", formatted)
    }
}
impl Game {
    pub fn new(players: Option<HashMap<NonFungibleKey, Player>>) -> Self {
        Self { players: players.unwrap_or(HashMap::new()) }
    }
    pub fn serialize(game: &Self) -> String {
        let players: HashMap<String, Player> = game.players.clone().into_iter()
            .map(|(k, p)| (k.to_string(), p))
            .collect();

        let serializable = GameSerialized { players };
        unescape(serde_json::to_string(&serializable).unwrap_or("".to_string())
            .as_mut_str())
            .unwrap_or("".to_string())
    }
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
            let badge = self.badges.take(1);
            let player = Player::empty();
            let auth = badge.present();
            let key = auth.get_non_fungible_key().clone();
            auth.drop();

            self.game.players.insert(key, player.clone());
            // self.game.players.insert(NonFungibleKey::from(0), player.clone());
            return badge;
        }

        pub fn state(&mut self) -> String {
            Game::serialize(&self.game)
        }
    }
}