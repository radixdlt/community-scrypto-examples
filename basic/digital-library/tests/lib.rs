use radix_engine::ledger::*;
use radix_engine::model::Bucket;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_new() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);

    let (key, sk, account) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();

    let args = args![10u32, dec!("1"), 3u64];
    let receipt = executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_function(package, "Library", "new", args)
                .call_method_with_all_resources(account, "deposit_batch")
                .build(executor.get_nonce([key]))
                .sign([&sk])
        )
        .unwrap();
    assert!(receipt.result.is_ok());

    assert_eq!(receipt.new_component_addresses.len(), 1);
    assert_eq!(receipt.new_resource_addresses.len(), 2);
}

#[test]
fn test_print_library() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);

    let (key, sk, account) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();
    let args = args![10u32, dec!("1"), 3u64];
    let receipt = executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_function(package, "Library", "new", args)
                .call_method_with_all_resources(account, "deposit_batch")
                .build(executor.get_nonce([key]))
                .sign([&sk])
        )
        .unwrap();
    let lib = receipt.new_component_addresses.get(0).unwrap();

    let print_library_transaction = TransactionBuilder::new()
        .call_method(*lib, "print_library", args![])
        .build(executor.get_nonce([key]))
        .sign([&sk]);
    let print_library_receipt = executor.validate_and_execute(&print_library_transaction).unwrap();
    assert!(print_library_receipt.result.is_ok());
    assert_eq!(print_library_receipt.logs.len(), 7);
}

#[test]
fn test_register() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);

    let (key, sk, account) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();
    let args = args![10u32, dec!("1"), 3u64];
    let receipt = executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_function(package, "Library", "new", args)
                .call_method_with_all_resources(account, "deposit_batch")
                .build(executor.get_nonce([key]))
                .sign([&sk])
        )
        .unwrap();
        let lib = receipt.new_component_addresses.get(0).unwrap();

    let register_transaction = TransactionBuilder::new()
        .withdraw_from_account(RADIX_TOKEN, account)
        .take_from_worktop_by_amount(Decimal::one(), RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(*lib, "register", args![Bucket(bucket_id)])
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([key]))
        .sign([&sk]);
    let register_receipt = executor.validate_and_execute(&register_transaction).unwrap();
    assert!(register_receipt.result.is_ok());

    // check print method for 1 less membership badge
    let print_library_receipt = executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_method(*lib, "print_library", args![])
                .build(executor.get_nonce([key]))
                .sign([&sk])
        )
        .unwrap();
    assert!(print_library_receipt.result.is_ok());
    assert_eq!(
        print_library_receipt.logs[1].1,
        "Membership price: 1, memberships available: 9"
    );
}
