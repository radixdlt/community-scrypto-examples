extern crate radix_engine;

use std::borrow::{Borrow, BorrowMut};
use std::str::FromStr;
use radix_engine::ledger::*;
use radix_engine::model::{Receipt};
use scrypto::core::Uuid;
use scrypto::prelude::{Actor, Address, Bucket, Context, H256};
use scrypto_unit::*;
use guess_it::{GameSerialized, State};

#[test]
fn test_create() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);

    let _creator = env.create_user("creator");
    env.publish_package("package", scrypto::include_code!("guess_it"));
    assert_eq!(env.packages.len(), 1);

    let (_component, _def, _) = create_game(&mut env);
}

#[test]
fn test_check_state() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);

    env.create_user("creator");
    env.publish_package("package", scrypto::include_code!("guess_it"));
    assert_eq!(env.packages.len(), 1);

    let (component, _def, _) = create_game(&mut env);
    // Check state
    let (state, _) = check_state(&mut env, component);
    let parsed: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
    assert!(parsed.players.len() < 1, "Expected 0 players at this point");
}

#[test]
fn test_join() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);

    env.create_user("creator");
    env.publish_package("package", scrypto::include_code!("guess_it"));
    assert_eq!(env.packages.len(), 1);
    let (component, _def, _) = create_game(&mut env);

    // Creator joins game
    let (_badge, _receipt) = join_game(&mut env, component);
    // Check state
    let (state, _receipt) = check_state(&mut env, component);
    // Assert against state
    let game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
    assert_ne!(game.players.len(), 0);
    assert_eq!(game.state, State::AcceptingChallenger);

    env.create_user("player_1");
    env.acting_as("player_1");
    // player_1 joins game
    join_game(&mut env, component);
    // Check state
    let (_state, _receipt) = check_state(&mut env, component);
    // Assert against state
    let game: GameSerialized = serde_json::from_str(_state.as_str()).unwrap();
    assert_eq!(game.players.len(), 2);
    assert_eq!(game.state, State::MakeGuess);
}

#[test]
fn test_make_guess() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);

    env.create_user("creator");
    env.publish_package("package", scrypto::include_code!("guess_it"));

    let (component, def, _) = create_game(&mut env);
    // Creator joins game
    let (_badge, _receipt) = join_game(&mut env, component);

    env.create_user("player_1");
    env.acting_as("player_1");
    let (_badge, _receipt) = join_game(&mut env, component);
    let badge = format!("1,{}", def.to_string());

    // SUT
    // Player 1 guesses
    let (_response, _receipt) = make_guess(&mut env, component, "my-guess".to_string(), &badge);
    // Assert against state
    let (state, _) = check_state(&mut env, component);
    let game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
    let guesses: Vec<String> = game.players.into_iter()
        .map(|(_key, player)| player.get_guess())
        .filter(|guess| !guess.is_empty())
        .collect();
    assert_eq!(guesses.len(), 1);

    // Creator guesses
    env.acting_as("creator");
    let (_response, _receipt) = make_guess(&mut env, component, "my-guess".to_string(), &badge);
    // Assert against state
    let (state, _) = check_state(&mut env, component);
    let game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
    let guesses: Vec<String> = game.players.into_iter()
        .map(|(_key, player)| player.get_guess())
        .filter(|guess| !guess.is_empty())
        .collect();
    assert_eq!(guesses.len(), 2);
    assert_eq!(game.state, State::WinnerSelection);
}

#[test]
fn test_check_winner() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);

    env.create_user("creator");
    env.publish_package("package", scrypto::include_code!("guess_it"));
    let (component, def, _) = create_game(&mut env);
    // Creator joins game
    let (_badge, _receipt) = join_game(&mut env, component);

    env.create_user("player_1");
    env.acting_as("player_1");
    // Player 1 joins game
    let (_badge, _receipt) = join_game(&mut env, component);
    let badge = format!("1,{}", def.to_string());

    // Player 1 guesses
    let (_response, _receipt) = make_guess(&mut env, component, "my-guess".to_string(), &badge);
    // Creator guesses
    env.acting_as("creator");
    let (_response, _receipt) = make_guess(&mut env, component, "my-guess".to_string(), &badge);

    // SUT
    // Check Winner
    let (_response, _receipt) = withdraw(&mut env, component, &badge);
    // Assert against state
    let (state, _) = check_state(&mut env, component);
    let _game: GameSerialized = serde_json::from_str(state.as_str()).unwrap();
    println!("Game Winner: {:#?}", _game.winner)
}

#[test]
fn test() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);

    env.create_user("creator");
    env.publish_package("package", scrypto::include_code!("guess_it"));
    let (component, def, _) = create_game(&mut env);

    // Test context and random number generation
    ledger.set_epoch(10);
    let (response, _receipt): ((Actor, Address, H256, u64, u128), Receipt) = get_context(&mut env, component);
    println!("ID: {} - Epoch: {}", response.4 % 5, response.3);
}

fn create_game<'a, L: SubstateStore>(
    env: &mut TestEnv<L>,
) -> (Address, Address, Receipt) {
    let params = vec!["1".into()];
    let receipt = env.call_function("GuessIt", "create", params);
    let component = receipt.component(0).unwrap();
    let badge_def = receipt.resource_def(0).unwrap();
    assert!(receipt.result.is_ok());

    (component, badge_def, receipt)
}

fn check_state(env: &mut TestEnv<InMemorySubstateStore>, component: Address) -> (String, Receipt) {
    env.create_user("player_1");
    env.acting_as("player_1");

    let params = vec!["1".into()];
    let mut receipt = env.call_method(&component, "state", params);
    let response: String = return_of_call_method(&mut receipt, "state");

    assert!(receipt.result.is_ok());
    (response, receipt)
}

fn join_game(env: &mut TestEnv<InMemorySubstateStore>, component: Address) -> (Bucket, Receipt) {
    let mut receipt = env.call_method(&component, "join", vec![]);
    // println!("Receipt from join_game: {:?}", receipt.outputs);
    assert!(receipt.result.is_ok());
    let badge: Bucket = return_of_call_method(&mut receipt, "join");
    return (badge, receipt);
}

fn make_guess(env: &mut TestEnv<InMemorySubstateStore>, component: Address, guess: String, badge: &String) -> (String, Receipt) {
    let mut receipt = env.call_method(&component, "make_guess", vec![guess, badge.to_string()]);
    assert!(receipt.result.is_ok());

    let response: Result<String, ()> = return_of_call_method(&mut receipt, "make_guess");
    return (response.unwrap_or("Failed to get response from 'make_guess'".to_string()), receipt);
}

fn withdraw(env: &mut TestEnv<InMemorySubstateStore>, component: Address, badge: &String) -> (String, Receipt) {
    let mut receipt = env.call_method(&component, "withdraw", vec![badge.to_string()]);
    println!("Receipt from join_game: {:?}", receipt.outputs);
    assert!(receipt.result.is_ok());

    let response: Result<String, ()> = return_of_call_method(&mut receipt, "withdraw");
    return (response.unwrap_or("Failed to get response from 'reveal_secret'".to_string()), receipt);
}

fn get_context(env: &mut TestEnv<InMemorySubstateStore>, component: Address) -> ((Actor, Address, H256, u64, u128), Receipt) {
    let mut receipt = env.call_method(&component, "get_context", vec![]);
    // println!("Receipt from join_game: {:?}", receipt.outputs);
    assert!(receipt.result.is_ok());

    let tuple: (Actor, Address, H256, u64, u128) = return_of_call_method(&mut receipt, "get_context");
    return ((tuple), receipt);
}