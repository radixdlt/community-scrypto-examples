use scrypto::prelude::*;
use sbor::*;

#[derive(TypeId, Encode, Decode, Describe)]
struct GameData {
    game_id: U64,
    prize_pool: Vault,
    initiator: Address,
    winning_player: Option<Address>,
    fee_to_join: Decimal,
    minimum_players: U64,
    epoch_game_length: U64,
    epoch_of_game_start: U64,
    players: Vec<Address>,
    has_started: bool,
    has_finished: bool,
    current_player_amount: U64,
}

#[blueprint]
mod game {
    struct KingOfTheHill {
        number_of_games: U64,
        games: Vec<GameData>,
    }

    impl KingOfTheHill {
        pub fn instantiate_contract() -> (ComponentAddress, Bucket) {
            let game_contract_owner: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "KOTH Game Contract Owner Badge")
                .mint_initial_supply(1);

            let games: Vec<GameData> = Vec::new();

            let game = Self {
                number_of_games: 0,
                games: games,
            }
            .instantiate();
            
            (game.globalize(), game_contract_owner)
        }

        pub fn take_winner_spot(&mut self, game_id: U64) {
            assert!(
                self.games[game_id].has_started == true,
                "Cannot take a spot in a game that has not started"
            );
            assert!(
                self.games[game_id].has_finished == false,
                "Cannot take a spot in a game that has finished"
            );
            assert!(
                self.games[game_id].players.contains(&Runtime::actor()),
                "You are not in this game"
            );
            info!("Taking winner spot in game {} ", game_id);

            self.games[game_id].winning_player = Runtime::actor();

            let end_epoch =
                self.games[game_id].epoch_of_game_start + self.games[game_id].epoch_game_length;

            if Runtime::current_epoch() >= end_epoch {
                self.games[game_id].has_finished = true;
            }
        }

        pub fn withdraw_winnings(&mut self, game_id: U64) -> Bucket {
            assert!(
                self.games[game_id].has_started == true,
                "Cannot withdraw winnings from a game that has not started"
            );
            assert!(
                self.games[game_id].has_finished == true,
                "Cannot withdraw winnings from a game that has not finished"
            );
            assert!(
                self.games[game_id].winning_player == Runtime::actor(),
                "You are not the winner of this game"
            );
            info!("Withdrawing winnings from game {} ", game_id);

            self.games[game_id]
                .prize_pool
                .take(self.games[game_id].fee_to_join)
        }

        pub fn start_game(&mut self, game_id: U64) {
            assert!(
                self.games[game_id].has_started == false,
                "Cannot start a game that has already started"
            );
            assert!(
                self.games[game_id].has_finished == false,
                "Cannot start a game that has finished"
            );
            assert!(
                self.games[game_id].players.len() >= self.games[game_id].minimum_players,
                "Not enough players to start the game"
            );
            info!("Starting game {} ", game_id);

            self.games[game_id].has_started = true;
            self.games[game_id].epoch_of_game_start = Runtime::current_epoch();
        }

        pub fn join_game(&mut self, fee_to_join: Bucket, game_id: U64) {
            assert!(
                self.games[game_id].has_started == false,
                "Cannot join a game that has started"
            );
            assert!(
                self.games[game_id].prize_pool.resource_address() == fee_to_join.resource_address(),
                "You must pay the entry fee in the correct currency"
            );
            assert!(
                self.games[game_id].fee_to_join <= fee_to_join.amount(),
                "Not enough money to join the game"
            );

            info!("Joining game {} ", game_id);
            self.games[game_id].players.push(Runtime::actor());
            self.games[game_id].prize_pool.deposit(fee_to_join);
        }

        pub fn leave_game(&mut self, game_id: U64) -> Bucket {
            assert!(
                self.games[game_id].has_started == false,
                "Cannot leave a game that has started"
            );
            assert!(
                self.games[game_id].has_finished == false,
                "Cannot leave a game that has finished"
            );
            assert!(
                self.games[game_id].players.contains(&Runtime::actor()),
                "You are not in this game"
            );
            info!("Leaving game {} ", game_id);

            let caller = Runtime::actor();
            self.games[game_id].players.remove(caller);
            self.games[game_id]
                .prize_pool
                .take(self.games[game_id].fee_to_join)
        }

        pub fn cancel_game(&mut self, game_id: U64) {
            assert!(
                self.games[game_id].initiator == Runtime::actor(),
                "Only the initiator can cancel the game"
            );
            assert!(
                self.games[game_id].has_started == false,
                "Cannot cancel a game that has started"
            );
            assert!(
                self.games[game_id].has_finished == false,
                "Cannot cancel a game that has finished"
            );
            info!("Fininshing game {} ", game_id);

            if self.games[game_id].players.len() > 0 {
                for player_address in self.games[game_id].players.iter() {
                    borrow_component!(*player_address).call::<()>(
                        "deposit",
                        args![self.games[game_id]
                            .prize_pool
                            .take(self.games[game_id].fee_to_join)],
                    )
                }
            }
            self.games[game_id].has_finished = true;
        }

        pub fn initialize_game(
            &mut self,
            fee_to_join: Decimal,
            minimum_players: U64,
            epoch_game_length: U64,
        ) {
            let players: Vec<Address> = Vec::new();

            let game = GameData {
                prize_pool: Vault::new(RADIX_TOKEN),
                game_id: self.number_of_games,
                initiator: Runtime::actor().component_address().unwrap(),
                fee_to_join: fee_to_join,
                minimum_players: minimum_players,
                players: players,
                winning_player: None,
                has_started: false,
                has_finished: false,
                current_player_amount: 0,
                epoch_of_game_start: 0,
                epoch_game_length: epoch_game_length,
            };

            self.games.push(game);
            self.number_of_games += 1;
        }
    }
}
