use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_accumulating_vault() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("scryptoAccumulator")).unwrap();
    
    executor.ledger_mut().set_epoch(0);

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "AccumulatingVault", "new", vec!["1.0".to_string()], None)
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `withdraw` method with 0.
    let component = receipt1.component(0).unwrap();
    let badge = receipt1.resource_def(1).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(component, "withdraw", vec!["0.0".to_string(), format!("1,{}", badge)], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
    
    // Test the `withdraw` method with 5.
    let transaction3 = TransactionBuilder::new(&executor)
        .call_method(component, "withdraw", vec!["5.0".to_string(), format!("1,{}", badge)], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt3 = executor.run(transaction3).unwrap();
    println!("{:?}\n", receipt3);
    assert!(!receipt3.result.is_ok());
    
    executor.ledger_mut().set_epoch(10);
    
    // Test the `withdraw` method with 5.
    let transaction4 = TransactionBuilder::new(&executor)
        .call_method(component, "withdraw", vec!["5.0".to_string(), format!("1,{}", badge)], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt4 = executor.run(transaction4).unwrap();
    println!("{:?}\n", receipt4);
    assert!(receipt4.result.is_ok());
}
