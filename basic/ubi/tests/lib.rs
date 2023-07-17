use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use scrypto::blueprints::clock::TimePrecision;

#[test]
fn test_instantiate_ubi() {
    TestEnv::new();
}

#[test]
fn test_add_identity_provider() {
    let mut test_env = TestEnv::new();
    test_env.add_identity_provider("testprovider0");
}

#[test]
fn test_add_identity_provider_twice() {
    let mut test_env = TestEnv::new();
    test_env.add_identity_provider("testprovider0");
    test_env.expect_failure().add_identity_provider("testprovider0");
    test_env.register("testprovider0");
}

#[test]
fn test_register() {
    let mut test_env = TestEnv::new();
    test_env.add_identity_provider("testprovider0");
    test_env.register("testprovider0");
}

#[test]
fn test_collect_ubi() {
    let mut test_env = TestEnv::new();
    test_env.add_identity_provider("testprovider0");
    test_env.register("testprovider0");
    test_env.set_to_day(30);
    test_env.collect_ubi("testprovider0");
    test_env.assert_balance(dec!(1000));
}

#[test]
fn test_collect_ubi_twice() {
    let mut test_env = TestEnv::new();
    test_env.add_identity_provider("testprovider0");
    test_env.register("testprovider0");
    test_env.set_to_day(30);
    test_env.collect_ubi("testprovider0");
    test_env.collect_ubi("testprovider0");
    test_env.assert_balance(dec!(1000));
}

#[test]
fn test_two_identity_providers() {
    let mut test_env = TestEnv::new();
    test_env.add_identity_provider("testprovider0");
    test_env.register("testprovider0");
    test_env.set_to_day(15);
    test_env.add_identity_provider("testprovider1");
    test_env.register("testprovider1");
    test_env.set_to_day(30);
    test_env.collect_ubi("testprovider0");
    test_env.collect_ubi("testprovider1");
    test_env.assert_balance(dec!(1000));
}

#[test]
fn test_deactivate_identity_provider() {
    let mut test_env = TestEnv::new();
    test_env.add_identity_provider("testprovider0");
    test_env.register("testprovider0");
    test_env.set_to_day(15);
    test_env.add_identity_provider("testprovider1");
    test_env.deactivate_identity_provider("testprovider0");
    test_env.add_identity_provider("testprovider2");
    test_env.add_identity_provider("testprovider3");
    test_env.collect_ubi("testprovider0");
    test_env.set_to_day(30);
    test_env.assert_balance(dec!(500));
}

pub struct TestEnv {
    pub test_runner: TestRunner,
    admin_account: ComponentAddress,
    ubi_component: ComponentAddress,
    ubi_token: ResourceAddress,
    super_badge: ResourceAddress,
    pub admin_public_key: EcdsaSecp256k1PublicKey,
    identity_provider_resources: HashMap<String, ResourceAddress>,
    expect_failure: bool,
    start_time: i64,
}

impl TestEnv {
    pub fn new() -> Self {
        let mut test_runner = TestRunner::builder().build();

        let (public_key, _private_key, account_component) = test_runner.new_allocated_account();
        let package_address = test_runner.compile_and_publish(this_package!());
        let admin_account_nft = NonFungibleGlobalId::from_public_key(&public_key);
        let manifest = ManifestBuilder::new()
            .call_function(package_address, "UBI", "instantiate_ubi", manifest_args!())
            .call_method(
                account_component,
                "deposit_batch",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![admin_account_nft]);
        //println!("{:?}\n", receipt);
        receipt.expect_commit_success();
        let commit_result = receipt.expect_commit(true);
        let start_time = test_runner.get_current_time(TimePrecision::Minute).seconds_since_unix_epoch;

        let test_env = Self {
            admin_account: account_component,
            test_runner: test_runner,
            ubi_component: commit_result.state_update_summary.new_components[0],
            ubi_token: commit_result.state_update_summary.new_resources[2],
            super_badge: commit_result.state_update_summary.new_resources[1],
            admin_public_key: public_key,
            identity_provider_resources: HashMap::new(),
            expect_failure: false,
            start_time
        };
        test_env
    }

    fn expect_failure(&mut self) -> &mut Self {
        self.expect_failure = true;
        self
    }

    fn assert_balance(&mut self, expected_balance: Decimal) {
        let balances = self.test_runner.get_component_resources(self.admin_account);
        let balance = balances.get(&self.ubi_token).unwrap();
        println!("Balance: {}, expected {}", balance, expected_balance);
        assert!((*balance - expected_balance).abs() < dec!("0.0001"));
    }

    fn add_identity_provider(&mut self, provider_name: &str) -> radix_engine::transaction::TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .create_proof_from_account(self.admin_account, self.super_badge)
            .call_method(self.ubi_component, "add_identity_provider", manifest_args!(provider_name))
            .call_method(
                self.admin_account,
                "deposit_batch",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.test_runner.execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&self.admin_public_key)]);
        let commit_result = receipt.expect_commit(!self.expect_failure);
        self.validate(&receipt);
        if receipt.is_commit_success() {
            self.identity_provider_resources.insert(String::from(provider_name), commit_result.state_update_summary.new_resources[0]);
        }
        receipt
    }

    fn deactivate_identity_provider(&mut self, provider_name: &str) -> radix_engine::transaction::TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .create_proof_from_account(self.admin_account, self.super_badge)
            .call_method(self.ubi_component, "deactivate_identity_provider", manifest_args!(provider_name))
            .call_method(
                self.admin_account,
                "deposit_batch",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.test_runner.execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&self.admin_public_key)]);
        self.validate(&receipt);
        receipt
    }

    fn validate(&mut self, receipt: &radix_engine::transaction::TransactionReceipt) {
        if self.expect_failure {
            receipt.expect_commit_failure();
            self.expect_failure = false;
        } else {
            receipt.expect_commit_success();
        }
    }

    fn register(&mut self, provider_name: &str) -> radix_engine::transaction::TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .create_proof_from_account(self.admin_account, self.super_badge)
            .call_method(self.ubi_component, "register", manifest_args!(provider_name, "testid"))
            .call_method(
                self.admin_account,
                "deposit_batch",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.test_runner.execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&self.admin_public_key)]);
        self.validate(&receipt);
        receipt
    }

    fn collect_ubi(&mut self, provider_name: &str) -> radix_engine::transaction::TransactionReceipt {
        let manifest = ManifestBuilder::new()
        .withdraw_from_account(self.admin_account, self.identity_provider_resources[provider_name], 1.into())
        .take_from_worktop(self.identity_provider_resources[provider_name], |builder, identity_badge| {
            builder.call_method(self.ubi_component, "collect_ubi", manifest_args!(identity_badge))
        })
        .call_method(
            self.admin_account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
        let receipt = self.test_runner.execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&self.admin_public_key)]);
        self.validate(&receipt);
        receipt
    }

    fn set_to_day(&mut self, days: i64) {
        let days_from_now_ms = self.start_time + days * 24 * 60 * 60 * 1000;
        self.test_runner.set_current_time(days_from_now_ms);
    }

}