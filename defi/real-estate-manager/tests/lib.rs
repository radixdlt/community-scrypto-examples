use radix_engine::ledger::*;
use scrypto::prelude::*;
use scrypto_unit::*;

fn setup_fixture<'a, L: SubstateStore>(env: &mut TestEnv<'a, L>) -> (User, User, ResourceDef) {
    let _root = env.create_user("root");
    env.acting_as("root");

    // magic some reserve tokens into existance
    let reserve_def = env.create_token(dec!(2000000));

    // create an owner and an investor
    let owner = env.create_user("owner");
    let investor = env.create_user("investor");

    // transfer some reserve tokens to them to use in testing
    let receipt = env.transfer_resource(dec!(1000000), &reserve_def, &owner);
    println!("transfer 100k to owner: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());
    env.transfer_resource(dec!(1000000), &reserve_def, &investor);
    println!("transfer 100k to investor: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    // use a publisher to publish the package
    let _pubisher = env.create_user("publisher");
    env.acting_as("publisher");

    const PACKAGE: &str = "bonding";

    env.publish_package(PACKAGE, include_code!("scrypto_bonding"));
    env.using_package(PACKAGE);

    (owner, investor, reserve_def)
}

#[test]
fn test_hello() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let (pk, sk, account) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();

    // Test the `instantiate_hello` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(package, "Hello", "instantiate_hello", args![])
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `free_token` method.
    let component = receipt1.new_component_addresses[0];
    let transaction2 = TransactionBuilder::new()
        .call_method(component, "free_token", args![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}
