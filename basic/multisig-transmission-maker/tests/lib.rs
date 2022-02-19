use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_multisig() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);

    let key1 = executor.new_public_key();
    let account1 = executor.new_account(key1);
    let key2 = executor.new_public_key();
    let account2 = executor.new_account(key2);
    let key3 = executor.new_public_key();
    let account3 = executor.new_account(key3);

    let package = executor.publish_package(include_code!("multisig_transmission_maker"));

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "MultiSigMaker", "new", vec!["2".to_string(), "2".to_string(), format!("{}", account3), format!("1000,{}", RADIX_TOKEN)], Some(account1))
        .deposit_all_buckets(account1)
        .build(vec![key1])
        .unwrap();
    let receipt1 = executor.run(transaction1, false).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.success);

    let badge_resource_def = receipt1.resource_def(1).unwrap();

    // Send one badge to account 2
    let send_badge = TransactionBuilder::new(&executor)
        .call_method(account1, "withdraw", vec!["1".to_string(), format!("{}", badge_resource_def)], Some(account1))
        .deposit_all_buckets(account2)
        .build(vec![key1])
        .unwrap();
    executor.run(send_badge, false).unwrap();

    let component = receipt1.component(0).unwrap();
    // Accept from account1
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(component, "approve", vec![format!("1,{:?}", badge_resource_def)], Some(account1))
        .build(vec![key1])
        .unwrap();
    let receipt2 = executor.run(transaction2, false).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.success);

    // Accept from account2
    let transaction3 = TransactionBuilder::new(&executor)
        .call_method(component, "approve", vec![format!("1,{:?}", badge_resource_def)], Some(account2))
        .build(vec![key2])
        .unwrap();
    let receipt3 = executor.run(transaction3, false).unwrap();
    println!("{:?}\n", receipt3);
    assert!(receipt3.success);
}
