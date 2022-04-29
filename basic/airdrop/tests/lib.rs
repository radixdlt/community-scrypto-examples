// TODO: Update the tests for this example once the `balances` method on the account blueprint has been changed to allow
// unauthenticated calls to it.

// use radix_engine::model::{SignedTransaction, Receipt};
// use radix_engine::transaction::*;
// use radix_engine::ledger::*;
// use scrypto::prelude::*;

// #[test] 
// fn test_airdrop_0_users() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();

//     // Set up environment.
//     let mut test_env = TestEnv::new(&mut ledger);

//     assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

//     let receipt = test_env.perform_airdrop(1000, RADIX_TOKEN);
//     assert!(!receipt.result.is_ok());
//     let log_message = &receipt.logs.get(0).unwrap().1;
//     assert!(log_message.starts_with("Panicked at 'You must register at least one recipient before performing an airdrop'"));

//     assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
// }

// #[test]
// fn test_airdrop_1_user() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();

//     // Set up environment.
//     let mut test_env = TestEnv::new(&mut ledger);
//     let (_, user1_account) = test_env.new_account();

//     assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
//     assert_eq!(test_env.get_balance(user1_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

//     test_env.add_recipient(user1_account);

//     let receipt = test_env.perform_airdrop(1000, RADIX_TOKEN);
//     println!("{:?}", receipt);
//     assert!(receipt.result.is_ok());

//     assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("999000").unwrap());
//     assert_eq!(test_env.get_balance(user1_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1001000").unwrap());
// }

// #[test]
// fn test_airdrop_3_users() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();

//     // Set up environment.
//     let mut test_env = TestEnv::new(&mut ledger);
//     let (_, user1_account) = test_env.new_account();
//     let (_, user2_account) = test_env.new_account();
//     let (_, user3_account) = test_env.new_account();

//     assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
//     assert_eq!(test_env.get_balance(user1_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
//     assert_eq!(test_env.get_balance(user2_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());
//     assert_eq!(test_env.get_balance(user3_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000000").unwrap());

//     test_env.add_recipient(user1_account);
//     test_env.add_recipient(user2_account);
//     test_env.add_recipient(user3_account);

//     let receipt = test_env.perform_airdrop(1000, RADIX_TOKEN);
//     assert!(receipt.result.is_ok());

//     assert_eq!(test_env.get_balance(test_env.admin_account, RADIX_TOKEN).unwrap(), Decimal::from_str("999000").unwrap());
//     assert_eq!(test_env.get_balance(user1_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000333.333333333333333333").unwrap());
//     assert_eq!(test_env.get_balance(user2_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000333.333333333333333333").unwrap());
//     assert_eq!(test_env.get_balance(user3_account, RADIX_TOKEN).unwrap(), Decimal::from_str("1000333.333333333333333334").unwrap());
// }

// struct TestEnv<'a> {
//     executor: TransactionExecutor<'a, InMemorySubstateStore>,
//     admin_key: EcdsaPublicKey,
//     admin_account: Address,
//     component: Address,
//     admin_badge: Address
// }

// impl<'a> TestEnv<'a> {
//     pub fn new(ledger: &'a mut InMemorySubstateStore) -> Self {
//         let mut executor = TransactionExecutor::new(ledger, false);

//         let package = executor.publish_package(include_code!("airdrop"));
//         let admin_key = executor.new_public_key();
//         let admin_account = executor.new_account(admin_key);

//         let tx = TransactionBuilder::new(&executor)
//             .call_function(package.unwrap(), "Airdrop", "new", vec![], None)
//             .call_method_with_all_resources(admin_account, "deposit_batch")
//             .build(vec![admin_key])
//             .unwrap();

//         let receipt = executor.run(tx).unwrap();
        
//         println!("{:?}\n", receipt);
//         assert!(receipt.result.is_ok());

//         let admin_badge = receipt.resource_def(0).unwrap();

//         Self {
//             executor,
//             admin_key,
//             admin_account,
//             component: receipt.component(0).unwrap(),
//             admin_badge
//         }
//     }

//     pub fn new_account(&mut self) -> (EcdsaPublicKey, Address) {
//         let key = self.executor.new_public_key();
//         (key, self.executor.new_account(key))
//     }

//     pub fn add_recipient(&mut self, recipient: Address) {
//         let tx = TransactionBuilder::new(&self.executor)
//             .call_method(
//                 self.component,
//                 "add_recipient",
//                 vec![
//                     format!("{}", recipient),
//                     format!("1,{}", self.admin_badge)
//                 ],
//                 Some(self.admin_account),
//             )
//             .call_method_with_all_resources(self.admin_account, "deposit_batch")
//             .build(vec![self.admin_key])
//             .unwrap();
//         let receipt = self.executor.run(tx).unwrap();
//         println!("{:?}\n", receipt);
//         assert!(receipt.result.is_ok());
//     }

//     fn perform_airdrop(&mut self, amount: i32, token: Address) -> radix_engine::model::Receipt {
//         let tx = TransactionBuilder::new(&self.executor)
//             .call_method(
//                 self.component,
//                 "perform_airdrop",
//                 vec![
//                     format!("{},{}", amount, token),
//                     format!("1,{}", self.admin_badge)
//                 ],
//                 Some(self.admin_account),
//             )
//             .call_method_with_all_resources(self.admin_account, "deposit_batch")
//             .build(vec![self.admin_key])
//             .unwrap();
//         let receipt = self.executor.run(tx).unwrap();
//         println!("{:?}\n", receipt);
//         receipt
//     }

//     fn get_balance(&self, account: ComponentAddress, token: ResourceAddress) -> Result<Decimal, String> {
//         let balances_transaction: SignedTransaction = TransactionBuilder::new()
//             .call_method(account, "balance", args!(token))
//             .build(executor.get_nonce([admin_public_key]))
//             .sign([&admin_private_key]);
//     }
// }
