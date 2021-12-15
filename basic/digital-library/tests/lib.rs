use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_new() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);

    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!());

    let args = vec!["10".to_string(), "1".to_string(), "3".to_string()];
    let receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .call_function(package, "Library", "new", args, None)
                .deposit_all_buckets(account)
                .build(vec![key])
                .unwrap(),
            false,
        )
        .unwrap();
    assert!(receipt.success);

    let components: Vec<Address> = receipt
        .new_entities
        .iter()
        .filter(|a| matches!(a, Address::Component(_)))
        .map(Clone::clone)
        .collect();
    assert_eq!(components.len(), 1);

    let resources: Vec<Address> = receipt
        .new_entities
        .iter()
        .filter(|a| matches!(a, Address::ResourceDef(_)))
        .map(Clone::clone)
        .collect();
    assert_eq!(resources.len(), 2);
}

#[test]
fn test_print_library() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);

    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!());
    let args = vec!["10".to_string(), "1".to_string(), "3".to_string()];
    let receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .call_function(package, "Library", "new", args, None)
                .deposit_all_buckets(account)
                .build(vec![key])
                .unwrap(),
            false,
        )
        .unwrap();
    let lib = receipt.component(0).unwrap();

    let print_library_transaction = TransactionBuilder::new(&executor)
        .call_method(lib, "print_library", vec![], None)
        .build(vec![key])
        .unwrap();
    let print_library_receipt = executor.run(print_library_transaction, false).unwrap();
    assert!(print_library_receipt.success);
    assert_eq!(print_library_receipt.logs.len(), 7);
}

#[test]
fn test_register() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);

    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!());
    let args = vec!["10".to_string(), "1".to_string(), "3".to_string()];
    let receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .call_function(package, "Library", "new", args, None)
                .deposit_all_buckets(account)
                .build(vec![key])
                .unwrap(),
            false,
        )
        .unwrap();
    let lib = receipt.component(0).unwrap();

    let xrd = format!("1,{}", RADIX_TOKEN);
    let register_transaction = TransactionBuilder::new(&executor)
        .call_method(lib, "register", vec![xrd], Some(account))
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let register_receipt = executor.run(register_transaction, false).unwrap();
    assert!(register_receipt.success);

    // check print method for 1 less membership badge
    let print_library_receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .call_method(lib, "print_library", vec![], None)
                .build(vec![key])
                .unwrap(),
            false,
        )
        .unwrap();
    assert!(print_library_receipt.success);
    assert_eq!(
        print_library_receipt.logs[1].1,
        "Membership price: 1, memberships available: 9"
    );
}
