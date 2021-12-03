use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_airdrop_0_users() {
    let mut ledger = InMemoryLedger::with_bootstrap();

    // Set up environment.
    let mut test_env = TestEnv::new(&mut ledger);

    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    let receipt = test_env.perform_airdrop(1000, RADIX_TOKEN);
    assert!(!receipt.success);
    let log_message = &receipt.logs.get(0).unwrap().1;
    assert!(log_message.starts_with("Panicked at 'You must register at least one recipient before performing an airdrop'"));

    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
}

#[test]
fn test_airdrop_1_user() {
    let mut ledger = InMemoryLedger::with_bootstrap();

    // Set up environment.
    let mut test_env = TestEnv::new(&mut ledger);
    let (_, user1_account) = test_env.new_account();

    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
    assert_eq!(test_env.get_balance(user1_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    test_env.add_recipient(user1_account);

    let receipt = test_env.perform_airdrop(1000, RADIX_TOKEN);
    println!("{:?}", receipt);
    assert!(receipt.success);

    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("999000").unwrap());
    assert_eq!(test_env.get_balance(user1_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1001000").unwrap());
}

#[test]
fn test_airdrop_3_users() {
    let mut ledger = InMemoryLedger::with_bootstrap();

    // Set up environment.
    let mut test_env = TestEnv::new(&mut ledger);
    let (_, user1_account) = test_env.new_account();
    let (_, user2_account) = test_env.new_account();
    let (_, user3_account) = test_env.new_account();

    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
    assert_eq!(test_env.get_balance(user1_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
    assert_eq!(test_env.get_balance(user2_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
    assert_eq!(test_env.get_balance(user3_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    test_env.add_recipient(user1_account);
    test_env.add_recipient(user2_account);
    test_env.add_recipient(user3_account);

    let receipt = test_env.perform_airdrop(1000, RADIX_TOKEN);
    assert!(receipt.success);

    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("999000").unwrap());
    assert_eq!(test_env.get_balance(user1_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000333.333333333333333333").unwrap());
    assert_eq!(test_env.get_balance(user2_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000333.333333333333333333").unwrap());
    assert_eq!(test_env.get_balance(user3_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000333.333333333333333334").unwrap());
}

struct TestEnv<'a> {
    executor: TransactionExecutor<'a, InMemoryLedger>,
    admin_key: Address,
    admin_account: Address,
    component: Address,
    admin_badge: Address
}

impl<'a> TestEnv<'a> {
    pub fn new(ledger: &'a mut InMemoryLedger) -> Self {
        let mut executor = TransactionExecutor::new(ledger, 0, 0);

        let package = executor.publish_package(include_code!());
        let admin_key = executor.new_public_key();
        let admin_account = executor.new_account(admin_key);

        let tx = TransactionBuilder::new(&executor)
            .call_function(package, "Airdrop", "new", vec![], None)
            .deposit_all_buckets(admin_account)
            .drop_all_bucket_refs()
            .build(vec![admin_key])
            .unwrap();
        let receipt = executor.run(tx, false).unwrap();
        println!("{:?}\n", receipt);
        assert!(receipt.success);

        let admin_badge = receipt.resource_def(0).unwrap();

        Self {
            executor,
            admin_key,
            admin_account,
            component: receipt.component(0).unwrap(),
            admin_badge
        }
    }

    pub fn new_account(&mut self) -> (Address, Address) {
        let key = self.executor.new_public_key();
        (key, self.executor.new_account(key))
    }

    pub fn add_recipient(&mut self, recipient: Address) {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "add_recipient",
                vec![
                    format!("{}", recipient),
                    format!("1,{}", self.admin_badge)
                ],
                Some(self.admin_account),
            )
            .deposit_all_buckets(self.admin_account)
            .drop_all_bucket_refs()
            .build(vec![self.admin_key])
            .unwrap();
        let receipt = self.executor.run(tx, false).unwrap();
        println!("{:?}\n", receipt);
        assert!(receipt.success);
    }

    fn perform_airdrop(&mut self, amount: i32, token: Address) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "perform_airdrop",
                vec![
                    format!("{},{}", amount, token),
                    format!("1,{}", self.admin_badge)
                ],
                Some(self.admin_account),
            )
            .deposit_all_buckets(self.admin_account)
            .drop_all_bucket_refs()
            .build(vec![self.admin_key])
            .unwrap();
        let receipt = self.executor.run(tx, false).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    fn get_balance(&self, account: Address, token: Address) -> Result<Decimal, String> {
        let ledger = self.executor.ledger();
        let account_component = ledger.get_component(account).unwrap();
        let mut vaults = vec![];
        let _res = radix_engine::utils::format_data_with_ledger(
            account_component
                .state(radix_engine::model::Auth::NoAuth)
                .unwrap(),
            ledger,
            &mut vaults,
        ).unwrap();

        for vid in vaults {
            let vault = self.executor.ledger().get_vault(vid).unwrap();
            let amount = vault.amount(radix_engine::model::Auth::NoAuth).unwrap();
            let resource_def_address = vault
                .resource_def(radix_engine::model::Auth::NoAuth)
                .unwrap();
            if token == resource_def_address {
                return Ok(amount);
            }
        }

        return Err(format!(
            "No vault found for token {} in account {}",
            token, account
        ));
    }
}
