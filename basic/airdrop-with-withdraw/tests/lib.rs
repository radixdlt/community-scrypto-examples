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
    
    let first_add_receipt = test_env.add_recipient(recipient_account, RADIX_TOKEN, token_by_recipient); 
    assert!(first_add_receipt.success);

    let second_add_receipt = test_env.add_recipient(recipient_account, RADIX_TOKEN ,token_by_recipient); 

    let log_message = &second_add_receipt.logs.get(0).unwrap().1;
    assert!(!second_add_receipt.success);
    assert!(log_message.starts_with("Panicked at 'Recipient already exist'"));
}



#[test]
fn try_withdraw_without_added_recipients_must_be_failed() {
    // Set up environment.

    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN);
    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

    // withdraw_token    
    let  (not_recipient_key, not_recipient_address)  = test_env.new_account();
    let not_withdraw_receipt =  test_env.withdraw_token(not_recipient_key, not_recipient_address);
    assert!(!not_withdraw_receipt.success);
    let log_message = &not_withdraw_receipt.logs.get(0).unwrap().1;
    assert!(log_message.starts_with("Panicked at 'Insufficient balance'"));
}

#[test]
fn try_withdraw_after_added_recipients_must_be_succeeded() {
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
        let add_recipient_receipt = test_env.add_recipient(recipient_address, RADIX_TOKEN ,token_by_recipient );
        assert!(add_recipient_receipt.success); 
        recipients_accounts.push((recipient_key, recipient_address));  
    }

    assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap() , Decimal::from_str("1000000").unwrap() - token_by_recipient * recipient_count);

    // withdraw_token
    for (recipient_key, recipient_address)  in recipients_accounts {
            let withdraw_receipt =  test_env.withdraw_token(recipient_key, recipient_address);
            assert_eq!(format!("withdraw_token : {}", token_by_recipient), withdraw_receipt.logs.get(0).unwrap().1);
            assert_eq!(test_env.get_balance(recipient_address, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap() + token_by_recipient);
    }
}

struct TestEnv<'a> {
    executor: TransactionExecutor<'a, InMemoryLedger>,
    admin_key: Address,
    admin_account: Address,
    component: Address,
    admin_badge: Address,
    user_badge : Address
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
        let user_badge = receipt.resource_def(1).unwrap(); 

        Self {
            executor,
            admin_key,
            admin_account,
            component: receipt.component(0).unwrap(),
            admin_badge,
            user_badge
        }
    }

    pub fn new_account(&mut self) -> (Address, Address) {
        let key = self.executor.new_public_key();
        return (key, self.executor.new_account(key))
    }

    

    pub fn add_recipient(&mut self, recipient: Address, token_address :Address ,tokens : Decimal) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "add_recipient",
                vec![
                    format!("{}", recipient),
                    format!("{},{}", tokens, token_address),
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

    fn withdraw_token(&mut self,recipient_key : Address, recipient_address : Address) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "withdraw_token",
                vec![
                    format!("1,{}", self.user_badge)
                ],
                Some(recipient_address),
            )
            .drop_all_bucket_refs()
            .deposit_all_buckets(recipient_address)
            .build(vec![recipient_key])
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