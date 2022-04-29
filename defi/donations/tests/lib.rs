use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_donations() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    
    let (pk, sk, account) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(package, "Donations", "new", args![dec!("10")])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    let component = receipt1.new_component_addresses[0];

    // Test the `make_badge` method.
    let make_badge_args = args![
        account, 
        "Test ID".to_string(), 
        "Test Title".to_string(), 
        "Test Desription".to_string(), 
        "Test URL".to_string(), 
        dec!("10"), 
        dec!("10")
    ];
    let transaction2 = TransactionBuilder::new()
        .call_method(component, "make_badge", make_badge_args)
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());

    // Test the `make_badge` method.
    let transaction3 = TransactionBuilder::new()
        .call_method(component, "get_badges", args![account])
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let mut receipt3 = executor.validate_and_execute(&transaction3).unwrap();
    println!("{:?}\n", receipt3);
    assert!(receipt3.result.is_ok());

    // Test the `donate` method.
    
    let badge_address: Vec<ResourceAddress> =  scrypto_decode(&receipt3.outputs[0].raw[..]).unwrap();
    let transaction4 = TransactionBuilder::new()
        .withdraw_from_account_by_amount(dec!("50"), RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(
                component, 
                "donate", 
                args![
                    account, 
                    badge_address[0], 
                    scrypto::resource::Bucket(bucket_id)
                ]
            )
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt4 = executor.validate_and_execute(&transaction4).unwrap();
    println!("{:?}\n", receipt4);
    assert!(receipt4.result.is_ok());
}