use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct PlayerBadge {
    game_id: usize,
    #[mutable]
    expired: bool,
}

#[derive(ScryptoSbor)]
struct GameData {
    game_id: usize,
    prize_pool: Vault,
    winning_player: NonFungibleLocalId,
    fee_to_join: Decimal,
    minimum_players: u64,
    current_player_amount: u64,
    epoch_game_length: u64,
    epoch_of_game_start: u64,
    has_started: bool,
    has_finished: bool,
    paid_out: bool,
}

#[blueprint]
mod game {
    struct KingOfTheHill {
        game_contract_badge: Vault,
        player_badge_address: ResourceAddress,
        games: Vec<GameData>,
        number_of_games: usize,
    }

    impl KingOfTheHill {
        pub fn instantiate_contract() -> ComponentAddress {
            let game_contract_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "KOTH Contract Badge")
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);

            let player_badge_address: ResourceAddress = ResourceBuilder::new_uuid_non_fungible::<PlayerBadge>()
                .metadata("name", "KOTH Player Badge")
                .updateable_non_fungible_data(rule!(require(game_contract_badge.resource_address())), rule!(deny_all))
                .create_with_no_initial_supply();

            let games: Vec<GameData> = Vec::new();

            Self {
                number_of_games: 0,
                games: games,
                player_badge_address: player_badge_address,
                game_contract_badge: Vault::with_bucket(game_contract_badge),
            }
            .instantiate()
            .globalize()
        }

        pub fn take_winner_spot(&mut self, game_id: usize, player_badge: Proof) {
            assert!(
                self.games[game_id].has_started == true,
                "Cannot take a spot in a game that has not started"
            );
            assert!(
                self.games[game_id].has_finished == false,
                "Cannot take a spot in a game that has finished"
            );

            let authorized_player = player_badge.validate_proof(self.player_badge_address).expect("Invalid NFT passed");
            let badge: NonFungible<PlayerBadge> = authorized_player.non_fungible();
            let badge_data: PlayerBadge = badge.data();

            assert!(
                badge_data.game_id == game_id,
                "You are not in this game"
            );
            assert!(
                badge_data.expired == false,
                "Your badge is expired"
            );

            let end_epoch =
                self.games[game_id].epoch_of_game_start + self.games[game_id].epoch_game_length;
            
            if Runtime::current_epoch() >= end_epoch {
                self.games[game_id].has_finished = true;
            }

            info!("Taking winner spot in game {} ", game_id);
            self.games[game_id].winning_player = badge.local_id().clone();
        }

        pub fn withdraw_winnings(&mut self, game_id: usize, player_badge: Proof) -> Bucket {
            assert!(
                self.games[game_id].paid_out == false,
                "Game has already been paid out"
            );
            assert!(
                self.games[game_id].has_started == true,
                "Cannot withdraw winnings from a game that has not started"
            );
            assert!(
                self.games[game_id].has_finished == true,
                "Cannot withdraw winnings from a game that has not finished"
            );

            let authorized_player = player_badge.validate_proof(self.player_badge_address).expect("Invalid NFT passed");
            let badge: NonFungible<PlayerBadge> = authorized_player.non_fungible();
            let badge_data: PlayerBadge = badge.data();

            assert!(
                badge_data.game_id == game_id,
                "You are not in this game"
            );
            assert!(
                badge_data.expired == false,
                "Your badge is expired"
            );
            assert!(
                badge.local_id() == &self.games[game_id].winning_player,
                "You did not win this game"
            );

            info!("Withdrawing winnings from game {} ", game_id);
            self.games[game_id].paid_out = true;
            self.games[game_id].prize_pool.take_all()
        }

        pub fn start_game(&mut self, game_id: usize, player_badge: Proof) {
            assert!(
                self.games[game_id].has_started == false,
                "Cannot start a game that has already started"
            );
            assert!(
                self.games[game_id].has_finished == false,
                "Cannot start a game that has finished"
            );
            assert!(
                self.games[game_id].current_player_amount >= self.games[game_id].minimum_players,
                "Not enough players to start the game"
            );

            let authorized_player = player_badge.validate_proof(self.player_badge_address).expect("Invalid NFT passed");
            let badge: NonFungible<PlayerBadge> = authorized_player.non_fungible();
            let badge_data: PlayerBadge = badge.data();

            assert!(
                badge_data.game_id == game_id,
                "You are not in this game"
            );
            assert!(
                badge_data.expired == false,
                "Your badge is expired"
            );
            info!("Starting game {} ", game_id);

            self.games[game_id].has_started = true;
            self.games[game_id].epoch_of_game_start = Runtime::current_epoch();
        }

        pub fn join_game(&mut self, fee_to_join: Bucket, game_id: usize) -> Bucket {
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
            self.games[game_id].prize_pool.put(fee_to_join);
            
            let issued_player_badge = self.game_contract_badge.authorize(|| {
                borrow_resource_manager!(self.player_badge_address).mint_uuid_non_fungible(
                    PlayerBadge {
                        game_id: game_id,
                        expired: false,
                })
            });

            self.games[game_id].current_player_amount += 1;
            info!("Issued badge for game: {} ", game_id);
            return issued_player_badge
        }

        pub fn leave_game(&mut self, game_id: usize, player_badge: Proof) -> Bucket {
            assert!(
                self.games[game_id].has_started == false,
                "Cannot leave a game that has started"
            );
            assert!(
                self.games[game_id].has_finished == false,
                "Cannot leave a game that has finished"
            );

            let authorized_player = player_badge.validate_proof(self.player_badge_address).expect("Invalid NFT passed");
            let badge: NonFungible<PlayerBadge> = authorized_player.non_fungible();
            let badge_data: PlayerBadge = badge.data();

            assert!(
                badge_data.game_id == game_id,
                "You are not in this game"
            );
            assert!(
                badge_data.expired == false,
                "You have already left this game"
            );


            self.game_contract_badge.authorize(|| {
                borrow_resource_manager!(self.player_badge_address).update_non_fungible_data(
                    &badge.local_id(), 
                    "expired",
                    true
                )
            });

            info!("Leaving game {}, returning deposited funds.", game_id);
            let game_fee = self.games[game_id].fee_to_join;
            self.games[game_id].prize_pool.take(game_fee)
        }

        pub fn initialize_game(
            &mut self,
            fee_to_join: Decimal,
            minimum_players: u64,
            epoch_game_length: u64,
        ) {

            let game = GameData {
                prize_pool: Vault::new(RADIX_TOKEN),
                game_id: self.number_of_games,
                fee_to_join: fee_to_join,
                minimum_players: minimum_players,
                winning_player: 0.into(),
                has_started: false,
                has_finished: false,
                paid_out: false,
                current_player_amount: 0,
                epoch_of_game_start: 0,
                epoch_game_length: epoch_game_length,
            };

            self.games.push(game);
            self.number_of_games += 1;
        }
    }
}
