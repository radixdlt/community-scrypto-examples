use crate::test_lib::TestLib;
use radix_engine::transaction::TransactionReceipt;
use scrypto::{blueprints::clock::TimePrecision, prelude::*};
use simple_vote::{vote::Vote, CurrentVote, VoteChoice};
use transaction::{builder::ManifestBuilder, model::TransactionManifest};

mod test_lib;

struct VoteTest {
    public_key: EcdsaSecp256k1PublicKey,
    account_component: ComponentAddress,
    resources: HashMap<String, (ResourceAddress, Option<String>)>,
    component_badge: ResourceAddress,
    admin_badge: ResourceAddress,
    member_badge: ResourceAddress,
    vote_results_addr: ResourceAddress,
    lib: TestLib,
    vote_component: ComponentAddress,
}

impl VoteTest {
    pub fn new() -> Self {
        let mut lib = TestLib::new();

        let (public_key, _private_key, account_component) = lib.test_runner.new_allocated_account();

        let package_address = lib.test_runner.compile_and_publish(this_package!());

        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "Vote",
                "instantiate_vote",
                manifest_args!("Test"),
            )
            .call_method(
                account_component,
                "deposit_batch",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = lib.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );
        println!("instantiate_vote receipt: {:?}", receipt);
        let result = receipt.expect_commit_success();
        let vote_component = result.new_component_addresses()[0];
        let new_resources = result.new_resource_addresses();
        let mut resources = HashMap::new();
        for addr in new_resources {
            let name = lib
                .expect_string_metadata(addr, "name")
                .expect("Name metadata key should be present");
            let symbol = lib.expect_string_metadata(addr, "symbol");
            resources.insert(name, (addr.clone(), symbol));
        }
        let component_badge = resources.get("Test Component").unwrap().0;
        let admin_badge = resources.get("Test Vote Admin").unwrap().0;
        let member_badge = resources.get("Test Voting Member").unwrap().0;
        let vote_results_addr = resources.get("Test Vote Result").unwrap().0;

        Self {
            public_key,
            account_component,
            resources,
            vote_component,
            component_badge: component_badge.clone(),
            admin_badge: admin_badge.clone(),
            member_badge: member_badge.clone(),
            vote_results_addr: vote_results_addr.clone(),
            lib,
        }
    }

    fn execute_become_member(&mut self) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.account_component,
                "create_proof_by_amount",
                manifest_args!(self.admin_badge, dec!("1")),
            )
            .call_method(self.vote_component, "become_member", manifest_args!())
            .call_method(
                self.account_component,
                "deposit_batch",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest);
        println!("become_member receipt: {:?}", receipt);
        receipt
    }

    fn execute_new_vote(&mut self, name: &str, duration: i64) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.account_component,
                "create_proof_by_amount",
                manifest_args!(self.admin_badge, dec!("1")),
            )
            .call_method(
                self.vote_component,
                "new_vote",
                manifest_args!(name, duration),
            )
            .build();
        let receipt = self.execute_manifest(manifest);
        println!("new_vote receipt: {:?}", receipt);
        receipt
    }

    fn execute_vote(&mut self, choice: VoteChoice) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.account_component,
                "create_proof_by_amount",
                manifest_args!(self.member_badge, dec!("1")),
            )
            .pop_from_auth_zone(|builder, member_proof| {
                builder.call_method(
                    self.vote_component,
                    "vote",
                    manifest_args!(choice, member_proof),
                )
            })
            .build();
        let receipt = self.execute_manifest(manifest);
        println!("vote receipt: {:?}", receipt);
        receipt
    }

    fn execute_end_vote(&mut self) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(self.vote_component, "end_vote", manifest_args!())
            .build();
        let receipt = self.execute_manifest(manifest);
        println!("end_vote receipt: {:?}", receipt);
        receipt
    }

    fn execute_manifest(&mut self, manifest: TransactionManifest) -> TransactionReceipt {
        self.lib.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&self.public_key)],
        )
    }
}

#[test]
fn test_vote_happy_path() {
    // INSTANTIATE
    let mut test = VoteTest::new();

    let expected_resource_names = vec![
        ("Test Vote Admin", None),
        ("Test Component", None),
        ("Test Voting Member", Some("TVM")),
        ("Test Vote Result", Some("TVR")),
    ];
    assert!(
        expected_resource_names.iter().all(|res| {
            match test.resources.get(res.0) {
                Some(v) => v.1.as_deref() == res.1,
                _ => false,
            }
        }),
        "Unexpected new resources, expected resources: {:?}\nbut got {:?}",
        expected_resource_names,
        test.resources
    );

    let vote_resources = test
        .lib
        .test_runner
        .get_component_resources(test.vote_component);
    assert_eq!(
        vote_resources.get(&test.component_badge),
        Some(dec!("1")).as_ref()
    );
    assert_eq!(
        vote_resources.get(&test.vote_results_addr),
        Some(dec!("0")).as_ref()
    );

    assert_eq!(
        test.lib
            .test_runner
            .account_balance(test.account_component, test.admin_badge.clone()),
        Some(dec!("1")),
    );

    // BECOME MEMBER
    test.execute_become_member().expect_commit_success();

    assert_eq!(
        test.lib
            .test_runner
            .account_balance(test.account_component, test.member_badge.clone()),
        Some(dec!("1")),
    );

    // NEW VOTE
    let vote_start = test.lib.test_runner.get_current_time(TimePrecision::Minute);
    let duration = 5i64;
    test.execute_new_vote("Test Vote", duration)
        .expect_commit_success();

    // VOTE
    test.execute_vote(VoteChoice::Yes).expect_commit_success();

    // DOUBLE VOTE ATTEMPT
    let receipt = test.execute_vote(VoteChoice::Yes);
    TestLib::expect_error_log(&receipt, "Member has already voted");
    receipt.expect_commit_failure();

    // END VOTE
    test.lib.test_runner.set_current_time(
        vote_start
            .add_minutes(duration)
            .expect("Could not make new time")
            .seconds_since_unix_epoch
            * 1000, // Needs milliseconds
    );
    test.execute_end_vote().expect_commit_success();

    let vote_state = test.lib.get_component_state::<Vote>(&test.vote_component);
    let vote_result_ids = test
        .lib
        .test_runner
        .inspect_non_fungible_vault(vote_state.vote_results.0)
        .expect("Vote results vault could not be accessed");
    assert_eq!(vote_result_ids.len(), 1);
    let vote_result_id = vote_result_ids.first().unwrap();
    let vote_result = test
        .lib
        .get_non_fungible_data::<CurrentVote>(&test.vote_results_addr, vote_result_id);
    assert_eq!(vote_result.yes_votes, 1);
    assert_eq!(vote_result.no_votes, 0);
    assert_eq!(vote_result.name, "Test Vote");
}

// TODO: test
// - access rules for methods
// - All assertions in methods
// - default resource behaviour states
// - configured resource behaviours
