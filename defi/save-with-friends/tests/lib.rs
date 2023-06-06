use assert_cmd::prelude::*;
use assert_cmd::Command;
use predicates::prelude::*;
use radix_engine::blueprints::account;
use regex::Regex;
use std::collections::BTreeMap;
use std::error::Error;

// These tests will use the `resim` binary to run the tests and
// will change the state of the local resim state.

#[test]
fn publish_package() {
    Setup::new();
}

#[test]
fn test_close_early() -> Result<(), Box<dyn Error>> {
    let setup = Setup::existing();

    println!("setup: {:?}", setup);

    // instantiate the blueprint

    let cmd = format!(
        "resim call-function {} new_amount_bound  --manifests/amount_bound_close_early.rtm",
        setup.package_address
    );
    let mut env_vars = BTreeMap::new();
    env_vars.insert("account_1", setup.accounts[0].component_address.clone());
    env_vars.insert("account_2", setup.accounts[1].component_address.clone());
    env_vars.insert("account_3", setup.accounts[2].component_address.clone());
    env_vars.insert("package_address", setup.package_address.clone());
    // TODO:
    // env_vars.insert("nft_address", setup.nft_address.clone());
    run(&cmd, env_vars);

    Ok(())
}

#[derive(Debug)]
struct Setup {
    accounts: Vec<TestAccount>,
    package_address: String,
}

impl Setup {
    fn new() -> Self {
        run("resim reset", None);

        let accounts = vec![
            TestAccount::resim_new(),
            TestAccount::resim_new(),
            TestAccount::resim_new(),
        ];

        let ouput = run("resim publish .", None);
        let package_address = parse(&ouput, r"Success! New Package: ([a-zA-Z0-9_]+)");
        println!("package address: {}", package_address);

        Self {
            accounts,
            package_address,
        }
    }

    fn existing() -> Self {
        let output = run("resim show-ledger", None);

        let accounts = parse_multiple(&output, r"(account_[a-zA-Z0-9_]+)");
        println!("accounts: {:?}", accounts);

        let accounts = accounts
            .iter()
            .map(|a| TestAccount::new("unknown".into(), "unknown".into(), a.into()))
            .collect();

        let package_address = parse_multiple(&output, r"(package_[a-zA-Z0-9_]+)")
            .iter()
            .last()
            .unwrap()
            .into();

        Self {
            accounts,
            package_address,
        }
    }

    fn get_default_account_address() -> String {
        let output = run("resim show-configs", None);
        parse(&output, r"Account Address: (account_[a-zA-Z0-9_]+)")
    }
}

#[derive(Debug)]
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

    fn new(public_key: String, private_key: String, component_address: String) -> Self {
        Self {
            public_key,
            private_key,
            component_address,
        }
    }
}

fn run(cmd: &str, env: Option<BTreeMap<String, String>>) -> String {
    println!("command: {}", cmd);
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

    let assert = cmd.assert().success();

    let output = &assert.get_output();
    println!("output: {:?}", output);
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
