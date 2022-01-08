use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn try_add_recipient_already_exist_must_be_failed() {

    let mut ledger = InMemoryLedger::with_bootstrap();
    // Set up environment.
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
    
    let token_by_recipient =  Decimal::from(10); 
    let (_, recipient_account)  = test_env.new_account();
    
    let first_add_receipt = test_env.add_recipient(recipient_account, token_by_recipient); 
    assert!(first_add_receipt.success);

    let second_add_receipt = test_env.add_recipient(recipient_account, token_by_recipient); 
    let log_message = &second_add_receipt.logs.get(0).unwrap().1;
    assert!(!second_add_receipt.success);
    assert!(log_message.starts_with("Panicked at 'Recipient already exist'"));
}

#[test]
fn try_put_airdrop_tokens_without_recipient_must_be_failed() {

    let mut ledger = InMemoryLedger::with_bootstrap();
    // Set up environment.
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
    
    let airdrop_tokens = Decimal::from(1000); 
    let receipt = test_env.put_airdrop_tokens(airdrop_tokens, RADIX_TOKEN);
    assert!(!receipt.success);
    
    let log_message = &receipt.logs.get(0).unwrap().1;
    assert!(log_message.starts_with("Panicked at 'You must register at least one recipient before put tokens'"));
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
}

#[test]
fn try_put_airdrop_tokens_with_insufficient_tokens_must_be_failed() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    // Set up environment.
    let token_by_recipient =  Decimal::from(10); 
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    let recipient_count = 3; 
    for _ in 0..(recipient_count) {
        let (_, recipient_account)  = test_env.new_account();
        test_env.add_recipient(recipient_account, token_by_recipient); 
    }

    let airdrop_tokens = Decimal::from(20);
    let receipt = test_env.put_airdrop_tokens(airdrop_tokens, RADIX_TOKEN);
    assert!(!receipt.success);
    
    let log_message = &receipt.logs.get(0).unwrap().1;
    assert!(log_message.starts_with("Panicked at 'The tokens quantity must be 30'"),);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
}


#[test]
fn try_put_airdrop_tokens_with_sufficient_tokens_must_be_succeeded() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    // Set up environment.
    let token_by_recipient : Decimal =  Decimal::from(100); 
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    // AddRecipients

    let recipient_count  = 10; 
    let mut accounts : Vec<Address> =  Vec::new();
    for _ in 0..recipient_count {
        let (_, recipient_account)  = test_env.new_account();
        let add_recipient_receipt = test_env.add_recipient(recipient_account, token_by_recipient);
        assert!(add_recipient_receipt.success); 
        accounts.push(recipient_account);  
    }

    // put airdrop tokens
    let airdrop_tokens = token_by_recipient * Decimal::from(recipient_count); 
    let receipt = test_env.put_airdrop_tokens(airdrop_tokens, RADIX_TOKEN);
    assert!(receipt.success);
    assert!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap() == (Decimal::from_str("1000000").unwrap() - airdrop_tokens));
    assert_eq!(format!("put_airdrop_tokens with recipients_count {} required_tokens {}",recipient_count, airdrop_tokens), receipt.logs.get(0).unwrap().1);
}


#[test]
fn try_withdraw_without_put_airdrop_tokens_must_be_failed() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let token_by_recipient : Decimal =  Decimal::from(100); 
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    // AddRecipients
    let recipient_count  = 2; 
    let mut recipients_accounts : Vec<(Address,Address)> =  Vec::new();
    for _ in 0..recipient_count {
        let (recipient_key, recipient_address)  = test_env.new_account();
        let add_recipient_receipt = test_env.add_recipient(recipient_address, token_by_recipient);
        assert!(add_recipient_receipt.success); 
        recipients_accounts.push((recipient_key, recipient_address));  
    }

    // withdraw_token
    for (recipient_key, recipient_address) in recipients_accounts {
        let withdraw_receipt =  test_env.withdraw_token(recipient_key,recipient_address);
        let log_message = &withdraw_receipt.logs.get(0).unwrap().1;
        assert!(log_message.starts_with("Panicked at 'Impossible withdrawal'"));
    }

}

#[test]
fn try_withdraw_with_not_added_recipient_must_be_failed() {
    // Set up environment.

    let mut ledger = InMemoryLedger::with_bootstrap();
    let token_by_recipient : Decimal =  Decimal::from(100); 
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    // AddRecipients
    let recipient_count  = 1; 
    let mut recipients_accounts : Vec<(Address,Address)> =  Vec::new();
    for _ in 0..recipient_count {
        let (recipient_key, recipient_address)  = test_env.new_account();
        let add_recipient_receipt = test_env.add_recipient(recipient_address, token_by_recipient);
        assert!(add_recipient_receipt.success); 
        recipients_accounts.push((recipient_key, recipient_address));  
    }

    // put airdrop tokens
    let airdrop_tokens = token_by_recipient * Decimal::from(recipient_count); 
    let receipt = test_env.put_airdrop_tokens(airdrop_tokens, RADIX_TOKEN);
    assert!(receipt.success);
    assert!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap() == (Decimal::from_str("1000000").unwrap() - airdrop_tokens));

    // withdraw_token
    
    let  (not_recipient_key, not_recipient_address)  = test_env.new_account();
    let not_withdraw_receipt =  test_env.withdraw_token(not_recipient_key, not_recipient_address);
    assert!(!not_withdraw_receipt.success);
    let log_message = &not_withdraw_receipt.logs.get(0).unwrap().1;
    assert!(log_message.starts_with("Panicked at 'Recipient not found'"));
}

#[test]
fn try_withdraw_after_put_airdrop_tokens_must_be_succeeded() {
    // Set up environment.

    let mut ledger = InMemoryLedger::with_bootstrap();
    let token_by_recipient : Decimal =  Decimal::from(100); 
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    // AddRecipients
    let recipient_count  = 2; 
    let mut recipients_accounts : Vec<(Address,Address)> =  Vec::new();
    for _ in 0..recipient_count {
        let (recipient_key, recipient_address)  = test_env.new_account();
        let add_recipient_receipt = test_env.add_recipient(recipient_address, token_by_recipient);
        assert!(add_recipient_receipt.success); 
        recipients_accounts.push((recipient_key, recipient_address));  
    }

    // put airdrop tokens
    let airdrop_tokens = token_by_recipient * Decimal::from(recipient_count); 
    let receipt = test_env.put_airdrop_tokens(airdrop_tokens, RADIX_TOKEN);
    assert!(receipt.success);
    assert!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap() == (Decimal::from_str("1000000").unwrap() - airdrop_tokens));

    // withdraw_token
    for (recipient_key, recipient_address)  in recipients_accounts {
            let withdraw_receipt =  test_env.withdraw_token(recipient_key, recipient_address);
            println!("{:?}\n", withdraw_receipt);
            assert_eq!(format!("withdraw_token : {}", token_by_recipient), withdraw_receipt.logs.get(0).unwrap().1);
    }
}

struct TestEnv<'a> {
    executor: TransactionExecutor<'a, InMemoryLedger>,
    admin_key: Address,
    admin_account: Address,
    component: Address,
    admin_badge: Address
}

impl<'a> TestEnv<'a> {
    pub fn new(ledger: &'a mut InMemoryLedger, token : Address) -> Self {
        let mut executor = TransactionExecutor::new(ledger, 0, 0);

        let package = executor.publish_package(include_code!("airdrop_with_withdraw"));
        let admin_key = executor.new_public_key();
        let admin_account = executor.new_account(admin_key);

        let tx = TransactionBuilder::new(&executor)
            .call_function(package, "AirdropWithWithdraw", "new", vec!
            [
             format!("{}",token)
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

    

    pub fn add_recipient(&mut self, recipient: Address, tokens : Decimal) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "add_recipient",
                vec![
                    format!("{}", recipient),
                    format!("{}", tokens),
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
        return receipt;
    }

    fn get_required_tokens(&mut self) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "get_required_tokens",
                vec![
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
        return receipt;
    }



    fn put_airdrop_tokens(&mut self, amount: Decimal, token: Address) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "put_airdrop_tokens",
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

    fn get_recipient_available_tokens(&mut self, recipient_key : Address, recipient_address : Address) -> Receipt{
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "get_recipient_available_tokens",
                vec![
                    format!("{}", recipient_address)
                ],
                Some(self.admin_account),
            )
            .drop_all_bucket_refs()
            .deposit_all_buckets(recipient_address)
            .build(vec![recipient_key])
            .unwrap();
        let receipt = self.executor.run(tx, false).unwrap();
        println!("{:?}\n", receipt);
        return receipt
    }

    fn withdraw_token(&mut self,recipient_key : Address, recipient_address : Address) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "withdraw_token",
                vec![
                    format!("{}", recipient_address),
                ],
                Some(recipient_address),
            )
            .drop_all_bucket_refs()
            .deposit_all_buckets(recipient_address)
            .build(vec![recipient_key])
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