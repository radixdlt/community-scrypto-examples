use radix_engine::ledger::*;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_slot_machine() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("slot_machine")).unwrap();

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
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `free_token` method.
    let casino = receipt1.component(0).unwrap();
    let vegas_token_ref = receipt1.resource_def(0).unwrap();
    let amount = fungible_amount(10, vegas_token_ref);
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(casino, "free_token", vec![], Some(account))
        .assert_worktop_contains(amount.amount().unwrap(), amount.resource_address())
        .take_from_worktop(&amount, |builder, bid| {
            builder
                .add_instruction(Instruction::CallMethod {
                    component_address: account,
                    method: "deposit".to_owned(),
                    args: vec![scrypto_encode(&bid)],
                })
                .0
        })
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());

    // Test the `play` method.
    let bet = Decimal::one();
    let winnigs = fungible_amount(2, vegas_token_ref);
    let transaction3 = TransactionBuilder::new(&executor)
        .call_method(
            casino,
            "play",
            vec![
                format!("{},{}", bet, vegas_token_ref).to_owned()
            ],
            Some(account))
        .take_from_worktop(&winnigs, |builder, winnigs| {
            builder
                .add_instruction(Instruction::CallMethod {
                    component_address: account,
                    method: "deposit".to_owned(),
                    args: vec![scrypto_encode(&winnigs)],
                })
                .0
        })
        .build(vec![key])
        .unwrap();
    let receipt3 = executor.run(transaction3).unwrap();
    println!("{:?}\n", receipt3);
    assert!(receipt3.result.is_ok());
}

fn fungible_amount(amount: i32, resource_address: Address) -> Resource {
    Resource::Fungible {
        amount: Decimal::from(amount),
        resource_address: resource_address,
    }
}