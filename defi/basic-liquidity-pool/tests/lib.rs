use radix_engine::ledger::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::model::FromPublicKey;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_hello() {
    // Setup the environment
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Create an account
    let (public_key, _private_key, account) = test_runner.new_allocated_account();
    let (public_key_1, _private_key_1, account1) = test_runner.new_allocated_account();
    let (public_key_2, _private_key_2, account2) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // output = await e($`resim call-function  ${pub_package} LiquidityPool new 0,${tokenXRD} "LPT" "LP_Token"`)

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .withdraw_from_account_by_amount(account, dec!("0"), RADIX_TOKEN)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_function(
                package_address,
                "LiquidityPool",
                "new",
                args!(Bucket(bucket_id), "LPT", "LP Token"),
            )

            //  builder.call_method(component, "buy_ticket", args!(Bucket(bucket_id)))
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    println!("{:?}\n", receipt);
    receipt.expect_commit_success();

    let component_liquidity_pool = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    let receipt1 = provide_liquidity(
        component_liquidity_pool,
        account1,
        public_key_1,
        dec!("100"),
        &mut test_runner,
    );
    println!("{:?}\n", receipt1);
    receipt1.expect_commit_success();

    let receipt2 = provide_liquidity(
        component_liquidity_pool,
        account2,
        public_key_2,
        dec!("100"),
        &mut test_runner,
    );

    println!("{:?}\n", receipt2);
    receipt2.expect_commit_success();

    // println!(
    //     "{:?}\n",

    assert_eq!(
        test_runner
            .get_component_resources(account1)
            .get(&RADIX_TOKEN)
            .expect("Some(900)"),
        &dec!("90")
    );

    // // Test the `free_token` method.
    // let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
    //     .call_method(component, "free_token", args!())
    //     .call_method(
    //         account_component,
    //         "deposit_batch",
    //         args!(Expression::entire_worktop()),
    //     )
    //     .build();
    // let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![NonFungibleAddress::from_public_key(&public_key)]);
    // println!("{:?}\n", receipt);
    // receipt.expect_commit_success();
}

fn provide_liquidity(
    component: ComponentAddress,
    account: ComponentAddress,
    public_key: EcdsaSecp256k1PublicKey,
    amount: Decimal,
    test_runner: &mut TestRunner<TypedInMemorySubstateStore>,
) -> radix_engine::transaction::TransactionReceipt {
    let manifest1 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .withdraw_from_account_by_amount(account, amount, RADIX_TOKEN)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(component, "add_liquidity", args!(Bucket(bucket_id)))
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();

    let receipt1 = test_runner.execute_manifest_ignoring_fee(
        manifest1,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt1
}
