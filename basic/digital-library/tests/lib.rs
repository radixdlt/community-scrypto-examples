use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_new() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);

    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("library")).unwrap();

    let args = vec!["10".to_string(), "1".to_string(), "3".to_string()];
    let receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .call_function(package, "Library", "new", args, None)
                .call_method_with_all_resources(account, "deposit_batch")
                .build(vec![key])
                .unwrap()
        )
        .unwrap();
    assert!(receipt.result.is_ok());

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
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);

    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("library")).unwrap();
    let args = vec!["10".to_string(), "1".to_string(), "3".to_string()];
    let receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .call_function(package, "Library", "new", args, None)
                .call_method_with_all_resources(account, "deposit_batch")
                .build(vec![key])
                .unwrap()
        )
        .unwrap();
    let lib = receipt.component(0).unwrap();

    let print_library_transaction = TransactionBuilder::new(&executor)
        .call_method(lib, "print_library", vec![], None)
        .build(vec![key])
        .unwrap();
    let print_library_receipt = executor.run(print_library_transaction).unwrap();
    assert!(print_library_receipt.result.is_ok());
    assert_eq!(print_library_receipt.logs.len(), 7);
}

#[test]
fn test_register() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);

    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("library")).unwrap();
    let args = vec!["10".to_string(), "1".to_string(), "3".to_string()];
    let receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .call_function(package, "Library", "new", args, None)
                .call_method_with_all_resources(account, "deposit_batch")
                .build(vec![key])
                .unwrap()
        )
        .unwrap();
    let lib = receipt.component(0).unwrap();

    let xrd = format!("1,{}", RADIX_TOKEN);
    let register_transaction = TransactionBuilder::new(&executor)
        .call_method(lib, "register", vec![xrd], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let register_receipt = executor.run(register_transaction).unwrap();
    assert!(register_receipt.result.is_ok());

    // check print method for 1 less membership badge
    let print_library_receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .call_method(lib, "print_library", vec![], None)
                .build(vec![key])
                .unwrap()
        )
        .unwrap();
    assert!(print_library_receipt.result.is_ok());
    assert_eq!(
        print_library_receipt.logs[1].1,
        "Membership price: 1, memberships available: 9"
    );
}
