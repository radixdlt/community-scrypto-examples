use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_multisig() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);

    let key1 = executor.new_public_key();
    let account1 = executor.new_account(key1);
    let key2 = executor.new_public_key();
    let account2 = executor.new_account(key2);
    let key3 = executor.new_public_key();
    let account3 = executor.new_account(key3);

    let package = executor.publish_package(include_code!("multisig_transmission_maker")).unwrap();

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "MultiSigMaker", "new", vec!["2".to_string(), "2".to_string(), format!("{}", account3), format!("1000,{}", RADIX_TOKEN)], Some(account1))
        .call_method_with_all_resources(account1, "deposit_batch")
        .build(vec![key1])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    let badge_resource_def = receipt1.resource_def(1).unwrap();

    // Send one badge to account 2
    let send_badge = TransactionBuilder::new(&executor)
        .withdraw_from_account(&radix_engine::transaction::Resource::from_str(&format!("{},{}", 1, badge_resource_def)).unwrap(), account1)
        .call_method_with_all_resources(account2, "deposit_batch")
        .build(vec![key1])
        .unwrap();
    executor.run(send_badge).unwrap();

    let component = receipt1.component(0).unwrap();
    // Accept from account1
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(component, "approve", vec![format!("1,{:?}", badge_resource_def)], Some(account1))
        .build(vec![key1])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());

    // Accept from account2
    let transaction3 = TransactionBuilder::new(&executor)
        .call_method(component, "approve", vec![format!("1,{:?}", badge_resource_def)], Some(account2))
        .build(vec![key2])
        .unwrap();
    let receipt3 = executor.run(transaction3).unwrap();
    println!("{:?}\n", receipt3);
    assert!(receipt3.result.is_ok());
}
