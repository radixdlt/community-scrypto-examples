extern crate radix_engine;

use radix_engine::ledger::*;
use radix_engine::model::Receipt;
use scrypto::prelude::{Address, Bucket};
use scrypto_unit::*;
use serde_json::Value::Null;

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
fn test_join() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);

    env.create_user("creator");
    env.publish_package("package", scrypto::include_code!("guess_it"));
    assert_eq!(env.packages.len(), 1);
    let (component, _def, _) = create_game(&mut env);

    join_game(&mut env, component);
    let (_response, _receipt) = check_state(&mut env, component);
    // Check state
    let state = check_state(&mut env, component).0;
    let parsed: serde_json::Value = serde_json::from_str(state.as_str()).unwrap();
    assert_ne!(parsed["players"], Null);

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