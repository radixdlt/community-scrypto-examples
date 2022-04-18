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
    let admin_badge_resource_address: ResourceAddress = get_resource_address_with_name(
        instantiation_receipt.new_resource_addresses, 
        String::from("Admin Badge"), 
        executor.substate_store()
    ).unwrap();

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
    let admin_badge_resource_address: ResourceAddress = get_resource_address_with_name(
        instantiation_receipt.new_resource_addresses.clone(), 
        String::from("Admin Badge"), 
        executor.substate_store()
    ).unwrap();
    let shareholder_badge_resource_address: ResourceAddress = get_resource_address_with_name(
        instantiation_receipt.new_resource_addresses.clone(), 
        String::from("Shareholder Badge"), 
        executor.substate_store()
    ).unwrap();

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
    let admin_badge_resource_address: ResourceAddress = get_resource_address_with_name(
        instantiation_receipt.new_resource_addresses, 
        String::from("Admin Badge"), 
        executor.substate_store()
    ).unwrap();

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

// /// ####################################################################################################################
// /// The resource addresses that we get in `receipt.new_resource_addresses` are not sorted in any way. So to get the 
// /// correct resource address for most operations we need to search for it using its metadata or through other means.
// /// ####################################################################################################################

#[derive(Debug, Clone)]
pub enum ResourceNameError {
    ResourceManagerNotFound,
    ResourceHasNoName,
}

/// Gets the name of a resource 
/// 
/// This function loads in the resource manager and the of a given resource address from the substate store and then 
/// tries to find the name of the resource from the metadata. If the resource does not exist or if the resource has no
/// name then a `ResourceNameError` is returned.
/// 
/// # Arguments:
/// 
/// * `resource_address` (ResourceAddress) - The address of the resource
/// * `substate_store` (T: SubstateStore) - A generic type representing a substate store.
/// 
/// # Returns:
///    
/// This function returns a `String` on the successful retrieval of the resource name and a `ResourceNameError` on
/// failure.
pub fn get_resource_name<T: SubstateStore>(
    resource_address: &ResourceAddress,
    substate_store: &T
) -> Result<String, ResourceNameError> {
    // Getting the resource manager from the substate
    let resource_manager: Option<radix_engine::model::ResourceManager> = substate_store
        .get_decoded_substate(resource_address)
        .map(|(resource, _)| resource);

    // Checking if a resource manager exists, if it does then we can try to attempt to find the name, else we return
    // an error
    match resource_manager {
        Some(rm) => {
            // Check the metadata to see if it contains a name
            match rm.metadata().get("name") {
                Some(name) => Ok(name.clone()),
                None => Err(ResourceNameError::ResourceHasNoName)
            }
        },
        None => Err(ResourceNameError::ResourceManagerNotFound)
    }
}

/// Returns the ResourceAddress of a resource which has a specific name.
/// 
/// This function takes in a vector of `ResourceAddress` objects and attempts to find the address of the resource which
/// has a the name `name` in the `substate_store`. If no resource is found with that given name then a `None` is 
/// returned.
/// 
/// # Arguments:
/// 
/// * `resource_addresses` (Vec<ResourceAddresss>) - A vector of the resource addresses to find the appropriate one in
/// * `name` (String) - The name that we're looking for.
/// * * `substate_store` (T: SubstateStore) - A generic type representing a substate store.
/// 
/// # Returns:
/// 
/// This function returns a resource address if one is found that has the desired name, otherwise, this function returns
/// a `None`
pub fn get_resource_address_with_name<T: SubstateStore>(
    resource_addresses: Vec<ResourceAddress>,
    name: String,
    substate_store: &T
) -> Option<ResourceAddress> {
    for ra in resource_addresses.iter() {
        match get_resource_name(ra, substate_store) {
            Ok(resource_name) => {
                if name == resource_name {
                    return Some(*ra)
                }
            },
            Err(_) => {}
        }
    }
    return None;
}
