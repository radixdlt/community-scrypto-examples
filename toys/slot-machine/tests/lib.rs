use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_slot_machine() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("slot_machine"));

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "SlotMachine",
            "new",
            vec![
                "100000".to_owned()
            ],
            None
        )
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1, false).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.success);

    // Test the `free_token` method.
    let component = receipt1.component(0).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(component, "free_token", vec![], Some(account))
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2, true).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.success);

    // Test the `play` method.
    let vegas_token_id = receipt1.resource_def(0).unwrap();
    let bet = 1;
    let transaction3 = TransactionBuilder::new(&executor)
        .call_method(
            component,
            "play",
            vec![
                format!("{},{}", bet, vegas_token_id).to_owned()
            ],
            Some(account))
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt3 = executor.run(transaction3, false).unwrap();
    println!("{:?}\n", receipt3);
    assert!(receipt3.success);
}
