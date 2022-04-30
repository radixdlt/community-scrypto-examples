use radix_engine::errors::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::model::{Receipt, SignedTransaction};
use radix_engine::transaction::*;
use regex::Regex;
use scrypto::crypto::{EcdsaPrivateKey, EcdsaPublicKey};
use scrypto::engine::types::*;
use scrypto::prelude::*;

#[macro_export]
macro_rules! assert_auth_error {
    ($error:expr) => {{
        if !matches!(
            $error,
            RuntimeError::AuthorizationError {
                authorization: _,
                function: _,
                error: ::radix_engine::model::MethodAuthorizationError::NotAuthorized
            }
        ) {
            panic!("Expected auth error but got: {:?}", $error);
        }
    }};
}

#[test]
fn add_beneficiary_fails_without_admin_badge() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (_beneficiary_public_key, _beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();
    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    let error: RuntimeError = adding_beneficiary_receipt.result.expect_err("Should have failed");
    assert_auth_error!(error);
}

#[test]
fn add_beneficiary_succeeds_with_admin_badge() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (_beneficiary_public_key, _beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    assert!(adding_beneficiary_receipt.result.is_ok());
}

#[test]
fn terminate_beneficiary_fails_without_admin_badge() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (_beneficiary_public_key, _beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    // Terminating the beneficiary
    let terminating_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .call_method(
            vesting_component,
            "terminate_beneficiary",
            args![NonFungibleId::from_u64(1u64)],
        )
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let terminating_beneficiary_receipt: Receipt = executor.validate_and_execute(&terminating_beneficiary_tx).unwrap();

let error: RuntimeError = terminating_beneficiary_receipt.result.expect_err("Should have failed");
    assert_auth_error!(error);
}

#[test]
fn terminate_beneficiary_succeeds_with_admin_badge() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (_beneficiary_public_key, _beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    // Terminating the beneficiary
    let terminating_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .call_method(
            vesting_component,
            "terminate_beneficiary",
            args![NonFungibleId::from_u64(1u64)],
        )
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let terminating_beneficiary_receipt: Receipt = executor.validate_and_execute(&terminating_beneficiary_tx).unwrap();

    assert!(terminating_beneficiary_receipt.result.is_ok());
}

#[test]
fn withdraw_without_beneficiary_badge_fails() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (beneficiary_public_key, beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    // Withdrawing the funds without a valid badge
    let withdrawing_funds_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account(RADIX_TOKEN, beneficiary_address)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.call_method(
                    vesting_component,
                    "withdraw_funds",
                    args![scrypto::resource::Proof(proof_id)],
                )
            })
        })
        .call_method_with_all_resources(beneficiary_address, "deposit_batch")
        .build(executor.get_nonce([beneficiary_public_key]))
        .sign([&beneficiary_private_key]);
    let withdrawing_funds_receipt: Receipt = executor.validate_and_execute(&withdrawing_funds_tx).unwrap();

    assert!(withdrawing_funds_receipt.result.is_err());
}

#[test]
fn withdraw_with_valid_beneficiary_badge_succeeds() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (beneficiary_public_key, beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    // Withdrawing the funds without a valid badge
    let withdrawing_funds_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account(beneficiary_badge, beneficiary_address)
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.call_method(
                    vesting_component,
                    "withdraw_funds",
                    args![scrypto::resource::Proof(proof_id)],
                )
            })
        })
        .call_method_with_all_resources(beneficiary_address, "deposit_batch")
        .build(executor.get_nonce([beneficiary_public_key]))
        .sign([&beneficiary_private_key]);
    let withdrawing_funds_receipt: Receipt = executor.validate_and_execute(&withdrawing_funds_tx).unwrap();

    assert!(withdrawing_funds_receipt.result.is_ok());
}

#[test]
fn withdraw_with_terminated_beneficiary_badge_fails() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (beneficiary_public_key, beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    // Terminating the beneficiary
    let terminating_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .call_method(
            vesting_component,
            "terminate_beneficiary",
            args![NonFungibleId::from_u64(1u64)],
        )
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _terminating_beneficiary_receipt: Receipt = executor.validate_and_execute(&terminating_beneficiary_tx).unwrap();

    // Withdrawing the funds without a valid badge
    let withdrawing_funds_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account(beneficiary_badge, beneficiary_address)
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.call_method(
                    vesting_component,
                    "withdraw_funds",
                    args![scrypto::resource::Proof(proof_id)],
                )
            })
        })
        .call_method_with_all_resources(beneficiary_address, "deposit_batch")
        .build(executor.get_nonce([beneficiary_public_key]))
        .sign([&beneficiary_private_key]);
    let withdrawing_funds_receipt: Receipt = executor.validate_and_execute(&withdrawing_funds_tx).unwrap();

    assert!(withdrawing_funds_receipt.result.is_err());
}

#[test]
fn should_be_empty_withdraw_before_cliff() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (beneficiary_public_key, beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    // Moving the epoch a little bit so that it's before the cliff but not 0
    executor.substate_store_mut().set_epoch(10);

    // Withdrawing the funds without a valid badge
    let withdrawing_funds_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account(beneficiary_badge, beneficiary_address)
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.call_method(
                    vesting_component,
                    "withdraw_funds",
                    args![scrypto::resource::Proof(proof_id)],
                )
            })
        })
        .call_method_with_all_resources(beneficiary_address, "deposit_batch")
        .build(executor.get_nonce([beneficiary_public_key]))
        .sign([&beneficiary_private_key]);
    let withdrawing_funds_receipt: Receipt = executor.validate_and_execute(&withdrawing_funds_tx).unwrap();

    let re: Regex = Regex::new(r"\[Withdraw Funds\]: Withdraw successful\. Withdrawing ([\d\.]+) tokens").unwrap();
    let withdrew_amount: Decimal = Decimal::from(
        re.captures(withdrawing_funds_receipt.logs[0].1.as_str())
            .and_then(|cap| cap.get(1).map(|r| r.as_str()))
            .unwrap(),
    );

    assert_eq!(withdrew_amount, dec!("0"));
}

#[test]
fn should_withdraw_initial_on_cliff() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (beneficiary_public_key, beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    // Moving the epoch a little bit so that it's before the cliff but not 0
    executor.substate_store_mut().set_epoch(20);

    // Withdrawing the funds without a valid badge
    let withdrawing_funds_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account(beneficiary_badge, beneficiary_address)
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.call_method(
                    vesting_component,
                    "withdraw_funds",
                    args![scrypto::resource::Proof(proof_id)],
                )
            })
        })
        .call_method_with_all_resources(beneficiary_address, "deposit_batch")
        .build(executor.get_nonce([beneficiary_public_key]))
        .sign([&beneficiary_private_key]);
    let withdrawing_funds_receipt: Receipt = executor.validate_and_execute(&withdrawing_funds_tx).unwrap();

    let re: Regex = Regex::new(r"\[Withdraw Funds\]: Withdraw successful\. Withdrawing ([\d\.]+) tokens").unwrap();
    let withdrew_amount: Decimal = Decimal::from(
        re.captures(withdrawing_funds_receipt.logs[0].1.as_str())
            .and_then(|cap| cap.get(1).map(|r| r.as_str()))
            .unwrap(),
    );

    assert_eq!(withdrew_amount, dec!("200000"));
}

#[test]
fn should_withdraw_total_long_after_due() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (beneficiary_public_key, beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    // Moving the epoch a little bit so that it's before the cliff but not 0
    executor.substate_store_mut().set_epoch(500);

    // Withdrawing the funds without a valid badge
    let withdrawing_funds_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account(beneficiary_badge, beneficiary_address)
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.call_method(
                    vesting_component,
                    "withdraw_funds",
                    args![scrypto::resource::Proof(proof_id)],
                )
            })
        })
        .call_method_with_all_resources(beneficiary_address, "deposit_batch")
        .build(executor.get_nonce([beneficiary_public_key]))
        .sign([&beneficiary_private_key]);
    let withdrawing_funds_receipt: Receipt = executor.validate_and_execute(&withdrawing_funds_tx).unwrap();

    let re: Regex = Regex::new(r"\[Withdraw Funds\]: Withdraw successful\. Withdrawing ([\d\.]+) tokens").unwrap();
    let withdrew_amount: Decimal = Decimal::from(
        re.captures(withdrawing_funds_receipt.logs[0].1.as_str())
            .and_then(|cap| cap.get(1).map(|r| r.as_str()))
            .unwrap(),
    );

    assert_eq!(withdrew_amount, Decimal::from(format!("{}", amount_of_funds).as_str()));
}

#[test]
fn terminating_beneficiary_after_giving_up_rights_fails() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (_beneficiary_public_key, _beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    // Giving up the termination rights
    let disable_termination_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .call_method(vesting_component, "disable_termination", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _disable_termination_receipt: Receipt = executor.validate_and_execute(&disable_termination_tx).unwrap();

    // Terminating the beneficiary
    let terminating_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .call_method(
            vesting_component,
            "terminate_beneficiary",
            args![NonFungibleId::from_u64(1u64)],
        )
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let terminating_beneficiary_receipt: Receipt = executor.validate_and_execute(&terminating_beneficiary_tx).unwrap();

    assert!(terminating_beneficiary_receipt.result.is_err());
}

#[test]
fn non_admin_cant_disable_termination() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating two accounts for the two parties involved.
    let (admin_public_key, admin_private_key, admin_address): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
        executor.new_account();
    let (_beneficiary_public_key, _beneficiary_private_key, beneficiary_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = executor.new_account();

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let usdt_resource_address: ResourceAddress = token_creation_receipt.new_resource_addresses[0];

    // Creating the vesting component
    let amount_of_funds: u64 = 1_000_000;
    let relative_cliff_epoch: u64 = 20;
    let relative_ending_epoch: u64 = 100;
    let percentage_available_on_cliff: Decimal = dec!("0.2");

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];
    let beneficiary_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[2];

    // Adding the beneficiary
    let adding_beneficiary_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_address)
        .withdraw_from_account_by_amount(amount_of_funds.into(), usdt_resource_address, admin_address)
        .take_from_worktop(usdt_resource_address, |builder, bucket_id| {
            builder.call_method(
                vesting_component,
                "add_beneficiary",
                args![
                    scrypto::resource::Bucket(bucket_id),
                    relative_cliff_epoch,
                    relative_ending_epoch,
                    percentage_available_on_cliff
                ],
            )
        })
        .take_from_worktop(beneficiary_badge, |builder, bucket_id| {
            builder.call_method(
                beneficiary_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let _adding_beneficiary_receipt: Receipt = executor.validate_and_execute(&adding_beneficiary_tx).unwrap();

    // Giving up the termination rights
    let disable_termination_tx: SignedTransaction = TransactionBuilder::new()
        .call_method(vesting_component, "disable_termination", args![])
        .call_method_with_all_resources(admin_address, "deposit_batch")
        .build(executor.get_nonce([admin_public_key]))
        .sign([&admin_private_key]);
    let disable_termination_receipt: Receipt = executor.validate_and_execute(&disable_termination_tx).unwrap();

let error: RuntimeError = disable_termination_receipt.result.expect_err("Should have failed");
    assert_auth_error!(error);
}

#[test]
fn not_enough_admins_cant_disable_termination() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, false);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating 10 accounts for the 10 different admins
    let mut admin_public_keys: Vec<EcdsaPublicKey> = Vec::new();
    let mut admin_private_keys: Vec<EcdsaPrivateKey> = Vec::new();
    let mut admin_addresses: Vec<ComponentAddress> = Vec::new();
    for _ in 0..10 {
        let (pub_key, priv_key, addr): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();
        admin_public_keys.push(pub_key);
        admin_private_keys.push(priv_key);
        admin_addresses.push(addr);
    }

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_addresses[0], "deposit_batch")
        .build(executor.get_nonce([admin_public_keys[0]]))
        .sign([&admin_private_keys[0]]);
    let _token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_addresses[0], "deposit_batch")
        .build(executor.get_nonce([admin_public_keys[0]]))
        .sign([&admin_private_keys[0]]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];

    // Creating 9 additional admin badges for the 9 additional admins in this vesting component
    let additional_admins_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_addresses[0])
        .call_method(vesting_component, "add_admin", args![dec!("9")])
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[1],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[2],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[3],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[4],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[5],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[6],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[7],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[8],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[9],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_addresses[0], "deposit_batch")
        .build(executor.get_nonce([admin_public_keys[0]]))
        .sign([&admin_private_keys[0]]);
    let additional_admins_receipt: Receipt = executor.validate_and_execute(&additional_admins_tx).unwrap();

    // Attempting to giveup the admin rights with only 4 admin badges
    let disable_termination_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_addresses[0])
        .create_proof_from_account(admin_badge, admin_addresses[1])
        .create_proof_from_account(admin_badge, admin_addresses[2])
        .create_proof_from_account(admin_badge, admin_addresses[3])
        .call_method(vesting_component, "disable_termination", args![])
        .build(executor.get_nonce([
            admin_public_keys[0],
            admin_public_keys[1],
            admin_public_keys[2],
            admin_public_keys[3],
        ]))
        .sign([&admin_private_keys[0], &admin_private_keys[1], &admin_private_keys[2], &admin_private_keys[3]]);
    let disable_termination_receipt: Receipt =
        executor.validate_and_execute(&disable_termination_tx).unwrap();

    let error: RuntimeError = disable_termination_receipt.result.expect_err("Should have failed");
    assert_auth_error!(error);
}

#[test]
fn enough_admins_can_disable_termination() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut executor: TransactionExecutor<InMemorySubstateStore> = TransactionExecutor::new(&mut ledger, true);

    // Publishing the package to the ledger
    let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

    // Creating 10 accounts for the 10 different admins
    let mut admin_public_keys: Vec<EcdsaPublicKey> = Vec::new();
    let mut admin_private_keys: Vec<EcdsaPrivateKey> = Vec::new();
    let mut admin_addresses: Vec<ComponentAddress> = Vec::new();
    for _ in 0..10 {
        let (pub_key, priv_key, addr): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) = executor.new_account();
        admin_public_keys.push(pub_key);
        admin_private_keys.push(priv_key);
        admin_addresses.push(addr);
    }

    // Creating a new token to use for the test
    let mut token_information: HashMap<String, String> = HashMap::new();
    token_information.insert("name".to_string(), "Teather".to_string());
    token_information.insert("symbol".to_string(), "USDT".to_string());

    let token_creation_tx: SignedTransaction = TransactionBuilder::new()
        .new_token_fixed(token_information, dec!("10000000"))
        .call_method_with_all_resources(admin_addresses[0], "deposit_batch")
        .build(executor.get_nonce([admin_public_keys[0]]))
        .sign([&admin_private_keys[0]]);
    let _token_creation_receipt: Receipt = executor.validate_and_execute(&token_creation_tx).unwrap();

    let component_creation_tx: SignedTransaction = TransactionBuilder::new()
        .call_function(package, "Vesting", "instantiate_vesting", args![])
        .call_method_with_all_resources(admin_addresses[0], "deposit_batch")
        .build(executor.get_nonce([admin_public_keys[0]]))
        .sign([&admin_private_keys[0]]);
    let component_creation_receipt: Receipt = executor.validate_and_execute(&component_creation_tx).unwrap();

    let vesting_component: ComponentAddress = component_creation_receipt.new_component_addresses[0];
    let admin_badge: ResourceAddress = component_creation_receipt.new_resource_addresses[1];

    // Creating 9 additional admin badges for the 9 additional admins in this vesting component
    let additional_admins_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(admin_badge, admin_addresses[0])
        .call_method(vesting_component, "add_admin", args![dec!("9")])
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[1],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[2],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[3],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[4],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[5],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[6],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[7],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[8],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .take_from_worktop_by_amount(dec!("1"), admin_badge, |builder, bucket_id| {
            builder.call_method(
                admin_addresses[9],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(admin_addresses[0], "deposit_batch")
        .build(executor.get_nonce([admin_public_keys[0]]))
        .sign([&admin_private_keys[0]]);
    let additional_admins_receipt: Receipt = executor.validate_and_execute(&additional_admins_tx).unwrap();
    println!("OUR TX{:?}", additional_admins_receipt);

    // Attempting to giveup the admin rights with only 4 admin badges
    let disable_termination_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account(admin_badge, admin_addresses[0])
        .withdraw_from_account(admin_badge, admin_addresses[1])
        .withdraw_from_account(admin_badge, admin_addresses[2])
        .withdraw_from_account(admin_badge, admin_addresses[3])
        .withdraw_from_account(admin_badge, admin_addresses[4])
        .take_from_worktop(admin_badge, |builder, bucket_id| {
            // Deposit it into a single account because we need to have a composite proof
            builder.call_method(
                admin_addresses[0],
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .create_proof_from_account(admin_badge, admin_addresses[0])
        .call_method(vesting_component, "disable_termination", args![])
        .clear_auth_zone()
        .build(executor.get_nonce([
            admin_public_keys[0],
            admin_public_keys[1],
            admin_public_keys[2],
            admin_public_keys[3],
            admin_public_keys[4],
        ]))
        .sign([
            &admin_private_keys[0],
            &admin_private_keys[1],
            &admin_private_keys[2],
            &admin_private_keys[3],
            &admin_private_keys[4],
        ]);
    let disable_termination_receipt: Receipt =
        executor.validate_and_execute(&disable_termination_tx).unwrap();

    assert!(disable_termination_receipt.result.is_ok());
}
