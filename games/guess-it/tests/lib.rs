extern crate radix_engine;
extern crate unescape;

use radix_engine::ledger::*;
use radix_engine::model::Receipt;
use scrypto::prelude::{Address, Bucket};
use scrypto_unit::*;
use serde_json::Value::Null;
use guess_it::GameSerialized;
use unescape::unescape;

#[test]
fn test_create_user() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.create_user("alice");
    test_env.create_user("bob");
    test_env.create_user("carol");

    assert!(test_env.users.contains_key("alice"));
    assert!(test_env.users.contains_key("bob"));
    assert!(test_env.users.contains_key("carol"));
    assert_eq!(test_env.users.len(), 3);
}

#[test]
fn test_get_user() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);
    test_env.create_user("alice");
    test_env.get_user("alice");
}

#[test]
fn test_acting_as() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    let user = test_env.create_user("alice");
    test_env.acting_as("alice");

    assert_eq!(user.account, test_env.current_user.unwrap().account);
    assert_eq!(user.key, test_env.current_user.unwrap().key);
}

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
    let (state, _receipt) = check_state(&mut env, component);
    let parsed: serde_json::Value = serde_json::from_str(state.as_str()).unwrap();
    println!("State: {}\nReceipt: {:#?}", parsed, _receipt.logs);
    assert_ne!(parsed["players"], Null);
}
#[test]
fn test_join() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);

    env.create_user("creator");
    env.publish_package("package", scrypto::include_code!("guess_it"));
    assert_eq!(env.packages.len(), 1);
    let (component, _def, _) = create_game(&mut env);

    let (_badge, _receipt) = join_game(&mut env, component);
    // println!("info {:?}", _receipt.logs);
    let (_response, _receipt) = check_state(&mut env, component);
    // Check state
    let (_state, _receipt) = check_state(&mut env, component);
    let _match_string1 = r#"{"players":{"42d588c2ac04d7257a77e586a9c600b5":{"guess":"","secret":"","bet":0,"share_of_pot":0}}}"#;
    let mut _unescaped = unescape(_state.as_str()).unwrap();
    // unescaped.pop();
    // unescaped.remove(0);
    // println!("info {:#}", _match_string1);
    assert_eq!(_unescaped, _state);
    println!("\nLeft: {}\nRight: {}\nOuter: {}", _unescaped, _state, _match_string1);
    // let _parsed: GameSerialized = serde_json::from_str(_unescaped.as_str()).unwrap();
    let _parsed: GameSerialized = serde_json::from_str(_state.as_str()).unwrap();
    assert_ne!(_parsed.players.len(), 0);
    // println!("match string: {:#?}", _parsed.players);

    // env.create_user("player_1");
    // env.acting_as("player_1");
    // join_game(&mut env, component);
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
    assert!(receipt.result.is_ok());

    let badge: Bucket = return_of_call_method(&mut receipt, "join");
    return (badge, receipt);
}