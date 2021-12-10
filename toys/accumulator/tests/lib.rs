use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_accumulating_vault() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!());
    
    executor.set_current_epoch(0);

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "AccumulatingVault", "new", vec!["1.0".to_string()], None)
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1, false).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.success);

    // Test the `withdraw` method with 0.
    let component = receipt1.component(0).unwrap();
    let badge = receipt1.resource_def(1).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(component, "withdraw", vec!["0.0".to_string(), format!("1,{}", badge)], Some(account))
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2, false).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.success);
    
    // Test the `withdraw` method with 5.
    let transaction3 = TransactionBuilder::new(&executor)
        .call_method(component, "withdraw", vec!["5.0".to_string(), format!("1,{}", badge)], Some(account))
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt3 = executor.run(transaction3, false).unwrap();
    println!("{:?}\n", receipt3);
    assert!(!receipt3.success);
    
    executor.set_current_epoch(10);
    
    // Test the `withdraw` method with 5.
    let transaction4 = TransactionBuilder::new(&executor)
        .call_method(component, "withdraw", vec!["5.0".to_string(), format!("1,{}", badge)], Some(account))
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt4 = executor.run(transaction4, false).unwrap();
    println!("{:?}\n", receipt4);
    assert!(receipt4.success);
}
