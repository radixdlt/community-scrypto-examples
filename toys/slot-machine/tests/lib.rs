use radix_engine::ledger::*;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::engine::types::*;
use scrypto::*;

#[test]
fn test_slot_machine() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (public_key, private_key, account_component_address) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();

    // Test the `new` function.
    let initial_amount = dec!("100000");
    let transaction1: SignedTransaction = TransactionBuilder::new()
        .call_function(
            package,
            "SlotMachine",
            "new",
            args![initial_amount]
        )
        .build(executor.get_nonce([public_key]))
        .sign([&private_key]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `free_token` method.
    let casino = receipt1.new_component_addresses[0];
    let vegas_token_ref = receipt1.new_resource_addresses[0];
    let transaction2 = TransactionBuilder::new()
        .call_method(casino, "free_token", args![])
        .assert_worktop_contains(vegas_token_ref)
        .call_method_with_all_resources(account_component_address, "deposit_batch")
        .build(executor.get_nonce([public_key]))
        .sign([&private_key]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());

    // Test the `play` method.
    let bet_amount = Decimal::one();
    let transaction3 = TransactionBuilder::new()
        .withdraw_from_account_by_amount(bet_amount, vegas_token_ref, account_component_address)
        .take_from_worktop(vegas_token_ref, |builder, bucket_id|{
            builder.call_method(
                casino,
                "play",
                args![scrypto::resource::Bucket(bucket_id)]
            )
        })
        .call_method_with_all_resources(account_component_address, "deposit_batch")
        .build(executor.get_nonce([public_key]))
        .sign([&private_key]);
    let receipt3 = executor.validate_and_execute(&transaction3).unwrap();
    println!("{:?}\n", receipt3);
    assert!(receipt3.result.is_ok());
}