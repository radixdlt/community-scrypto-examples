use assert_cmd::prelude::*;
use assert_cmd::Command;
use predicates::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

// These tests will use the `resim` binary to run the tests and
// will change the state of the local resim state.
// The tests are not suitable for running in parallel.
// Run fresh_setup before running other tests.

#[test]
fn fresh_setup() {
    Setup::new();
}

#[test]
fn show_info() {
    let setup = Setup::existing();

    // default account being a friend can see the info
    let cmd = format!(
        "resim call-method {} show_info -p {}:1",
        setup.env_vars.get("component_address").unwrap(),
        setup.env_vars.get("nft_address").unwrap()
    );
    run(&cmd, None);

    // non-friend account can't see the info
    let cmd = format!(
        "resim call-method {} show_info -s {}",
        setup.env_vars.get("component_address").unwrap(),
        setup.non_friends[0].private_key
    );
    compose_command(&cmd, None)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unauthorized"));
}

// this test deprecates the current setup
// rerun fresh_setup to get a new setup
#[test]
fn test_withdraw() {
    let setup = Setup::existing();
    let mut env_vars = setup.env_vars.clone();
    env_vars.insert("account".into(), setup.friends[0].component_address.clone());

    // when goal is not achieved withdraw fails
    let mut cmd = compose_command("resim run manifests/withdraw.rtm", Some(&env_vars));
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("Goal has not been achieved"));

    // when goal is achieved withdraw succeeds
    env_vars.insert("amount".into(), "1000".into());
    run("resim run manifests/deposit.rtm", Some(&env_vars));
    run("resim run manifests/withdraw.rtm", Some(&env_vars));
}

#[test]
fn test_only_friends_can_deposit() -> Result<(), Box<dyn Error>> {
    let setup = Setup::existing();

    let mut env_vars = setup.env_vars.clone();

    // deposit from account_0
    env_vars.insert("account".into(), setup.friends[0].component_address.clone());
    let cmd = format!("resim run manifests/deposit.rtm",);
    run(&cmd, Some(&env_vars));

    // deposit from account_1
    env_vars.insert("account".into(), setup.friends[1].component_address.clone());
    let cmd = format!(
        "resim run manifests/deposit.rtm -s {}",
        setup.friends[1].private_key
    );
    run(&cmd, Some(&env_vars));

    // deposit from a non-friend account
    let cmd = format!(
        "resim run manifests/deposit.rtm -s {}",
        setup.non_friends[0].private_key
    );

    let mut cmd = compose_command(&cmd, Some(&env_vars));
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unauthorized"));

    Ok(())
}

#[test]
fn test_close_early() -> Result<(), Box<dyn Error>> {
    let setup = Setup::existing();

    let keys: Vec<String> = setup
        .friends
        .iter()
        .map(|acc| acc.private_key.clone())
        .collect();

    let cmd = format!(
        "resim run manifests/close_early.rtm -s {},{},{}",
        keys[0], keys[1], keys[2]
    );

    // close early
    run(&cmd, Some(&setup.env_vars));

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Setup {
    env_vars: BTreeMap<String, String>,
    friends: Vec<TestAccount>,
    non_friends: Vec<TestAccount>,
}

impl Setup {
    fn new() -> Self {
        run("resim reset", None);

        let friends = vec![
            TestAccount::resim_new(),
            TestAccount::resim_new(),
            TestAccount::resim_new(),
        ];

        let ouput = run("resim publish .", None);
        let package_address = parse(&ouput, r"Success! New Package: ([a-zA-Z0-9_]+)");
        println!("package address: {}", package_address);

        let mut env_vars = BTreeMap::new();

        env_vars.insert("package_address".into(), package_address.clone());
        env_vars.insert("payer_account".into(), friends[0].component_address.clone());
        env_vars.insert("account_0".into(), friends[0].component_address.clone());

        env_vars.insert("account_1".into(), friends[1].component_address.clone());
        env_vars.insert("account_2".into(), friends[2].component_address.clone());
        env_vars.insert("amount".into(), "100".into());

        // component_address
        let cmd = format!("resim run manifests/instantiate.rtm");
        let ouput = run(&cmd, Some(&env_vars));
        env_vars.insert(
            "component_address".into(),
            parse(&ouput, r"Component: ([a-zA-Z0-9_]+)"),
        );

        // nft_address
        let resource_addresses = parse_multiple(&ouput, r"Resource: ([a-zA-Z0-9_]+)");
        let nft_address = resource_addresses.iter().last().unwrap();
        env_vars.insert("nft_address".into(), nft_address.clone());

        let setup = Self {
            env_vars,
            friends,
            non_friends: vec![TestAccount::resim_new()],
        };
        to_file(&json!(setup));
        setup
    }

    fn existing() -> Self {
        // publishing a package takes a few seconds
        // use existing setup.json to save time
        // read from file
        let file = fs::File::open("tests/tmp/setup.json").unwrap();
        // deserialize it
        let setup: Setup = serde_json::from_reader(file).unwrap();
        setup
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TestAccount {
    public_key: String,
    private_key: String,
    component_address: String,
}

impl TestAccount {
    fn resim_new() -> Self {
        let resim_output = run("resim new-account", None);
        let public_key = parse(&resim_output, r"Public key: ([a-zA-Z0-9]+)");
        let private_key = parse(&resim_output, r"Private key: ([a-zA-Z0-9]+)");
        let component_address = parse(&resim_output, r"Account component address: ([a-zA-Z0-9_]+)");

        Self {
            public_key,
            private_key,
            component_address,
        }
    }
}

fn compose_command(cmd: &str, env: Option<&BTreeMap<String, String>>) -> Command {
    let mut input = cmd.split(" ").into_iter();
    let cmd = input.next().unwrap();
    let mut cmd = Command::new(cmd);

    if let Some(env) = env {
        for (key, value) in env {
            cmd.env(key, value);
        }
    }

    for arg in input {
        cmd.arg(arg);
    }

    cmd
}

fn run(cmd: &str, env: Option<&BTreeMap<String, String>>) -> String {
    println!("command: {}", cmd);
    let mut cmd = compose_command(cmd, env);

    let assert = cmd.assert().success();

    let output = &assert.get_output();
    // println!("output: {:?}", output);
    let output = &output.stdout;
    let output = String::from_utf8(output.to_vec()).unwrap();

    println!("output: {}", output);
    output
}

fn parse(output: &str, regex: &str) -> String {
    let re = Regex::new(regex).unwrap();
    let captures = re.captures(output).unwrap();
    captures.get(1).unwrap().as_str().to_string()
}

fn parse_multiple(output: &str, regex: &str) -> Vec<String> {
    let re = Regex::new(regex).unwrap();
    let captures = re.captures_iter(output);
    let mut result = vec![];
    for capture in captures {
        result.push(capture.get(1).unwrap().as_str().to_string());
    }
    result
}

fn to_file(value: &Value) {
    fs::create_dir_all("tests/tmp").unwrap();
    let mut file = fs::File::create("tests/tmp/setup.json").unwrap();
    serde_json::to_writer_pretty(&mut file, value).unwrap();
}
