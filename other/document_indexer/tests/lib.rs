//! This test suit uses the `TestRunner` class. Each test in the suite
//! runs through a set of related transactions as indicated by its
//! name.
//!
//! All the public methods in the `DocumentIndexer` blueprint get
//! called as part of these tests, and this file demonstrates
//! transaction manifest building for all of them.
//!
//! The functions named `call_...` all perform a call to the
//! ledger. They return the `CommitResult` if any, and some also
//! return the return values from the ledger method if there were
//! any. Most take a `succeed` argument which tells the function
//! whether it should expect to succeed or not, and they assert on
//! this condition.
//!
//! Note that the majority of transaction manifests built in this file
//! do not start by calling `lock_fee`, as they are run using
//! `execute_manifest_ignoring_fee`. A few get run using
//! `execute_manifest` instead and those do lock fees first.

use scrypto::prelude::*;
use transaction::prelude::*;
use scrypto_unit::*;
use radix_engine::transaction::{CommitResult, BalanceChange};
use document_indexer::IndexEntry;

struct User {
    pubkey: Secp256k1PublicKey,
    _privkey: Secp256k1PrivateKey,
    account: ComponentAddress,
}

impl User {
    /// Creates a new user account on the ledger.
    fn new(test_runner: &mut TestRunner) -> Self {
        let (user_pubk, user_privk, user_account) =
            test_runner.new_allocated_account();

        User {
            pubkey: user_pubk,
            _privkey: user_privk,
            account: user_account,
        }
    }
}

/// Transfers a bunch of non-fungibles from one user account to
/// another.
///
/// Note that we use `try_deposit_batch_or_abort` since we're trying
/// to give funds to someone other than the signer of the transaction.
fn give_nf_tokens(test_runner: &mut TestRunner,
                  giver: &User,
                  recip: &User,
                  gift_token: &ResourceAddress,
                  nflids: BTreeSet<NonFungibleLocalId>)
{
    let manifest = ManifestBuilder::new()
        .withdraw_non_fungibles_from_account(
            giver.account,
            *gift_token,
            &nflids)
        .try_deposit_batch_or_abort(recip.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&giver.pubkey)],
    );

    receipt.expect_commit_success();
}

/// Builds a transaction manifest for `DocumentIndexer::new` and runs
/// it. Always expects success.
fn call_new(test_runner: &mut TestRunner,
            package: PackageAddress,
            notary: &User,
            name: Option<&str>,
            owner: &NonFungibleGlobalId,
            admins: BTreeSet<NonFungibleGlobalId>,
            users_resaddr: &ResourceAddress)
            -> (ComponentAddress, CommitResult) {

    let manifest = ManifestBuilder::new()
        .call_function(
            package,
            "DocumentIndexer",
            "new",
            manifest_args!(
                name,
                owner,
                admins,
                users_resaddr
            ),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    let result = receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    (component, result.clone())
}

/// Builds a transaction manifest for `DocumentIndexer::organization`
/// and runs it. Always expects success.
fn call_organization(test_runner: &mut TestRunner,
                     docindexer: ComponentAddress,
                     notary: &User)
                     -> (Option<String>, CommitResult) {

    let manifest = ManifestBuilder::new()
        .call_method(docindexer,
                     "organization",
                     manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    let result = receipt.expect_commit_success();

    let organization = receipt.expect_commit(true).output(1);

    (organization, result.clone())
}

/// Builds and returns a transaction manifest for
/// `DocumentIndexer::add_document`.
fn build_add_document_manifest(docindexer: ComponentAddress,
                               signer: &User,
                               user: &NonFungibleGlobalId,
                               document_name: &str)
                               -> TransactionManifestV1 {
    ManifestBuilder::new()
        .lock_fee(signer.account, dec!(1))
        .create_proof_from_account_of_non_fungibles(
            signer.account,
            user.resource_address(),
            &BTreeSet::from([user.local_id().clone()]))
        .call_method(
            docindexer,
            "add_document",
            manifest_args!(
                document_name))
        .build()
}

/// Builds a transaction manifest for `DocumentIndexer::add_document`
/// and runs it. Expects success if `succeed` is true, otherwise
/// expects failure.
fn call_add_document(test_runner: &mut TestRunner,
                     docindexer: ComponentAddress,
                     signer: &User,
                     user: &NonFungibleGlobalId,
                     document_name: &str,
                     succeed: bool)
                     -> CommitResult {

    let manifest = build_add_document_manifest(
        docindexer, signer, user, document_name);

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&signer.pubkey)],
    );

    let result = if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() };

    result.clone()
}

/// Builds a transaction manifest for `DocumentIndexer::add_document`
/// and runs it while deducting transaction fees. Expects success if
/// `succeed` is true, otherwise expects rejection.
fn call_add_document_with_fee(test_runner: &mut TestRunner,
                              docindexer: ComponentAddress,
                              signer: &User,
                              user: &NonFungibleGlobalId,
                              document_name: &str,
                              succeed: bool)
                              -> Option<CommitResult> {

    let manifest = build_add_document_manifest(
        docindexer, signer, user, document_name);

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&signer.pubkey)],
    );

    if succeed { Some(receipt.expect_commit_success().clone()) }
    else { receipt.expect_rejection(); None }
}

/// Builds and returns a transaction manifest for
/// `DocumentIndexer::register_document_edition`.
fn build_register_document_edition_manifest(
    docindexer: ComponentAddress,
    signer: &User,
    user_in_auth_zone: &NonFungibleGlobalId,
    user_passed_as_arg: &NonFungibleGlobalId,
    document_name: &str,
    document_hash: &str,
    attributes: &HashMap<String, String>)
    -> TransactionManifestV1 {

    ManifestBuilder::new()
        .lock_fee(signer.account, dec!(1))
        .create_proof_from_account_of_non_fungibles(
            signer.account,
            user_in_auth_zone.resource_address(),
            &BTreeSet::from([user_in_auth_zone.local_id().clone()]))
        .call_method(
            docindexer,
            "register_document_edition",
            manifest_args!(
                user_passed_as_arg.local_id(),
                document_name,
                document_hash,
                attributes)
        )
        .build()
}

/// Builds a transaction manifest for
/// `DocumentIndexer::register_document_edition` and runs it. Expects
/// success if `succeed` is true, otherwise expects failure.
///
/// Note that under normal circumstances you will pass the same user
/// as `user_in_auth_zone` and `user_passed_as_arg`. These are
/// different only when you're testing abberant behaviour.
fn call_register_document_edition(
    test_runner: &mut TestRunner,
    docindexer: ComponentAddress,
    signer: &User,
    user_in_auth_zone: &NonFungibleGlobalId,
    user_passed_as_arg: &NonFungibleGlobalId,
    document_name: &str,
    document_hash: &str,
    attributes: &HashMap<String, String>,
    succeed: bool)
    -> CommitResult {

    let manifest = build_register_document_edition_manifest(
        docindexer, signer, user_in_auth_zone, user_passed_as_arg,
        document_name, document_hash, attributes);

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&signer.pubkey)],
    );

    let result = if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() };

    result.clone()
}

/// Builds a transaction manifest for
/// `DocumentIndexer::register_document_edition` which can seem
/// confused about which user to use: it puts both in the auth zone
/// and passes one of them as argument to the method. This should only
/// work if the one passed as argument has the same local-id as a
/// legitimate user that is in the auth zone.
///
/// Expects success if `succeed` is true, otherwise expects failure.
fn call_register_document_edition_confused(
    test_runner: &mut TestRunner,
    docindexer: ComponentAddress,
    signer: &User,
    user_in_auth_zone: &NonFungibleGlobalId,
    user_passed_as_arg: &NonFungibleGlobalId,
    document_name: &str,
    document_hash: &str,
    attributes: &HashMap<String, String>,
    succeed: bool)
    -> CommitResult {

    let manifest = ManifestBuilder::new()
        .lock_fee(signer.account, dec!(1))
        .create_proof_from_account_of_non_fungibles(
            signer.account,
            user_in_auth_zone.resource_address(),
            &BTreeSet::from([user_in_auth_zone.local_id().clone()]))
        .create_proof_from_account_of_non_fungibles(
            signer.account,
            user_passed_as_arg.resource_address(),
            &BTreeSet::from([user_passed_as_arg.local_id().clone()]))
        .call_method(
            docindexer,
            "register_document_edition",
            manifest_args!(
                user_passed_as_arg.local_id(),
                document_name,
                document_hash,
                attributes)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&signer.pubkey)],
    );

    let result = if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() };

    result.clone()
}

/// Builds a transaction manifest for
/// `DocumentIndexer::register_document_edition` and runs it while
/// deducting transaction fees. Expects success if `succeed` is true,
/// otherwise expects rejection.
fn call_register_document_edition_with_fee(
    test_runner: &mut TestRunner,
    docindexer: ComponentAddress,
    signer: &User,
    user: &NonFungibleGlobalId,
    document_name: &str,
    document_hash: &str,
    attributes: &HashMap<String, String>,
    succeed: bool)
    -> Option<CommitResult> {

    let manifest = build_register_document_edition_manifest(
        docindexer, signer, user, user,
        document_name, document_hash, attributes);

    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&signer.pubkey)],
    );

    if succeed { Some(receipt.expect_commit_success().clone()) }
    else { receipt.expect_rejection(); None }
}

/// Builds a transaction manifest for `DocumentIndexer::edition_count`
/// and runs it, returning the edition count. Expects success if
/// `succeed` is true, otherwise expects failure.
fn call_edition_count(test_runner: &mut TestRunner,
                      docindexer: ComponentAddress,
                      signer: &User,
                      document_name: &str,
                      succeed: bool)
                      -> (Option<u64>, CommitResult) {

    let manifest = ManifestBuilder::new()
        .call_method(
            docindexer,
            "edition_count",
            manifest_args!(document_name))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&signer.pubkey)],
    );

    let result = if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() };

    let count = if succeed { Some(result.output(1)) } else { None };
    
    (count, result.clone())
}

/// Builds a transaction manifest for `DocumentIndexer::editions`
/// and runs it, returning the editions. Expects success if
/// `succeed` is true, otherwise expects failure.
fn call_editions(test_runner: &mut TestRunner,
                 docindexer: ComponentAddress,
                 signer: &User,
                 document_name: &str,
                 start: u64,
                 count: u64,
                 succeed: bool)
                 -> (Option<Vec<IndexEntry>>, CommitResult) {

    let manifest = ManifestBuilder::new()
        .call_method(
            docindexer,
            "editions",
            manifest_args!(document_name, start, count))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&signer.pubkey)],
    );

    let result = if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() };

    let count = if succeed { Some(result.output(1)) } else { None };
    
    (count, result.clone())
}

/// Builds a transaction manifest for `DocumentIndexer::add_funds` and
/// runs it. Expects success if `succeed` is true, otherwise expects
/// failure.
fn call_add_funds(test_runner: &mut TestRunner,
                  docindexer: ComponentAddress,
                  signer: &User,
                  amount: Decimal,
                  succeed: bool)
                  -> CommitResult {

    let manifest = ManifestBuilder::new()
        .withdraw_from_account(signer.account, RADIX_TOKEN, amount)
        .take_from_worktop(RADIX_TOKEN, amount, "funds_bucket")
        .call_method_with_name_lookup(
            docindexer,
            "add_funds",
            |lookup| manifest_args!(
                lookup.bucket("funds_bucket")))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&signer.pubkey)],
    );

    let result = if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() };

    result.clone()
}

/// Builds a transaction manifest for `DocumentIndexer::remove_funds`
/// and runs it. Expects success if `succeed` is true, otherwise
/// expects failure.
fn call_remove_funds(test_runner: &mut TestRunner,
                     docindexer: ComponentAddress,
                     signer: &User,
                     user: &NonFungibleGlobalId,
                     amount: Decimal,
                     succeed: bool)
                     -> CommitResult {

    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            signer.account,
            user.resource_address(),
            &BTreeSet::from([user.local_id().clone()]))
        .call_method(
            docindexer,
            "remove_funds",
            manifest_args!(amount))
        .deposit_batch(signer.account)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&signer.pubkey)],
    );

    let result = if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() };

    result.clone()
}


/// This tests that the organization name can be set on component
/// instantiation and retrieved later, primarily testing the
/// `DocumentIndexer::organization` method.
#[test]
fn test_organization() {
    let mut test_runner = TestRunner::builder().build();

    let package = test_runner.compile_and_publish(this_package!());

    let alice = User::new(&mut test_runner);
    let alice_nfgid =
        NonFungibleGlobalId::from_public_key(&alice.pubkey);

    let users_resaddr =
        test_runner.create_non_fungible_resource(alice.account);

    let admins: BTreeSet<NonFungibleGlobalId> =
        std::iter::once(alice_nfgid.clone()).collect();

    let (docindexer, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 Some("My indexer"),
                 &alice_nfgid,
                 admins,
                 &users_resaddr);

    let (organization, _) =
        call_organization(&mut test_runner,
                          docindexer,
                          &alice);

    assert_eq!("My indexer",
               organization.expect("Organization name should be set"),
               "Organization name should be as we set it");
}


/// This tests the `DocumentIndexer::add_document` method.
#[test]
fn test_add_document() {
    let mut test_runner = TestRunner::builder().build();

    let package = test_runner.compile_and_publish(this_package!());

    let alice = User::new(&mut test_runner);
    let alice_nfgid =
        NonFungibleGlobalId::from_public_key(&alice.pubkey);

    let users_resaddr =
        test_runner.create_non_fungible_resource(alice.account);
    let user_nfgid = NonFungibleGlobalId::new(users_resaddr, 1.into());

    let nonusers_resaddr =
        test_runner.create_non_fungible_resource(alice.account);
    let nonuser_nfgid = NonFungibleGlobalId::new(nonusers_resaddr, 1.into());

    let admins: BTreeSet<NonFungibleGlobalId> =
        std::iter::once(alice_nfgid.clone()).collect();

    let (docindexer, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 None,
                 &alice_nfgid,
                 admins,
                 &users_resaddr);

    // First time should succeed
    call_add_document(&mut test_runner,
                      docindexer,
                      &alice,
                      &user_nfgid,
                      "New Document",
                      true);
    
    // Second time with same name should fail
    call_add_document(&mut test_runner,
                      docindexer,
                      &alice,
                      &user_nfgid,
                      "New Document",
                      false);

    // A different name should work however
    call_add_document(&mut test_runner,
                      docindexer,
                      &alice,
                      &user_nfgid,
                      "New Document (1)",
                      true);

    // Non-approved user should fail
    call_add_document(&mut test_runner,
                      docindexer,
                      &alice,
                      &nonuser_nfgid,
                      "Fake Document",
                      false);
}


/// This test primarily tests the
/// `DocumentIndexer::register_document_edition` method but
/// secondarily also tests `DocumentIndexer::edition_count` and
/// `DocumentIndexer::editions`.
#[test]
fn test_register_document_edition() {
    let mut test_runner = TestRunner::builder().build();

    let package = test_runner.compile_and_publish(this_package!());

    let alice = User::new(&mut test_runner);
    let alice_nfgid =
        NonFungibleGlobalId::from_public_key(&alice.pubkey);

    let users_resaddr =
        test_runner.create_non_fungible_resource(alice.account);
    let user1_nfgid = NonFungibleGlobalId::new(users_resaddr, 1.into());
    let user2_nfgid = NonFungibleGlobalId::new(users_resaddr, 2.into());

    let nonusers_resaddr =
        test_runner.create_non_fungible_resource(alice.account);
    let nonuser1_nfgid = NonFungibleGlobalId::new(nonusers_resaddr, 1.into());
    let nonuser2_nfgid = NonFungibleGlobalId::new(nonusers_resaddr, 2.into());

    let admins: BTreeSet<NonFungibleGlobalId> =
        std::iter::once(alice_nfgid.clone()).collect();

    let (docindexer, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 None,
                 &alice_nfgid,
                 admins,
                 &users_resaddr);

    // This should fail on non-existing documents
    call_edition_count(
        &mut test_runner,
        docindexer,
        &alice,
        "Test Document",
        false);

    call_add_document(&mut test_runner,
                      docindexer,
                      &alice,
                      &user1_nfgid,
                      "Test Document",
                      true);

    assert_eq!(0,
               call_edition_count(
                   &mut test_runner,
                   docindexer,
                   &alice,
                   "Test Document",
                   true).0.unwrap(),
               "There should be no editions yet");

    call_register_document_edition(&mut test_runner,
                                   docindexer,
                                   &alice,
                                   &user1_nfgid,
                                   &user1_nfgid,
                                   "Test Document",
                                   "hashtesthash",
                                   &HashMap::new(),
                                   true);
    
    assert_eq!(1,
               call_edition_count(
                   &mut test_runner,
                   docindexer,
                   &alice,
                   "Test Document",
                   true).0.unwrap(),
               "There should now be 1 edition");

    // We now try to pass a non-user with local-id 2 to the method
    // (and this one is in auth zone) while we also have an actual
    // user in auth zone with local-id 1. This shouldn't be accepted.
    call_register_document_edition_confused(&mut test_runner,
                                            docindexer,
                                            &alice,
                                            &user1_nfgid,
                                            &nonuser2_nfgid,
                                            "Test Document",
                                            "hashtesthash",
                                            &HashMap::new(),
                                            false);

    // We try to have a user with local-id 1 in the auth zone but pass
    // a user with local-id 2 to the method. This shouldn't work when
    // local-id 2 isn't in auth zone.
    call_register_document_edition(&mut test_runner,
                                   docindexer,
                                   &alice,
                                   &user1_nfgid,
                                   &user2_nfgid,
                                   "Test Document",
                                   "hashtesthash",
                                   &HashMap::new(),
                                   false);

    // We're back to testing things that should work again, adding a
    // document edition.
    call_register_document_edition(&mut test_runner,
                                   docindexer,
                                   &alice,
                                   &user1_nfgid,
                                   &user1_nfgid,
                                   "Test Document",
                                   "hashtesthash2",
                                   &HashMap::from(
                                       [("version".to_owned(),"3.0".to_owned()),
                                        ("author".to_owned(), "Scryptonight".to_owned())]),
                                   true);

    let editions = call_editions(&mut test_runner,
                                 docindexer,
                                 &alice,
                                 "Test Document",
                                 1,
                                 1,
                                 true).0.unwrap();

    assert_eq!(user1_nfgid,
               editions[0].registering_user,
               "registering_user should be user1");
    assert_eq!(0,
               editions[0].registration_timestamp,
               "timestamp should be 0");
    assert_eq!("hashtesthash2",
               editions[0].document_hash,
               "hash should be as we set it");
    assert_eq!(2,
               editions[0].attributes.len(),
               "there should be two attributes");
    assert_eq!("3.0",
               editions[0].attributes["version"],
               "version should be as we set it");
    assert_eq!("Scryptonight",
               editions[0].attributes["author"],
               "author should be as we set it");
    
    assert_eq!(2,
               call_edition_count(
                   &mut test_runner,
                   docindexer,
                   &alice,
                   "Test Document",
                   true).0.unwrap(),
               "There should now be 2 editions");

    // A non-approved user should not be allowed to do this
    call_register_document_edition(&mut test_runner,
                                   docindexer,
                                   &alice,
                                   &nonuser1_nfgid,
                                   &nonuser1_nfgid,
                                   "Test Document",
                                   "hashtesthash3",
                                   &HashMap::new(),
                                   false);

    // Check that another approved user can also do it
    let bob = User::new(&mut test_runner);
    // Give Bob a user token
    give_nf_tokens(&mut test_runner,
                &alice,
                &bob,
                &users_resaddr,
                BTreeSet::from([user2_nfgid.local_id().clone()]));
    
    call_register_document_edition(&mut test_runner,
                                   docindexer,
                                   &bob,
                                   &user2_nfgid,
                                   &user2_nfgid,
                                   "Test Document",
                                   "hashtesthash4",
                                   &HashMap::new(),
                                   true);

    assert_eq!(3,
               call_edition_count(
                   &mut test_runner,
                   docindexer,
                   &bob,
                   "Test Document",
                   true).0.unwrap(),
               "There should now be 3 editions");
}


/// This tests the `DocumentIndexer::editions` method. Of the tests in
/// this package it is the one that is most likely to start failing if
/// Scrypto cost units increase so keep that in mind when maintaining
/// it.
#[test]
fn test_editions() {
    let mut test_runner = TestRunner::builder().build();

    let package = test_runner.compile_and_publish(this_package!());

    let alice = User::new(&mut test_runner);
    let alice_nfgid =
        NonFungibleGlobalId::from_public_key(&alice.pubkey);

    let users_resaddr =
        test_runner.create_non_fungible_resource(alice.account);
    let user1_nfgid = NonFungibleGlobalId::new(users_resaddr, 1.into());

    let admins: BTreeSet<NonFungibleGlobalId> =
        std::iter::once(alice_nfgid.clone()).collect();

    let (docindexer, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 None,
                 &alice_nfgid,
                 admins,
                 &users_resaddr);

    call_add_document(&mut test_runner,
                      docindexer,
                      &alice,
                      &user1_nfgid,
                      "Test Document 1",
                      true);

    for count in 0..100 {
        call_register_document_edition(&mut test_runner,
                                       docindexer,
                                       &alice,
                                       &user1_nfgid,
                                       &user1_nfgid,
                                       "Test Document 1",
                                       &format!("doc1hashtesthash{}", count),
                                       &HashMap::from(
                                           [
                                               ("version".to_owned(),
                                                count.to_string())
                                           ]),
                                       true);
    }

    // We add another document just to assert that doing so won't mess
    // up the tests
    call_add_document(&mut test_runner,
                      docindexer,
                      &alice,
                      &user1_nfgid,
                      "Test Document 2",
                      true);

    for count in 0..100 {
        call_register_document_edition(&mut test_runner,
                                       docindexer,
                                       &alice,
                                       &user1_nfgid,
                                       &user1_nfgid,
                                       "Test Document 2",
                                       &format!("doc2hashtesthash{}", count),
                                       &HashMap::new(),
                                       true);
    }


    // Testing fetch that starts beyond end
    let (editions, _) = 
        call_editions(&mut test_runner,
                      docindexer,
                      &alice,
                      "Test Document 1",
                      500, 5,
                      true);
    assert_eq!(0, editions.unwrap().len(),
               "Fetch beyond end should result in zero length return");
    

    // Fetching indices 14..18 inclusive and checking contents
    let (editions, _) = 
        call_editions(&mut test_runner,
                      docindexer,
                      &alice,
                      "Test Document 1",
                      14, 5,
                      true);
    assert_eq!(5, editions.as_ref().unwrap().len(),
               "Should have 5 entries returned");

    for count in 0..5 {
        assert_eq!(count + 14,
                   editions.as_ref().unwrap()[count]
                   .attributes.get("version").unwrap().parse::<usize>().unwrap(),
                   "Version in attributes should be same as its index");
    }


    // Fetching all entries by using a big count.
    let (editions, _) = 
        call_editions(&mut test_runner,
                      docindexer,
                      &alice,
                      "Test Document 1",
                      0, 9999999,
                      true);
    assert_eq!(100, editions.as_ref().unwrap().len(),
               "Should have 100 entries returned");

    for count in 0..100 {
        assert_eq!(count,
                   editions.as_ref().unwrap()[count]
                   .attributes.get("version").unwrap().parse::<usize>().unwrap(),
                   "Version in attributes should be same as its index");
        assert_eq!(format!("doc1hashtesthash{}", count),
                   editions.as_ref().unwrap()[count].document_hash,
                   "Hash in index entry should be as we set it");
    }
}


/// This tests the `DocumentIndexer::add_funds` and
/// `DocumentIndexer::remove_funds` methods.
#[test]
fn test_funds() {
    let mut test_runner = TestRunner::builder().build();

    let package = test_runner.compile_and_publish(this_package!());

    // Alice will own the component
    let alice = User::new(&mut test_runner);
    let alice_nfgid =
        NonFungibleGlobalId::from_public_key(&alice.pubkey);

    // Bob is an anonymous user
    let bob = User::new(&mut test_runner);
    let bob_nfgid =
        NonFungibleGlobalId::from_public_key(&bob.pubkey);

    let users_resaddr =
        test_runner.create_non_fungible_resource(alice.account);
    let user1_nfgid = NonFungibleGlobalId::new(users_resaddr, 1.into());

    let admins: BTreeSet<NonFungibleGlobalId> =
        std::iter::once(alice_nfgid.clone()).collect();

    let (docindexer, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 None,
                 &alice_nfgid,
                 admins,
                 &users_resaddr);

    // Anyone (e.g. Bob) should be able to add funds
    let result =
        call_add_funds(&mut test_runner,
                       docindexer,
                       &bob,
                       dec!(100),
                       true);

    if let BalanceChange::Fungible(change) = result
        .balance_changes()
        .get(&GlobalAddress::from(bob.account.clone())).unwrap()
        .get(&RADIX_TOKEN).unwrap() {
            assert_eq!(dec!(-100),
                       *change,
                       "Bob should be 100 XRD down");
        } else {
            panic!("Change should be fungible");
        }
    
    // Only admins (i.e. not Bob) can remove them
    call_remove_funds(&mut test_runner,
                      docindexer,
                      &bob,
                      &bob_nfgid,
                      dec!(100),
                      false);

    // Alice can remove them
    let result =
        call_remove_funds(&mut test_runner,
                          docindexer,
                          &alice,
                          &user1_nfgid,
                          dec!(100),
                          true);

    if let BalanceChange::Fungible(change) = result
        .balance_changes()
        .get(&GlobalAddress::from(alice.account.clone())).unwrap()
        .get(&RADIX_TOKEN).unwrap() {
            assert_eq!(dec!(100),
                       *change,
                       "Alice should be 100 XRD up");
        } else {
            panic!("Change should be fungible");
        }

}


/// This tests that the subsidy actually works, which involves the
/// `DocumentIndexer::add_funds`, `DocumentIndexer::add_document` and
/// `DocumentIndexer::register_document_edition` methods.
#[test]
fn test_subsidy() {
    let mut test_runner = TestRunner::builder().build();

    let package = test_runner.compile_and_publish(this_package!());

    // Alice will own the component
    let alice = User::new(&mut test_runner);
    let alice_nfgid =
        NonFungibleGlobalId::from_public_key(&alice.pubkey);

    // Bob is an anonymous user
    let bob = User::new(&mut test_runner);

    let users_resaddr =
        test_runner.create_non_fungible_resource(alice.account);
    let user1_nfgid = NonFungibleGlobalId::new(users_resaddr, 1.into());
    let user2_nfgid = NonFungibleGlobalId::new(users_resaddr, 2.into());
    give_nf_tokens(&mut test_runner,
                   &alice,
                   &bob,
                   &users_resaddr,
                   BTreeSet::from([2.into()]));

    let admins: BTreeSet<NonFungibleGlobalId> =
        std::iter::once(alice_nfgid.clone()).collect();

    let (docindexer, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 None,
                 &alice_nfgid,
                 admins,
                 &users_resaddr);

    

    // Alice adds a tiny bit of funds to the indexer to check it still
    // works when underfunded (this shouldn't result in subsidy)
    call_add_funds(&mut test_runner,
                   docindexer,
                   &alice,
                   dec!(1),
                   true);
    call_add_document_with_fee(&mut test_runner,
                               docindexer,
                               &alice,
                               &user1_nfgid,
                               "Test Document unsubsidized",
                               true);

    
    // Alice adds some real funds to the indexer so Bob has something
    // to play with.
    call_add_funds(&mut test_runner,
                   docindexer,
                   &alice,
                   dec!(100),
                   true);

    // Bob revels in subsidy, adding a document and its first edition
    // to the indexer.
    let bobs_starting_xrd_balance =
        *test_runner.get_component_resources(bob.account).get(&RADIX_TOKEN).unwrap();
    let funds_starting_xrd_balance =
        *test_runner.get_component_resources(docindexer).get(&RADIX_TOKEN).unwrap();

    call_add_document_with_fee(&mut test_runner,
                               docindexer,
                               &bob,
                               &user2_nfgid,
                               "Test Document will succeed",
                               true);

    call_register_document_edition_with_fee(&mut test_runner,
                                            docindexer,
                                            &bob,
                                            &user2_nfgid,
                                            "Test Document will succeed",
                                            "hashit",
                                            &HashMap::new(),
                                            true);

    let bobs_final_xrd_balance =
        *test_runner.get_component_resources(bob.account).get(&RADIX_TOKEN).unwrap();
    assert_eq!(bobs_starting_xrd_balance,
               bobs_final_xrd_balance,
               "Bob shouldn't have spent any XRD on this");

    let funds_final_xrd_balance =
        *test_runner.get_component_resources(docindexer).get(&RADIX_TOKEN).unwrap();
    assert!(funds_starting_xrd_balance > funds_final_xrd_balance,
            "Tx fees should have come from component funds");
}
