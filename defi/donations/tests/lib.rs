use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_donations() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("donations")).unwrap();
    let xrd = "030000000000000000000000000000000000000000000000000004";

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "Donations", "new", vec![String::from("10")], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    let component = receipt1.component(0).unwrap();

    // Test the `make_badge` method.
    let make_badge_args = vec![account.to_string(), "Test ID".to_string(), "Test Title".to_string(), "Test Desription".to_string(), "Test URL".to_string(), String::from("10"), String::from("10")];
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(component, "make_badge", make_badge_args, Some(account))
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());

    // Test the `make_badge` method.
    let transaction3 = TransactionBuilder::new(&executor)
        .call_method(component, "get_badges", vec![account.to_string()], Some(account))
        .build(vec![key])
        .unwrap();
    let mut receipt3 = executor.run(transaction3).unwrap();
    println!("{:?}\n", receipt3);
    assert!(receipt3.result.is_ok());

    // Test the `donate` method.
    
    let badge_address: Vec<Address> =  scrypto_decode(&receipt3.outputs[0].raw[..]).unwrap();
    let transaction4 = TransactionBuilder::new(&executor)
        .call_method(component, "donate", vec![account.to_string(), badge_address[0].to_string(), format!("{},{}", 50, xrd)], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt4 = executor.run(transaction4).unwrap();
    println!("{:?}\n", receipt4);
    assert!(receipt4.result.is_ok());
}