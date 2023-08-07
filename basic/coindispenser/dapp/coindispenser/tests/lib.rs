use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_coindispenser() {
    // Setup the environment
    let mut test_runner = TestRunner::builder().build();

    // Create an account
    let (public_key, _private_key, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "CoinDispenser",
            "instantiate",
            manifest_args!()
        )
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop)
        );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();

    dump_manifest_to_file_system(
        &buildmanifest,
        objectnames,
        "./",
        Some("instantiate"),
        &NetworkDefinition::simulator(),
    ).err();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    let component = receipt.expect_commit(true).new_component_addresses()[0];
    let proofbadge = receipt.expect_commit(true).new_resource_addresses()[0];

    // Test the `deposit' method.
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account, RADIX_TOKEN, 100)
        .take_from_worktop(RADIX_TOKEN, 99, "bucket")
        .call_method_with_name_lookup(component, "deposit",
        |lookup|( lookup.bucket("bucket"),))
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        &buildmanifest,
        objectnames,
        "./",
        Some("deposit"),
        &NetworkDefinition::simulator(),
    ).err();
    

    let receipt = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();



    // Test the `set_redeem_pair' method.
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account, 
            proofbadge, 1)
        .call_method(component, "set_redeem_pair",
            (RADIX_TOKEN, RADIX_TOKEN, dec!("0.99"),),
        );
    
    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        &buildmanifest,
        objectnames,
        "./",
        Some("set_redeem_pair"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();

    // Test the `redeem_coin' method.
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account, RADIX_TOKEN, 100)
        .take_from_worktop(RADIX_TOKEN, 99, "bucket")
        .call_method_with_name_lookup(component, "redeem_coin",
        |lookup|( lookup.bucket("bucket"),))
        .call_method(
           account,
           "deposit_batch",
           manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        &buildmanifest,
        objectnames,
        "./",
        Some("redeem_coin"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();

    // Test the `keystore_swap' method.
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account, 
            proofbadge, 1)
        .call_method(component, "keystore_swap",
            (RADIX_TOKEN,));

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        &buildmanifest,
        objectnames,
        "./",
        Some("keystore_swap"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();


    // Test the `withdrawal' method.
    let manifest = ManifestBuilder::new()
    .create_proof_from_account_of_amount(account, 
        proofbadge, 1)
    .call_method(component, "withdrawal",
        (RADIX_TOKEN,))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );
    
    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        &buildmanifest,
        objectnames,
        "./",
        Some("withdrawal"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();

}
