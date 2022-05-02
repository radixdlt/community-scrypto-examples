// TODO: The tests here rely on the scrypto-unit module which is not currently compatible with v0.4.0 yet. We should 
// look into updating them once it's compatible.

// extern crate radix_engine;

// use std::collections::HashMap;
// use radix_engine::ledger::*;
// use radix_engine::model::{Receipt};
// use scrypto::prelude::{Actor, Address, Bucket, H256, NonFungibleKey};
// use scrypto::types::RADIX_TOKEN;
// use scrypto_unit::*;
// use guess_it::{GameSerialized, Player, State};

// #[test]
// fn test_create() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();
//     let mut env = TestEnv::new(&mut ledger);

//     let _creator = env.create_user("creator");
//     env.publish_package("package", scrypto::include_code!("guess_it"));
//     assert_eq!(env.packages.len(), 1);

//     let (_component, _def, _) = create_game(&mut env);
// }

// #[test]
// fn test_check_state() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();
//     let mut env = TestEnv::new(&mut ledger);

//     env.create_user("creator");
//     env.publish_package("package", scrypto::include_code!("guess_it"));
//     assert_eq!(env.packages.len(), 1);

//     let (component, _def, _) = create_game(&mut env);
//     // Check state
//     let (state, _) = check_state(&mut env, component);
//     let parsed: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
//     assert!(parsed.players.len() < 1, "Expected 0 players at this point");
// }

// #[test]
// fn test_join() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();
//     let mut env = TestEnv::new(&mut ledger);
//     let rad_payment: String = format!("1,{}", RADIX_TOKEN.to_string());

//     env.create_user("creator");
//     env.publish_package("package", scrypto::include_code!("guess_it"));
//     assert_eq!(env.packages.len(), 1);
//     let (component, def, _) = create_game(&mut env);
//     let bucket = format!("1,{}", def.to_string());

//     // Creator joins game
//     let ((_badge, _difference, _nft_id), _receipt) = join_game(&mut env, component, &rad_payment, &bucket);
//     // Check state
//     let (state, _receipt) = check_state(&mut env, component);
//     // Assert against state
//     let game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
//     assert_ne!(game.players.len(), 0);
//     assert_eq!(game.state, State::AcceptingChallenger);

//     env.create_user("player_1");
//     env.acting_as("player_1");
//     // player_1 joins game
//     join_game(&mut env, component, &rad_payment, &bucket);
//     // Check state
//     let (_state, _receipt) = check_state(&mut env, component);
//     // Assert against state
//     let game: GameSerialized = serde_json::from_str(_state.as_str()).unwrap();
//     assert_eq!(game.players.len(), 2);
//     assert_eq!(game.state, State::MakeGuess);
// }

// #[test]
// fn test_make_guess() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();
//     let mut env = TestEnv::new(&mut ledger);
//     let rad_payment: String = format!("1,{}", RADIX_TOKEN.to_string());

//     env.create_user("creator");
//     env.publish_package("package", scrypto::include_code!("guess_it"));

//     let (component, def, _) = create_game(&mut env);
//     let bucket = format!("1,{}", def.to_string());
//     // Creator joins game
//     let ((_badge, _difference, _nft_id), _receipt) = join_game(&mut env, component, &rad_payment, &bucket);

//     env.create_user("player_1");
//     env.acting_as("player_1");
//     let ((_badge, _difference, _nft_id), _receipt) = join_game(&mut env, component, &rad_payment, &bucket);
//     let badge = format!("1,{}", def.to_string());

//     // SUT
//     // Player 1 guesses
//     let (_response, _receipt) = make_guess(&mut env, component, "1".to_string(), &badge);
//     // Assert against state
//     let (state, _) = check_state(&mut env, component);
//     let game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
//     let guesses: Vec<u128> = game.players.into_iter()
//         .map(|(_key, player)| player.get_guess())
//         .filter(|guess| guess > &0u128)
//         .collect();
//     assert_eq!(guesses.len(), 1);

//     // Creator guesses
//     env.acting_as("creator");
//     let (_response, _receipt) = make_guess(&mut env, component, "3".to_string(), &badge);
//     // Assert against state
//     let (state, _) = check_state(&mut env, component);
//     let game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
//     let guesses: Vec<u128> = game.players.into_iter()
//         .map(|(_key, player)| player.get_guess())
//         .filter(|guess| guess > &0u128)
//         .collect();
//     assert_eq!(guesses.len(), 2);
//     assert_eq!(game.state, State::Payout);
// }

// #[test]
// fn test_check_winner() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();
//     let mut env = TestEnv::new(&mut ledger);
//     let rad_payment: String = format!("1,{}", RADIX_TOKEN.to_string());

//     // Create known users
//     env.create_user("creator");
//     let _creator = *env.get_user("creator");
//     env.create_user("player_1");
//     let _player_1 = *env.get_user("player_1");

//     // Creator creates game
//     env.acting_as("creator");
//     env.publish_package("package", scrypto::include_code!("guess_it"));
//     let (component, def, _) = create_game(&mut env);
//     let bucket = format!("1,{}", def.to_string());
//     // Creator joins game
//     let ((_badge, _difference, _creator_nft_id), _receipt) = join_game(&mut env, component, &rad_payment, &bucket);

//     // Player 1 joins game
//     env.acting_as("player_1");
//     let ((_badge, _difference, _player_1_nft_id), _receipt) = join_game(&mut env, component, &rad_payment, &bucket);
//     // Check state
//     let (state, _) = check_state(&mut env, component);
//     let _game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();

//     // Creator guesses
//     env.acting_as("creator");
//     let (_response, _receipt) = make_guess(&mut env, component, "1".to_string(), &bucket);
//     // Player 1 guesses
//     env.acting_as("player_1");
//     let (_response, _receipt) = make_guess(&mut env, component, "5".to_string(), &bucket);
//     // Assert against state
//     let (state, _) = check_state(&mut env, component);
//     let game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
//     // SUT
//     assert_eq!(game.winner, _creator_nft_id);
// }

// #[test]
// fn test_withdraw() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();
//     let mut env = TestEnv::new(&mut ledger);
//     let rad_payment: String = format!("1,{}", RADIX_TOKEN.to_string());

//     // Create known users
//     env.create_user("creator");
//     let _creator = *env.get_user("creator");
//     env.create_user("player_1");
//     let _player_1 = *env.get_user("player_1");

//     // Creator creates game
//     env.acting_as("creator");
//     env.publish_package("package", scrypto::include_code!("guess_it"));
//     let (component, def, _) = create_game(&mut env);
//     let bucket = format!("1,{}", def.to_string());
//     // Creator joins game
//     let ((_badge, _difference, _creator_nft_id), _receipt) = join_game(&mut env, component, &rad_payment, &bucket);

//     // Player 1 joins game
//     env.acting_as("player_1");
//     let ((_badge, _difference, _player_1_nft_id), _receipt) = join_game(&mut env, component, &rad_payment, &bucket);
//     // Check state
//     let (state, _) = check_state(&mut env, component);
//     let _game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
//     // Creator guesses
//     env.acting_as("creator");
//     let (_response, _receipt) = make_guess(&mut env, component, "1".to_string(), &bucket);
//     // Player 1 guesses
//     env.acting_as("player_1");
//     let (_response, _receipt) = make_guess(&mut env, component, "5".to_string(), &bucket);
//     // Assert against state
//     let (state, _) = check_state(&mut env, component);
//     let _game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();

//     // SUT
//     env.acting_as("creator");
//     let (_response, _receipt) = withdraw(&mut env, component, &bucket);
//     // Assert against state
//     let (state, _) = check_state(&mut env, component);
//     let _game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
//     assert_ne!(_game.winner, "00000000000000000000000000000000");
// }

// #[test]
// fn test_context() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();
//     let mut env = TestEnv::new(&mut ledger);

//     env.create_user("creator");
//     env.publish_package("package", scrypto::include_code!("guess_it"));
//     let (component, _def, _) = create_game(&mut env);

//     // Test context for random number generation
//     env.executor.ledger_mut().set_epoch(15);
//     let (_response, _receipt): ((Actor, Address, H256, u64, u128), Receipt) = _get_context(&mut env, component);
//     // println!("Uuid: {} - Epoch: {}", _response.4 % 6, _response.3);
//     assert_eq!(env.executor.ledger().get_epoch(), 15);
// }

// #[test]
// fn test_rng() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();
//     let mut env = TestEnv::new(&mut ledger);

//     env.create_user("creator");
//     env.publish_package("package", scrypto::include_code!("guess_it"));
//     let (component, _def, _) = create_game(&mut env);

//     let (response, _receipt): ((Actor, Address, H256, u64, u128), Receipt) = _get_context(&mut env, component);
//     let players = HashMap::from([
//         (0, Player { guess: 3u128 }),
//         (1, Player { guess: 4u128 })
//     ]);
//     let multiplier = players.into_iter()
//         .map(|(_,p)| p.guess)
//         .reduce(|a,b| a * b).unwrap_or(1);
//     let uuid = response.4;
//     let rng = ((uuid / multiplier) % 6) + 1;
//     // println!("Uuid: {}, multiplier: {}, rng: {}", uuid, multiplier, rng);
//     assert_ne!(rng, 0);
// }

// #[test]
// fn test_nft_keys_are_comparable() {
//     let v1: Vec<NonFungibleKey> = vec!();
//     let v2: Vec<NonFungibleKey> = vec!();
//     assert_eq!(v1, v2);
// }

// fn create_game<'a, L: SubstateStore>(
//     env: &mut TestEnv<L>,
// ) -> (Address, Address, Receipt) {
//     let params = vec!["game name".into(), "1".into()];
//     let receipt = env.call_function("GuessIt", "create", params);
//     let component = receipt.component(0).unwrap();
//     let badge_def = receipt.resource_def(0).unwrap();
//     assert!(receipt.result.is_ok());

//     (component, badge_def, receipt)
// }

// fn check_state(env: &mut TestEnv<InMemorySubstateStore>, component: Address) -> (String, Receipt) {
//     let params = vec!["1".into()];
//     let mut receipt = env.call_method(&component, "state", params);
//     let response: String = return_of_call_method(&mut receipt, "state");

//     assert!(receipt.result.is_ok());
//     (response, receipt)
// }

// fn join_game(env: &mut TestEnv<InMemorySubstateStore>, component: Address, payment: &str, bucket: &String) -> ((Bucket, Bucket, String), Receipt) {
//     let mut receipt = env.call_method(&component, "join", vec![payment.to_string(), bucket.clone()]);
//     // println!("Receipt from join_game: {:?}", receipt.outputs);
//     assert!(receipt.result.is_ok());
//     let (badge, difference, nft_id): (Bucket, Bucket, String) = return_of_call_method(&mut receipt, "join");
//     return ((badge, difference, nft_id), receipt);
// }

// fn make_guess(env: &mut TestEnv<InMemorySubstateStore>, component: Address, guess: String, badge: &String) -> (String, Receipt) {
//     let mut receipt = env.call_method(&component, "make_guess", vec![guess, badge.to_string()]);
//     assert!(receipt.result.is_ok());

//     let response: String = return_of_call_method(&mut receipt, "make_guess");
//     return (response, receipt);
// }

// fn withdraw(env: &mut TestEnv<InMemorySubstateStore>, component: Address, badge: &String) -> ((Bucket, String), Receipt) {
//     let mut receipt = env.call_method(&component, "withdraw_funds", vec![badge.clone()]);
//     assert!(receipt.result.is_ok());

//     let response: (Bucket, String) = return_of_call_method(&mut receipt, "withdraw_funds");
//     return (response, receipt);
// }

// fn _get_context(env: &mut TestEnv<InMemorySubstateStore>, component: Address) -> ((Actor, Address, H256, u64, u128), Receipt) {
//     let mut receipt = env.call_method(&component, "get_context", vec![]);
//     // println!("Receipt from join_game: {:?}", receipt.outputs);
//     assert!(receipt.result.is_ok());

//     let tuple: (Actor, Address, H256, u64, u128) = return_of_call_method(&mut receipt, "get_context");
//     return ((tuple), receipt);
// }