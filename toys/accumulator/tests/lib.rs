use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_accumulating_vault() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let (pk, sk, account) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();
    
    executor.substate_store_mut().set_epoch(0);

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(package, "AccumulatingVault", "new", args![Decimal::one()])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce(vec![pk]))
        .sign([&sk]);

    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `withdraw` method with 0.
    let component = receipt1.new_component_addresses.get(0).unwrap();
    let badge = receipt1.new_resource_addresses.get(1).unwrap();
    let transaction2 = TransactionBuilder::new()
        .withdraw_from_account(*badge, account)
        .take_from_worktop(*badge, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.push_to_auth_zone(proof_id)
            })
        })
        .call_method(*component, "withdraw", args![Decimal::zero()])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce(vec![pk]))
        .sign([&sk]);

    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
    
    // Test the `withdraw` method with 5.
    let transaction3 = TransactionBuilder::new()
        .withdraw_from_account(*badge, account)
        .take_from_worktop(*badge, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.push_to_auth_zone(proof_id)
            })
        })
        .call_method(*component, "withdraw", args![dec!("5.0")])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce(vec![pk]))
        .sign([&sk]);

    let receipt3 = executor.validate_and_execute(&transaction3).unwrap();
    println!("{:?}\n", receipt3);
    assert!(!receipt3.result.is_ok());
    
    executor.substate_store_mut().set_epoch(10);
    
    // Test the `withdraw` method with 5.
    let transaction4 = TransactionBuilder::new()
        .withdraw_from_account(*badge, account)
        .take_from_worktop(*badge, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.push_to_auth_zone(proof_id)
            })
        })
        .call_method(*component, "withdraw", args![dec!("5.0")])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt4 = executor.validate_and_execute(&transaction4).unwrap();
    println!("{:?}\n", receipt4);
    assert!(receipt4.result.is_ok());
}
