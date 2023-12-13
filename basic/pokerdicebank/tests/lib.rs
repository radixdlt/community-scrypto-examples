use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_pokerdicebank() {
    // Setup the environment
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();

    // Create an account
    let (public_key, _private_key, account) = test_runner.new_allocated_account();

    // create an other coin
    let new_fungible_token = test_runner.create_fungible_resource(dec!(5000), DIVISIBILITY_NONE, account);

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Test the `instantiate` function.
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "PokerDiceBank",
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
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("instantiate"),
        &NetworkDefinition::simulator(),
    ).err();

    let receipt1 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt1);

    let component = receipt1.expect_commit(true).new_component_addresses()[0];
    let proofbadge = receipt1.expect_commit(true).new_resource_addresses()[0];
    let playerbadge = receipt1.expect_commit(true).new_resource_addresses()[1];

    // Test the `admin_deposit_primairy' method.
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account, XRD, 1000)
        .take_from_worktop(XRD, 1000, "bucket")
        .create_proof_from_account_of_amount(account, 
            proofbadge, 1)
        .call_method_with_name_lookup(component, "admin_deposit_primairy",
        |lookup|( lookup.bucket("bucket"),))
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("admin_deposit_primairy"),
        &NetworkDefinition::simulator(),
    ).err();
    

    let receipt2 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt2);
    receipt2.expect_commit_success();


    // Test the `admin_deposit_secondairy' method.
    let manifest = ManifestBuilder::new()
    .withdraw_from_account(account, new_fungible_token, 2500)
    .take_from_worktop(new_fungible_token, 2500, "bucket")
    .create_proof_from_account_of_amount(account, 
        proofbadge, 1)
    .call_method_with_name_lookup(component, "admin_deposit_secondairy",
    |lookup|( lookup.bucket("bucket"),))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("admin_deposit_secondairy"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt3 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt3);
    receipt3.expect_commit_success();

    // Test the `admin_swap_config' method.
    let manifest = ManifestBuilder::new()
    .create_proof_from_account_of_amount(account, 
        proofbadge, 1)
    .call_method(component, "admin_swap_config",
            (new_fungible_token, dec!("50"),))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("admin_swap_config"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt4 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt4);
    receipt4.expect_commit_success();

    // Test the `admin_set_active_token' method.
    let manifest = ManifestBuilder::new()
    .create_proof_from_account_of_amount(account, 
        proofbadge, 1)
    .call_method(component, "admin_set_active_token",
    manifest_args!(new_fungible_token))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("admin_set_active_token"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt5_2 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt5_2);
    receipt5_2.expect_commit_success();
        
    // Test the `admin_global_swap_activate' method.
    let manifest = ManifestBuilder::new()
    .create_proof_from_account_of_amount(account, 
        proofbadge, 1)
    .call_method(component, "admin_global_swap_activate",
    manifest_args!(true))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("admin_global_swap_activate"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt5 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt5);
    receipt5.expect_commit_success();
    
    // Test the `admin_token_swap_activate' method.
    let manifest = ManifestBuilder::new()
    .create_proof_from_account_of_amount(account, 
        proofbadge, 1)
    .call_method(component, "admin_token_swap_activate",
    manifest_args!(new_fungible_token, true))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("admin_token_swap_activate"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt5_1 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt5_1);
    receipt5_1.expect_commit_success();

    // Test the `get_tokens' method.
    let manifest = ManifestBuilder::new()
    .withdraw_from_account(account, XRD, 1)
    .take_from_worktop(XRD, 1, "bucket")
    .call_method_with_name_lookup(component, "get_tokens",
    |lookup|( lookup.bucket("bucket"),))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("get_tokens"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt6 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt6);
    receipt6.expect_commit_success();


    // Test the `redeem_tokens' method.
    let manifest = ManifestBuilder::new()
    .withdraw_from_account(account, new_fungible_token, 19)
    .take_from_worktop(new_fungible_token, 19, "bucket")
    .call_method_with_name_lookup(component, "redeem_tokens",
    |lookup|( lookup.bucket("bucket"),))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("redeem_tokens"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt7 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt7);
    receipt7.expect_commit_success();

    // Test the `admin_pokerdicebank_activate' method.
    let manifest = ManifestBuilder::new()
    .create_proof_from_account_of_amount(account, 
        proofbadge, 1)
    .call_method(component, "admin_pokerdicebank_activate",
    manifest_args!(true))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("admin_pokerdicebank_activate"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt8 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt8);
    receipt8.expect_commit_success();


    // Test the `deposit_new' method.
    let manifest = ManifestBuilder::new()
    .withdraw_from_account(account, new_fungible_token, 100)
    .take_from_worktop(new_fungible_token, 100, "bucket")
    .call_method_with_name_lookup(component, "deposit_new",
    |lookup|( lookup.bucket("bucket"),))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("deposit_new"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt9 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt9);
    receipt9.expect_commit_success();

    // Test the `deposit_add' method.
    let manifest = ManifestBuilder::new()
    .withdraw_from_account(account, new_fungible_token, 75)
    .take_from_worktop(new_fungible_token, 75, "bucket")
    .create_proof_from_account_of_non_fungibles(
        account,
        playerbadge,
        btreeset!(NonFungibleLocalId::integer(1))
    )
    .create_proof_from_auth_zone_of_non_fungibles(
        playerbadge,
        btreeset!(NonFungibleLocalId::integer(1)),
        "proof"
    )
    .call_method_with_name_lookup(component, "deposit_add",
    |lookup|( lookup.bucket("bucket"), lookup.proof("proof"),))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("deposit_add"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt10 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt10);
    receipt10.expect_commit_success();

    // Test the `admin_update_keystore' method.
    let manifest = ManifestBuilder::new()
    .create_proof_from_account_of_amount(account, 
        proofbadge, 1)
    .call_method(component, "admin_update_keystore",
    manifest_args!(NonFungibleLocalId::integer(1), dec!(275)))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
        "./manifests/",
        Some("admin_update_keystore"),
        &NetworkDefinition::simulator(),
    ).err();
    
    let receipt11 = test_runner.execute_manifest_ignoring_fee(
        buildmanifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt11);
    receipt11.expect_commit_success();


    /*
    // Test the `set_redeem_pair' method.
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account, 
            proofbadge, 1)
        .call_method(component, "set_redeem_pair",
            (XRD, XRD, dec!("0.99"),),
        );
    
    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
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
        .withdraw_from_account(account, XRD, 100)
        .take_from_worktop(XRD, 99, "bucket")
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
        objectnames,
        &buildmanifest,
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
            (XRD,));

    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
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
        (XRD,))
    .call_method(
        account,
        "deposit_batch",
        manifest_args!(ManifestExpression::EntireWorktop),
    );
    
    let objectnames = manifest.object_names();
    let buildmanifest = &manifest.build();
    
    dump_manifest_to_file_system(
        objectnames,
        &buildmanifest,
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
 */
}
