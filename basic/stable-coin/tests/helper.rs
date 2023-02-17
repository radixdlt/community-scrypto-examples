use radix_engine::ledger::*;
use radix_engine::transaction::TransactionReceipt;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::model::FromPublicKey;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::signing::EcdsaSecp256k1PrivateKey;

pub struct User {
    pub public_key: EcdsaSecp256k1PublicKey,
    pub private_key: EcdsaSecp256k1PrivateKey,
    pub compo_addr: ComponentAddress,
}

#[allow(unused)]
pub fn make_users(test_runner: &mut TestRunner<TypedInMemorySubstateStore>) -> (User, User, User) {
    let user1 = make_one_user(test_runner);
    let user2 = make_one_user(test_runner);
    let user3 = make_one_user(test_runner);
    (user1, user2, user3)
}
#[allow(unused)]
pub fn make_one_user(test_runner: &mut TestRunner<TypedInMemorySubstateStore>) -> User {
    let (public_key, private_key, compo_addr) = test_runner.new_allocated_account();
    User {
        public_key,
        private_key,
        compo_addr,
    }
}

#[allow(unused)]
pub fn deploy_blueprint(
    test_runner: &mut TestRunner<TypedInMemorySubstateStore>,
    blueprint_name: &str,
    init_func_name: &str,
    decimal1: Decimal,
    keys_owned: Vec<String>,
    values_owned: Vec<String>,
    user: &User,
) -> (Vec<ResourceAddress>, PackageAddress, ComponentAddress) {
    println!("--------== deploy_blueprint");
    let package_addr = test_runner.compile_and_publish(this_package!());
    //publish_package_with_owner
    println!("check1");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_function(
            package_addr,
            blueprint_name,
            init_func_name,
            args!(decimal1, keys_owned, values_owned),
        )
        .call_method(
            user.compo_addr,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    println!("check2");
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&user.public_key)],
    );
    println!("{}() receipt: {:?}\n", init_func_name, receipt);

    receipt.expect_commit_success();
    let component = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];
    println!("{}() receipt success", init_func_name);
    let resources = receipt.new_resource_addresses();
    ((*resources).clone(), package_addr, component)
}

#[allow(unused)]
pub fn user_balance(
    test_runner: &mut TestRunner<TypedInMemorySubstateStore>,
    user: &User,
    resource_addr: ResourceAddress,
) -> Decimal {
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_method(user.compo_addr, "balance", args!(resource_addr))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&user.public_key)],
    );
    println!("user_balance receipt:{:?}\n", receipt);
    receipt.expect_commit_success();
    receipt.output(1)
}

#[allow(unused)]
pub fn get_data(
    test_runner: &mut TestRunner<TypedInMemorySubstateStore>,
    user: &User,
    component: ComponentAddress,
) -> (u16, Decimal, Decimal) {
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_method(component, "get_data", args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&user.public_key)],
    );
    println!("get_data receipt:{:?}\n", receipt);
    receipt.expect_commit_success();
    println!("get_data receipt success");
    let data: (u16, Decimal, Decimal) = receipt.output(1);
    println!("data:{:?}\n", data);
    data
}

#[allow(unused)]
pub fn call_function(
    test_runner: &mut TestRunner<TypedInMemorySubstateStore>,
    user: &User,
    package_address: PackageAddress,
    blueprint_name: &str,
    function_name: &str,
    token_addr: ResourceAddress,
    keys_owned: Vec<String>,
) -> TransactionReceipt {
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_function(
            package_address,
            blueprint_name,
            function_name,
            args!(token_addr, keys_owned),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&user.public_key)],
    );
    println!("{} receipt:{:?}\n", function_name, receipt);
    receipt.expect_commit_success();
    println!("{} receipt success", function_name);
    receipt
}

#[allow(unused)]
pub fn invoke(
    test_runner: &mut TestRunner<TypedInMemorySubstateStore>,
    user: &User,
    component: ComponentAddress,
    func_name: &str,
    amount: Decimal,
) {
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_method(component, func_name, args!(amount))
        .call_method(
            user.compo_addr,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&user.public_key)],
    );
    println!("{} receipt: {:?}\n", func_name, receipt);
    receipt.expect_commit_success();
    println!("{} ends successfully", func_name);
}

#[allow(unused)]
pub fn invoke_badge_access_decimal(
    test_runner: &mut TestRunner<TypedInMemorySubstateStore>,
    user: &User,
    component: ComponentAddress,
    func_name: &str,
    amount: Decimal,
    badge_addr: ResourceAddress,
    badge_amount: Decimal,
) {
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .create_proof_from_account_by_amount(user.compo_addr, badge_amount, badge_addr)
        .call_method(component, func_name, args!(amount))
        .call_method(
            user.compo_addr,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&user.public_key)],
    );
    println!("{} receipt: {:?}\n", func_name, receipt);
    receipt.expect_commit_success();
    println!("{} ends successfully", func_name);
}

#[allow(unused)]
pub fn invoke_badge_access(
    test_runner: &mut TestRunner<TypedInMemorySubstateStore>,
    user: &User,
    component: ComponentAddress,
    func_name: &str,
    badge_addr: ResourceAddress,
    badge_amount: Decimal,
) {
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .create_proof_from_account_by_amount(user.compo_addr, badge_amount, badge_addr)
        .call_method(component, func_name, args!())
        .call_method(
            user.compo_addr,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&user.public_key)],
    );
    println!("{} receipt: {:?}\n", func_name, receipt);
    receipt.expect_commit_success();
    println!("{} ends successfully", func_name);
}

#[allow(unused)]
pub fn invoke_badge_access_with_bucket(
    test_runner: &mut TestRunner<TypedInMemorySubstateStore>,
    user: &User,
    component: ComponentAddress,
    func_name: &str,
    amount: Decimal,
    token_addr: ResourceAddress,
    badge_addr: ResourceAddress,
    badge_amount: Decimal,
) {
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .create_proof_from_account_by_amount(user.compo_addr, badge_amount, badge_addr)
        .withdraw_from_account_by_amount(user.compo_addr, amount, token_addr)
        .take_from_worktop_by_amount(amount, token_addr, |builder, bucket_id| {
            builder.call_method(component, func_name, args!(Bucket(bucket_id)))
        })
        .call_method(
            user.compo_addr,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&user.public_key)],
    );
    println!("{} receipt: {:?}\n", func_name, receipt);
    receipt.expect_commit_success();
    println!("{} ends successfully", func_name);
}

#[allow(unused)]
pub fn update_metadata(
    test_runner: &mut TestRunner<TypedInMemorySubstateStore>,
    user: &User,
    component: ComponentAddress,
    key: String,
    value: String,
    badge_addr: ResourceAddress,
    badge_amount: Decimal,
) {
    let func_name = "update_metadata";
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .create_proof_from_account_by_amount(user.compo_addr, badge_amount, badge_addr)
        .call_method(component, func_name, args!(key, value))
        .call_method(
            user.compo_addr,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&user.public_key)],
    );
    println!("{} receipt: {:?}\n", func_name, receipt);
    receipt.expect_commit_success();
    println!("{} ends successfully", func_name);
}

#[allow(unused)]
pub fn convert_str_slices(str_vec: Vec<&str>) -> Vec<String> {
    let mut owned_strings: Vec<String> = vec![];
    for i in str_vec {
        owned_strings.push(i.to_owned());
    }
    owned_strings
}
#[allow(unused)]
pub fn vec_option_string(option_vec: Vec<Option<String>>) -> Vec<String> {
    let mut owned_strings: Vec<String> = vec![];
    for e in option_vec {
        if e.is_some() {
            owned_strings.push(e.unwrap());
        } else {
            owned_strings.push("---".to_owned());
        }
    }
    owned_strings
}

//floats and NaNs won't compare! use a tolerance for comparing the other values.
#[allow(unused)]
pub fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

#[allow(unused)]
pub fn find_replace<T: PartialEq>(mut v: Vec<T>, key: &T, value: T) -> (Vec<T>, T) {
    let index = v.iter().position(|r| *r == *key).unwrap();
    let replaced = std::mem::replace(&mut v[index], value);
    (v, replaced)
}

#[allow(unused)]
pub fn find_replace_two_vec<T: PartialEq>(
    mut v: Vec<T>,
    keys: &Vec<T>,
    key: &T,
    value: T,
) -> (Vec<T>, T) {
    let index = keys.iter().position(|r| *r == *key).unwrap();
    let replaced = std::mem::replace(&mut v[index], value);
    (v, replaced)
}

#[allow(unused)]
pub fn send_tokens(
    test_runner: &mut TestRunner<TypedInMemorySubstateStore>,
    user_from: &User,
    user_to: &User,
    component: ComponentAddress,
    amount: Decimal,
    token_addr: ResourceAddress,
) {
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .withdraw_from_account_by_amount(user_from.compo_addr, amount, token_addr)
        .take_from_worktop_by_amount(amount, token_addr, |builder, bucket_id| {
            builder.call_method(user_to.compo_addr, "deposit", args!(Bucket(bucket_id)))
        })
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&user_from.public_key)],
    );
    println!("send_token receipt: {:?}\n", receipt);
    receipt.expect_commit_success();
    println!("send_token ends successfully");
}
