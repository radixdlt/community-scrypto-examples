use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_slot_machine() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(
            package,
            "SlotMachine",
            "new",
            args![
                dec!("100000")
            ],
        )
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `free_token` method.
    let casino = receipt1.new_component_addresses.get(0).unwrap();
    let vegas_token_ref = receipt1.new_resource_addresses.get(0).unwrap();
    let transaction2 = TransactionBuilder::new()
        .call_method(*casino, "free_token", args![])
        .assert_worktop_contains_by_amount(dec!("10"), *vegas_token_ref)
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());

    // Test the `play` method.
    let bet = Decimal::one();
    let transaction3 = TransactionBuilder::new()
        .withdraw_from_account_by_amount(bet, *vegas_token_ref, account)
        .take_from_worktop(*vegas_token_ref, |builder, bucket_id| {
            builder.call_method(*casino, "play", args![Bucket(bucket_id)])
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);

    let receipt3 = executor.validate_and_execute(&transaction3).unwrap();
    println!("{:?}\n", receipt3);
    assert!(receipt3.result.is_ok());
}