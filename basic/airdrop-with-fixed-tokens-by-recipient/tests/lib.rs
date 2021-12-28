use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn try_perform_airdrop_without_recipient_must_panicked() {

    let mut ledger = InMemoryLedger::with_bootstrap();
    // Set up environment.
    let token_by_recipient =  Decimal::from(10); 
    let mut test_env = TestEnv::new(&mut ledger, token_by_recipient);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
    let airdrop_tokens = Decimal::from(1000); 
    let receipt = test_env.perform_airdrop(airdrop_tokens, RADIX_TOKEN);
    assert!(!receipt.success);
    let log_message = &receipt.logs.get(0).unwrap().1;
    assert!(log_message.starts_with("Panicked at 'You must register at least one recipient before performing an airdrop'"));
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
}

#[test]
fn try_perform_airdrop_with_insufficient_tokens_must_panicked() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    // Set up environment.
    let token_by_recipient =  Decimal::from(10); 
    let mut test_env = TestEnv::new(&mut ledger, token_by_recipient);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    let recipient_count = 3; 
    for _ in 0..(recipient_count) {
        let (_, recipient_account)  = test_env.new_account();
        test_env.add_recipient(recipient_account); 
    }

    let airdrop_tokens = Decimal::from(20);
    let receipt = test_env.perform_airdrop(airdrop_tokens, RADIX_TOKEN);
    assert!(!receipt.success);
    let log_message = &receipt.logs.get(0).unwrap().1;
    assert!(log_message.starts_with("Panicked at 'The tokens quantity is not sufficient'"));
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

}

#[test]
fn try_perform_airdrop_with_sufficient_tokens_must_be_succeeded() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    // Set up environment.
    let token_by_recipient : Decimal =  Decimal::from(100); 
    let mut test_env = TestEnv::new(&mut ledger, token_by_recipient);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    let recipient_count  = 10; 
    for _ in 0..recipient_count {
        let (_, recipient_account)  = test_env.new_account();
        test_env.add_recipient(recipient_account); 
    }

    let airdrop_tokens = token_by_recipient * Decimal::from(recipient_count); 
    let receipt = test_env.perform_airdrop(airdrop_tokens, RADIX_TOKEN);
    assert!(receipt.success);
    assert!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap() == (Decimal::from_str("1000000").unwrap() - airdrop_tokens));
}


#[test]
fn try_perform_airdrop_with_additionnal_tokens_must_be_succeeded() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    // Set up environment.
    let token_by_recipient : Decimal =  Decimal::from(100); 
    let mut test_env = TestEnv::new(&mut ledger, token_by_recipient);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    let recipient_count  = 10; 
    for _ in 0..recipient_count {
        let (_, recipient_account)  = test_env.new_account();
        test_env.add_recipient(recipient_account); 
    }

    let additional_tokens = 2; 
    let airdrop_tokens = token_by_recipient * Decimal::from(recipient_count) + additional_tokens; 
    let receipt = test_env.perform_airdrop(airdrop_tokens, RADIX_TOKEN);
    assert!(receipt.success);
    assert!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap() == (Decimal::from_str("1000000").unwrap() - airdrop_tokens));
}
struct TestEnv<'a> {
    executor: TransactionExecutor<'a, InMemoryLedger>,
    admin_key: Address,
    admin_account: Address,
    component: Address,
    admin_badge: Address 
}

impl<'a> TestEnv<'a> {
    pub fn new(ledger: &'a mut InMemoryLedger,tokens_by_recipient : Decimal) -> Self {
        let mut executor = TransactionExecutor::new(ledger, 0, 0);

        let package = executor.publish_package(include_code!("airdrop_with_fixed_tokens_by_recipient"));
        let admin_key = executor.new_public_key();
        let admin_account = executor.new_account(admin_key);

        let tx = TransactionBuilder::new(&executor)
            .call_function(package, "AirdropWithFixedTokensByRecipient", "new", vec!
            [
             format!("{}",tokens_by_recipient)
            ], None)
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
        return (key, self.executor.new_account(key))
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
            .drop_all_bucket_refs()
            .deposit_all_buckets(self.admin_account)
            .build(vec![self.admin_key])
            .unwrap();
        let receipt = self.executor.run(tx, false).unwrap();
        println!("{:?}\n", receipt);
        assert!(receipt.success);
    }

    fn perform_airdrop(&mut self, amount: Decimal, token: Address) -> Receipt {
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
            .drop_all_bucket_refs()
            .deposit_all_buckets(self.admin_account)
            .build(vec![self.admin_key])
            .unwrap();
        let receipt = self.executor.run(tx, false).unwrap();
        println!("{:?}\n", receipt);
        return receipt
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