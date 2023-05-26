use crate::test_lib::TestLib;
use radix_engine::transaction::{CommitResult, TransactionResult};
use scrypto::{blueprints::clock::TimePrecision, prelude::*};
use simple_vote::VoteChoice;
use transaction::builder::ManifestBuilder;

mod test_lib;

#[test]
fn test_vote_happy_path() {
    let mut lib = TestLib::new();

    let (public_key, _private_key, account_component) = lib.test_runner.new_allocated_account();

    let package_address = lib.test_runner.compile_and_publish(this_package!());

    // INSTANTIATE
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
    let expected_resource_names = vec![
        ("Test Vote Admin", None),
        ("Test Component", None),
        ("Test Voting Member", Some("TVM")),
        ("Test Vote Result", Some("TVR")),
    ];
    let mut resources = HashMap::new();
    for addr in new_resources {
        let name = lib
            .expect_string_metadata(addr, "name")
            .expect("Name metadata key should be present");
        let symbol = lib.expect_string_metadata(addr, "symbol");
        resources.insert(name, (addr, symbol));
    }
    assert!(
        expected_resource_names.iter().all(|res| {
            match resources.get(res.0) {
                Some(v) => v.1.as_deref() == res.1,
                _ => false,
            }
        }),
        "Unexpected new resources, expected resources: {:?}\nbut got {:?}",
        expected_resource_names,
        resources
    );
    let component_badge = resources.get("Test Component").unwrap().0;
    let admin_badge = resources.get("Test Vote Admin").unwrap().0;
    let member_badge = resources.get("Test Voting Member").unwrap().0;
    let vote_results = resources.get("Test Vote Result").unwrap().0;

    let vote_resources = lib.test_runner.get_component_resources(vote_component);
    assert_eq!(
        vote_resources.get(component_badge),
        Some(dec!("1")).as_ref()
    );
    assert_eq!(vote_resources.get(vote_results), Some(dec!("0")).as_ref());

    assert_eq!(
        lib.test_runner
            .account_balance(account_component, admin_badge.clone()),
        Some(dec!("1")),
    );

    // BECOME MEMBER
    let manifest = ManifestBuilder::new()
        .call_method(
            account_component,
            "create_proof_by_amount",
            manifest_args!(admin_badge, dec!("1")),
        )
        .call_method(vote_component, "become_member", manifest_args!())
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
    println!("become_member receipt: {:?}", receipt);
    receipt.expect_commit_success();

    assert_eq!(
        lib.test_runner
            .account_balance(account_component, member_badge.clone()),
        Some(dec!("1")),
    );

    // NEW VOTE
    let vote_start = lib.test_runner.get_current_time(TimePrecision::Minute);
    let duration = 5i64;
    let manifest = ManifestBuilder::new()
        .call_method(
            account_component,
            "create_proof_by_amount",
            manifest_args!(admin_badge, dec!("1")),
        )
        .call_method(
            vote_component,
            "new_vote",
            manifest_args!("Test Vote", duration),
        )
        .build();
    let receipt = lib.test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("new_vote receipt: {:?}", receipt);
    receipt.expect_commit_success();

    // VOTE
    let manifest = ManifestBuilder::new()
        .call_method(
            account_component,
            "create_proof_by_amount",
            manifest_args!(member_badge, dec!("1")),
        )
        .pop_from_auth_zone(|builder, member_proof| {
            builder.call_method(
                vote_component,
                "vote",
                manifest_args!(VoteChoice::Yes, member_proof),
            )
        })
        .build();
    let receipt = lib.test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("vote receipt: {:?}", receipt);
    receipt.expect_commit_success();

    // DOUBLE VOTE ATTEMPT
    let manifest = ManifestBuilder::new()
        .call_method(
            account_component,
            "create_proof_by_amount",
            manifest_args!(member_badge, dec!("1")),
        )
        .pop_from_auth_zone(|builder, member_proof| {
            builder.call_method(
                vote_component,
                "vote",
                manifest_args!(VoteChoice::Yes, member_proof),
            )
        })
        .build();
    let receipt = lib.test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("vote again receipt: {:?}", receipt);
    lib.expect_error_log(&receipt, "Member has already voted");
    receipt.expect_commit_failure();

    // END VOTE
    lib.test_runner.set_current_time(
        vote_start
            .add_minutes(duration)
            .expect("Could not make new time")
            .seconds_since_unix_epoch
            * 1000, // Needs milliseconds
    );
    let manifest = ManifestBuilder::new()
        .call_method(vote_component, "end_vote", manifest_args!())
        .build();
    let receipt = lib.test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("end_vote receipt: {:?}", receipt);
    receipt.expect_commit_success();

    // TODO: test access rules for methods
}
