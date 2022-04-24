use scrypto::crypto::{EcdsaPrivateKey, EcdsaPublicKey};
use radix_engine::model::{SignedTransaction, Receipt};
use radix_engine::transaction::*;
use radix_engine::ledger::*;
use radix_engine::errors::*;
use scrypto::prelude::*;

#[test]
fn admin_can_add_shareholder() {
    // Setting up the ledger and executor for this test
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the PaymentSplitter package
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (
        admin_public_key, 
        admin_private_key, 
        admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();
    let (
        _non_admin_public_key, 
        _non_admin_private_key, 
        non_admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();

    // Creating a new payment splitter component
    let instantiation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "PaymentSplitter", "instantiate_payment_splitter", vec![scrypto_encode(&RADIX_TOKEN)])
        .call_method_with_all_resources(admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let instantiation_receipt: Receipt = executor.validate_and_execute(&instantiation_tx).unwrap();

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt.new_component_addresses[0];
    let admin_badge_resource_address: ResourceAddress = instantiation_receipt.new_resource_addresses[0];

    // Attempting to add a new shareholder to the payment splitter
    let adding_shareholder_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account_by_amount(dec!("1"), admin_badge_resource_address, admin_component_address)
        .call_method(payment_splitter_component_address, "add_shareholder", vec![scrypto_encode(&dec!("20"))])
        .call_method_with_all_resources(non_admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let adding_shareholder_receipt: Receipt = executor.validate_and_execute(&adding_shareholder_tx).unwrap();

    assert!(adding_shareholder_receipt.result.is_ok());
}

#[test]
fn shareholder_cant_add_shareholder() {
    // Setting up the ledger and executor for this test
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the PaymentSplitter package
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (
        admin_public_key, 
        admin_private_key, 
        admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();
    let (
        non_admin_public_key, 
        non_admin_private_key, 
        non_admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();

    // Creating a new payment splitter component
    let instantiation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "PaymentSplitter", "instantiate_payment_splitter", vec![scrypto_encode(&RADIX_TOKEN)])
        .call_method_with_all_resources(admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let instantiation_receipt: Receipt = executor.validate_and_execute(&instantiation_tx).unwrap();

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt.new_component_addresses[0];
    let admin_badge_resource_address: ResourceAddress = instantiation_receipt.new_resource_addresses[0];
    let shareholder_badge_resource_address: ResourceAddress = instantiation_receipt.new_resource_addresses[2];

    // Attempting to add a new shareholder to the payment splitter
    let adding_shareholder_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account_by_amount(dec!("1"), admin_badge_resource_address, admin_component_address)
        .call_method(payment_splitter_component_address, "add_shareholder", vec![scrypto_encode(&dec!("20"))])
        .call_method_with_all_resources(non_admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let adding_shareholder_receipt: Receipt = executor.validate_and_execute(&adding_shareholder_tx).unwrap();
    assert!(adding_shareholder_receipt.result.is_ok());

    // Attempting to add a new shareholder to the payment splitter
    let unauthed_adding_shareholder_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account_by_amount(dec!("1"), shareholder_badge_resource_address, non_admin_component_address)
        .call_method(payment_splitter_component_address, "add_shareholder", vec![scrypto_encode(&dec!("999999"))])
        .call_method_with_all_resources(non_admin_component_address, "deposit_batch")
        .build(executor.get_nonce([non_admin_public_key]))
        .sign([&non_admin_private_key]);
    let unauthed_adding_shareholder_receipt: Receipt = executor.validate_and_execute(&unauthed_adding_shareholder_tx).unwrap();

    // We know that we should not be authorized to add them; so, we check for an AuthorizationError
    let runtime_error: RuntimeError = unauthed_adding_shareholder_receipt.result.expect_err("Should Fail");
    match runtime_error {
        RuntimeError::AuthorizationError(_, _) => {},
        _ => panic!("Adding shareholder should not be allowed, error: {}", runtime_error)
    }
}

#[test]
fn unauthed_cant_lock_splitter() {
    // Setting up the ledger and executor for this test
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the PaymentSplitter package
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (
        admin_public_key, 
        admin_private_key, 
        admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();
    let (
        non_admin_public_key, 
        non_admin_private_key, 
        _non_admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();

    // Creating a new payment splitter component
    let instantiation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "PaymentSplitter", "instantiate_payment_splitter", vec![scrypto_encode(&RADIX_TOKEN)])
        .call_method_with_all_resources(admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let instantiation_receipt: Receipt = executor.validate_and_execute(&instantiation_tx).unwrap();

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt.new_component_addresses[0];

    // Attempting to add a new shareholder to the payment splitter
    let unauthed_locking_tx: SignedTransaction = TransactionBuilder::new()
        .call_method(payment_splitter_component_address, "lock_splitter", args![])
        .build(executor.get_nonce([non_admin_public_key]))
        .sign([&non_admin_private_key]);
    let unauthed_locking_receipt: Receipt = executor.validate_and_execute(&unauthed_locking_tx).unwrap();

    // We know that we should not be authorized to add them; so, we check for an AuthorizationError
    let runtime_error: RuntimeError = unauthed_locking_receipt.result.expect_err("Should Fail");
    match runtime_error {
        RuntimeError::AuthorizationError(_, _) => {},
        _ => panic!("Adding shareholder should not be allowed, error: {}", runtime_error)
    }
}

#[test]
fn admin_cant_add_shareholder_after_locking() {
    // Setting up the ledger and executor for this test
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the PaymentSplitter package
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (
        admin_public_key, 
        admin_private_key, 
        admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();
    let (
        _non_admin_public_key, 
        _non_admin_private_key, 
        non_admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();

    // Creating a new payment splitter component
    let instantiation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "PaymentSplitter", "instantiate_payment_splitter", vec![scrypto_encode(&RADIX_TOKEN)])
        .call_method_with_all_resources(admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let instantiation_receipt: Receipt = executor.validate_and_execute(&instantiation_tx).unwrap();

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt.new_component_addresses[0];
    let admin_badge_resource_address: ResourceAddress = instantiation_receipt.new_resource_addresses[0];

    // Locking the payment splitter
    let locking_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account_by_amount(dec!("1"), admin_badge_resource_address, admin_component_address)
        .call_method(payment_splitter_component_address, "lock_splitter", args![])
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _locking_receipt: Receipt = executor.validate_and_execute(&locking_tx).unwrap();

    // Attempting to add a new shareholder to the payment splitter
    let adding_shareholder_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account_by_amount(dec!("1"), admin_badge_resource_address, admin_component_address)
        .call_method(payment_splitter_component_address, "add_shareholder", vec![scrypto_encode(&dec!("20"))])
        .call_method_with_all_resources(non_admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let adding_shareholder_receipt: Receipt = executor.validate_and_execute(&adding_shareholder_tx).unwrap();

    // Adding an additional shareholder should fail.
    assert!(!adding_shareholder_receipt.result.is_ok());
}

#[test]
fn anybody_can_deposit() {
    // Setting up the ledger and executor for this test
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the PaymentSplitter package
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (
        admin_public_key, 
        admin_private_key, 
        admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();
    let (
        non_admin_public_key, 
        non_admin_private_key, 
        non_admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();

    // Creating a new payment splitter component
    let instantiation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "PaymentSplitter", "instantiate_payment_splitter", vec![scrypto_encode(&RADIX_TOKEN)])
        .call_method_with_all_resources(admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let instantiation_receipt: Receipt = executor.validate_and_execute(&instantiation_tx).unwrap();

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt.new_component_addresses[0];

    // Depositing funds into the payment splitter
    let deposit_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account_by_amount(dec!("100000"), RADIX_TOKEN, non_admin_component_address)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(
                payment_splitter_component_address, 
                "deposit", 
                vec![scrypto_encode(&scrypto::resource::Bucket(bucket_id))],
            )
        })
        .call_method_with_all_resources(non_admin_component_address, "deposit_batch")
        .build(executor.get_nonce([non_admin_public_key]))
        .sign([&non_admin_private_key]);
    let deposit_receipt: Receipt = executor.validate_and_execute(&deposit_tx).unwrap();

    // Adding an additional shareholder should fail.
    assert!(deposit_receipt.result.is_ok());
}

#[test]
fn custom_rule_splitter_works_with_correct_badges() {
    // Setting up the ledger and executor for this test
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, true);

    // Publishing the PaymentSplitter package
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (
        admin_public_key, 
        admin_private_key, 
        admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();

    // Creating multiple badges to use for the splitter testing 
    let badge_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_badge_fixed(HashMap::new(), dec!("1"))
        .new_badge_fixed(HashMap::new(), dec!("1"))
        .new_badge_fixed(HashMap::new(), dec!("1"))
        .call_method_with_all_resources(admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let badge_creation_receipt: Receipt = executor.validate_and_execute(&badge_creation_tx).unwrap();

    let supervisor_badge_resource_address: ResourceAddress = badge_creation_receipt.new_resource_addresses[0];
    let admin_badge_resource_address: ResourceAddress = badge_creation_receipt.new_resource_addresses[1];
    let superadmin_badge_resource_address: ResourceAddress = badge_creation_receipt.new_resource_addresses[2];

    // Creating the access rule which we would like to use for the addition of shareholders
    let rule: AccessRule = rule!(
        require(supervisor_badge_resource_address) 
        && require(admin_badge_resource_address) 
        && require(superadmin_badge_resource_address)
    );

    let instantiation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(
            package, 
            "PaymentSplitter", 
            "instantiate_custom_access_payment_splitter", 
            args![
                RADIX_TOKEN,
                rule
            ]
        )
        .call_method_with_all_resources(admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let instantiation_receipt: Receipt = executor.validate_and_execute(&instantiation_tx).unwrap();

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt.new_component_addresses[0];

    // Attempting to add a new shareholder to the payment splitter
    let adding_shareholder_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account_by_amount(dec!("1"), admin_badge_resource_address, admin_component_address)
        .create_proof_from_account_by_amount(dec!("1"), superadmin_badge_resource_address, admin_component_address)
        .create_proof_from_account_by_amount(dec!("1"), supervisor_badge_resource_address, admin_component_address)
        .call_method(payment_splitter_component_address, "add_shareholder", args!(dec!("20")))
        .call_method_with_all_resources(admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let adding_shareholder_receipt: Receipt = executor.validate_and_execute(&adding_shareholder_tx).unwrap();
    println!("{:?}", adding_shareholder_receipt);

    // Adding an additional shareholder should fail.
    assert!(adding_shareholder_receipt.result.is_ok());
}

#[test]
fn custom_rule_splitter_doesnt_work_with_incorrect_badges() {
    // Setting up the ledger and executor for this test
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, true);

    // Publishing the PaymentSplitter package
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (
        admin_public_key, 
        admin_private_key, 
        admin_component_address
    ): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();

    // Creating multiple badges to use for the splitter testing 
    let badge_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_badge_fixed(HashMap::new(), dec!("1"))
        .new_badge_fixed(HashMap::new(), dec!("1"))
        .new_badge_fixed(HashMap::new(), dec!("1"))
        .call_method_with_all_resources(admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let badge_creation_receipt: Receipt = executor.validate_and_execute(&badge_creation_tx).unwrap();

    let supervisor_badge_resource_address: ResourceAddress = badge_creation_receipt.new_resource_addresses[0];
    let admin_badge_resource_address: ResourceAddress = badge_creation_receipt.new_resource_addresses[1];
    let superadmin_badge_resource_address: ResourceAddress = badge_creation_receipt.new_resource_addresses[2];

    // Creating the access rule which we would like to use for the addition of shareholders
    let rule: AccessRule = rule!(
        require(supervisor_badge_resource_address) 
        && require(admin_badge_resource_address) 
        && require(superadmin_badge_resource_address)
    );

    let instantiation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(
            package, 
            "PaymentSplitter", 
            "instantiate_custom_access_payment_splitter", 
            args![
                RADIX_TOKEN,
                rule
            ]
        )
        .call_method_with_all_resources(admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let instantiation_receipt: Receipt = executor.validate_and_execute(&instantiation_tx).unwrap();

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt.new_component_addresses[0];

    // Attempting to add a new shareholder to the payment splitter
    let adding_shareholder_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account_by_amount(dec!("1"), admin_badge_resource_address, admin_component_address)
        .create_proof_from_account_by_amount(dec!("1"), superadmin_badge_resource_address, admin_component_address)
        .call_method(payment_splitter_component_address, "add_shareholder", args!(dec!("20")))
        .call_method_with_all_resources(admin_component_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let adding_shareholder_receipt: Receipt = executor.validate_and_execute(&adding_shareholder_tx).unwrap();
    println!("{:?}", adding_shareholder_receipt);

    // Adding an additional shareholder should fail.
    assert!(adding_shareholder_receipt.result.is_err());
}