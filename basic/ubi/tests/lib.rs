use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn register_by_admin() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    let receipt = test_env.register(test_env.admin_account, test_env.admin_account, test_env.admin_key);
    assert!(receipt.success);
}

#[test]
fn register_by_person() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    let receipt = test_env.register(test_env.person_account, test_env.person_account, test_env.person_key);
    assert!(!receipt.success);
}

#[test]
fn register_twice_by_admin() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.register(test_env.admin_account, test_env.admin_account, test_env.admin_key);
    test_env.advance_epoch(5);
    let receipt = test_env.register(test_env.admin_account, test_env.admin_account, test_env.admin_key);
    assert!(receipt.success);
}

#[test]
fn available_tokens_person() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(5);
    let receipt = test_env.available_tokens(test_env.person_account, test_env.person_key);
    assert!(receipt.success);
    assert_eq!("available_tokens: 5", receipt.logs.get(0).unwrap().1);
}

#[test]
fn available_tokens_bot() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(5);
    let receipt = test_env.available_tokens(test_env.bot_account, test_env.bot_key);
    assert!(!receipt.success);
}

#[test]
fn collect_ubi_person() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(33);
    let receipt = test_env.collect_ubi(test_env.person_account, test_env.person_key);
    assert!(receipt.success);
    assert_eq!(test_env.get_balance(test_env.person_account, test_env.ubi_token).unwrap(),  Decimal::from_str("33").unwrap());
}

#[test]
fn collect_ubi_bot() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(5);
    let receipt = test_env.collect_ubi(test_env.bot_account, test_env.bot_key);
    assert!(!receipt.success);
}

#[test]
fn send_tokens_to_bot() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(20);
    test_env.collect_ubi(test_env.person_account, test_env.person_key);
    let receipt = test_env.send_tokens(test_env.person_account, test_env.person_key, test_env.bot_account, 10);
    assert!(receipt.success);
    assert_eq!(test_env.get_balance(test_env.person_account, test_env.ubi_token).unwrap(),  Decimal::from_str("10").unwrap());
    assert_eq!(test_env.get_balance(test_env.bot_account, test_env.ubi_token).unwrap(),  Decimal::from_str("8").unwrap());
}

struct TestEnv<'a> {
    executor: TransactionExecutor<'a, InMemoryLedger>,
    ubi_token: Address,
    admin_key: Address,
    admin_account: Address,
    person_key: Address,
    person_account: Address,
    bot_key: Address,
    bot_account: Address,
    component: Address,
    admin_badge: Address,
    person_badge: Address
}

impl<'a> TestEnv<'a> {
    pub fn new(ledger: &'a mut InMemoryLedger) -> Self {
        let mut executor = TransactionExecutor::new(ledger, 0, 0);

        let package = executor.publish_package(include_code!("ubi"));
        let admin_key = executor.new_public_key();
        let admin_account = executor.new_account(admin_key);

        let tx = TransactionBuilder::new(&executor)
            .call_function(package, "UBI", "new", vec![], Some(admin_account))
            .drop_all_bucket_refs()
            .deposit_all_buckets(admin_account)
            .build(vec![admin_key])
            .unwrap();
        let receipt = executor.run(tx, false).unwrap();
        println!("{:?}\n", receipt);
        assert!(receipt.success);

        let admin_badge = receipt.resource_def(1).unwrap();
        let person_badge = receipt.resource_def(2).unwrap();
        let ubi_token = receipt.resource_def(3).unwrap();
        let person_key = executor.new_public_key();
        let person_account = executor.new_account(person_key);
        let bot_key = executor.new_public_key();
        let bot_account = executor.new_account(bot_key);

        let mut test_env = Self {
            executor,
            ubi_token,
            admin_key,
            admin_account,
            person_key,
            person_account,
            bot_key,
            bot_account,
            component: receipt.component(0).unwrap(),
            admin_badge,
            person_badge
        };
        test_env.register(person_account, admin_account, admin_key);
        test_env
    }

    pub fn advance_epoch(&mut self, amount: u64) {
        self.executor.set_current_epoch(self.executor.current_epoch() + amount);
    }

    pub fn register(&mut self, address: Address, admin_address: Address, admin_key: Address) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(self.component, "register", vec![address.to_string(), format!("1,{}", self.admin_badge)], Some(admin_address))
            .drop_all_bucket_refs()
            .deposit_all_buckets(admin_address)
            .build(vec![admin_key])
            .unwrap();
        let receipt = self.executor.run(tx, false).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    pub fn available_tokens(&mut self, address: Address, key: Address) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(self.component, "available_tokens", vec![format!("1,{}", self.person_badge)], Some(address))
            .drop_all_bucket_refs()
            .deposit_all_buckets(address)
            .build(vec![key])
            .unwrap();
        let receipt = self.executor.run(tx, false).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    pub fn collect_ubi(&mut self, address: Address, key: Address) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(self.component, "collect_ubi", vec![format!("1,{}", self.person_badge)], Some(address))
            .drop_all_bucket_refs()
            .deposit_all_buckets(address)
            .build(vec![key])
            .unwrap();
        let receipt = self.executor.run(tx, false).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    pub fn send_tokens(&mut self, address: Address, key: Address, to: Address, amount: u64) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(self.component, "send_tokens", vec![to.to_string(), format!("{},{}", amount, self.ubi_token)], Some(address))
            .drop_all_bucket_refs()
            .deposit_all_buckets(address)
            .build(vec![key])
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
                .state(radix_engine::model::Actor::SuperUser)
                .unwrap(),
            ledger,
            &mut vaults,
        ).unwrap();
        for vid in vaults {
            let vault = self.executor.ledger().get_vault(vid).unwrap();
            let amount = vault.amount(radix_engine::model::Actor::SuperUser).unwrap();
            let resource_def_address = vault
                .resource_address(radix_engine::model::Actor::SuperUser)
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
