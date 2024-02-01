use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

use transaction::prelude::*;
use radix_engine::transaction::{CommitResult, BalanceChange};
use scrypto::api::ObjectModuleId;

use flexi_faucet::FaucetNfData;

/// A convenience struct for grouping together a user account's data.
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


/// Our badge requirement can be either fungible or non-fungible, and
/// this enum encapsulates that.
#[derive(Clone)]
enum BadgeSpec {
    Fungible(ResourceAddress, Decimal),
    NonFungible(NonFungibleGlobalId)
}


/// Mints a number of the faucet's non-fungible tokens, based on
/// `FaucetNfData` non-fungible data.
fn mint_ruid_non_fungibles(test_runner: &mut TestRunner,
                           notary: &User,
                           resource: ResourceAddress,
                           minting_badge: &NonFungibleGlobalId,
                           faucet: ComponentAddress,
                           amount: u64) {

    let mut minted = 0;

    // We mint in batches because there is a max-substates-write limit
    // that we might hit otherwise when making lots of NFTs.
    while minted < amount {
        let mut to_mint = amount - minted;
        if to_mint > 150 { to_mint = 150; }

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungibles(
                notary.account,
                minting_badge.resource_address(),
                &BTreeSet::from([minting_badge.local_id().clone()]))
            .mint_ruid_non_fungible(
                resource,
                (0..to_mint).map(
                    |_|  FaucetNfData {
                        faucet, attributes: HashMap::new()
                    })
                    .collect::<Vec<FaucetNfData>>())
            .deposit_batch(notary.account)
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
        );

        receipt.expect_commit_success();
        minted += to_mint;
    }
}

/// Creates a new non-fungible resource and returns it and one of its
/// non-fungibles.
fn create_nf_resource_and_badge(test_runner: &mut TestRunner, user: &User) ->
    (ResourceAddress, NonFungibleGlobalId)
{
    let resource = test_runner.create_non_fungible_resource(user.account);

    (resource,
     NonFungibleGlobalId::new(resource, 1.into()))
}

/// Creates a new fungible resource where the supplied
/// `minting_badges` resource has access to mint tokens. Everyone can
/// deposit and withdraw as well as change minter role. Everything
/// else is forbidden.
fn create_mintable_fungible_resource(
    test_runner: &mut TestRunner,
    notary: &User,
    minting_badges: ResourceAddress,
    supply: Option<Decimal>) -> ResourceAddress
{
    let roles = FungibleResourceRoles {
        mint_roles: mint_roles!(
            minter => rule!(require(minting_badges));
            minter_updater => rule!(allow_all);
        ),
        withdraw_roles: withdraw_roles!(
            withdrawer => rule!(allow_all);
            withdrawer_updater => rule!(deny_all);
        ),
        deposit_roles: deposit_roles!(
            depositor => rule!(allow_all);
            depositor_updater => rule!(deny_all);
        ),
        burn_roles: None,
        freeze_roles: None,
        recall_roles: None,
    };
    let manifest = ManifestBuilder::new()
        .create_fungible_resource(OwnerRole::None,
                                  true,
                                  18,
                                  roles,
                                  metadata!(),
                                  supply)
        .deposit_batch(notary.account)
        .build();
    
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    receipt.expect_commit_success().new_resource_addresses()[0]
}

/// Changes the minting role on a resource to require
/// `minting_badges`. Minting role update must be available to
/// everyone for this to work.
fn update_minting_role(
    test_runner: &mut TestRunner,
    notary: &User,
    resource: ResourceAddress,
    minting_badges: ResourceAddress)
{
    let manifest = ManifestBuilder::new()
        .update_role(resource,
                     ObjectModuleId::Main,
                     RoleKey::new(MINTER_ROLE),
                     rule!(require(minting_badges)))
        .build();
    
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    receipt.expect_commit_success();
}

/// Builds a transaction manifest for `FlexiFaucet::new` and runs
/// it. Sets minting badge to None. Always expects success.
fn call_new(test_runner: &mut TestRunner,
            package: PackageAddress,
            notary: &User,
            owner: &NonFungibleGlobalId,
            token: ResourceAddress,
            max_per_tap: Option<Decimal>,
            tap_cooldown_secs: Option<u64>,
            taker_badges: Option<ResourceAddress>,
            minimum_fungible_badge_tokens: Option<Decimal>)
            -> (ComponentAddress, CommitResult) {

    let manifest =
        ManifestBuilder::new()
        .call_function(
            package,
            "FlexiFaucet",
            "new",
            manifest_args!(
                owner,
                token,
                max_per_tap,
                tap_cooldown_secs,
                taker_badges,
                minimum_fungible_badge_tokens,
                None::<String>,
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

/// Builds a transaction manifest for
/// `FlexiFaucet::call_new_non_fungible_resource` and runs it. Always
/// expects success.
fn call_new_non_fungible_resource(
    test_runner: &mut TestRunner,
    package: PackageAddress,
    notary: &User,
    owner: &NonFungibleGlobalId,
    minting_badge: ResourceAddress,
    recallable: bool,
    name: Option<String>,
    description: Option<String>)
    -> (ResourceAddress, CommitResult) {

    let manifest =
        ManifestBuilder::new()
        .call_function(
            package,
            "FlexiFaucet",
            "new_non_fungible_resource",
            manifest_args!(
                rule!(require(owner.clone())),
                minting_badge,
                recallable,
                name,
                description,
            ),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    let result = receipt.expect_commit_success();
    let resource = receipt.expect_commit(true).new_resource_addresses()[0];

    (resource, result.clone())
}

/// Builds a transaction manifest for `FlexiFaucet::new` and runs
/// it. Supports fungible minting badges only. Always expects
/// success.
fn call_new_f(test_runner: &mut TestRunner,
              package: PackageAddress,
              notary: &User,
              owner: &NonFungibleGlobalId,
              token: ResourceAddress,
              max_per_tap: Option<Decimal>,
              tap_cooldown_secs: Option<u64>,
              taker_badges: Option<ResourceAddress>,
              minimum_fungible_badge_tokens: Option<Decimal>,
              minting_badge: (ResourceAddress, Decimal))
              -> (ComponentAddress, CommitResult) {

    let manifest =
        ManifestBuilder::new()
        .withdraw_from_account(
            notary.account,
            minting_badge.0,
            minting_badge.1)
        .take_all_from_worktop(minting_badge.0,
                               "minting_badge_bucket")
        .call_function_with_name_lookup(
            package,
            "FlexiFaucet",
            "new",
            |lookup| manifest_args!(
                owner,
                token,
                max_per_tap,
                tap_cooldown_secs,
                taker_badges,
                minimum_fungible_badge_tokens,
                Some(lookup.bucket("minting_badge_bucket")),
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

/// Builds a transaction manifest for `FlexiFaucet::new` and runs
/// it. Supports non-fungible minting badges only. Always expects
/// success.
fn call_new_nf(test_runner: &mut TestRunner,
               package: PackageAddress,
               notary: &User,
               owner: &NonFungibleGlobalId,
               token: ResourceAddress,
               max_per_tap: Option<Decimal>,
               tap_cooldown_secs: Option<u64>,
               taker_badges: Option<ResourceAddress>,
               minimum_fungible_badge_tokens: Option<Decimal>,
               minting_badge: &NonFungibleGlobalId)
               -> (ComponentAddress, CommitResult) {

    let manifest =
        ManifestBuilder::new()
        .withdraw_non_fungibles_from_account(
            notary.account,
            minting_badge.resource_address(),
            &BTreeSet::from([minting_badge.local_id().clone()]))
        .take_all_from_worktop(minting_badge.resource_address(),
                               "minting_badge_bucket")
        .call_function_with_name_lookup(
            package,
            "FlexiFaucet",
            "new",
            |lookup| manifest_args!(
                owner,
                token,
                max_per_tap,
                tap_cooldown_secs,
                taker_badges,
                minimum_fungible_badge_tokens,
                Some(lookup.bucket("minting_badge_bucket")),
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


/// Builds a transaction manifest for `FlexiFaucet::activate` and runs
/// it.
fn call_activate(test_runner: &mut TestRunner,
                 flexifaucet: ComponentAddress,
                 notary: &User,
                 user: &NonFungibleGlobalId,
                 succeed: bool)
            -> CommitResult {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            notary.account,
            user.resource_address(),
            &BTreeSet::from([user.local_id().clone()]))
        .call_method(
            flexifaucet,
            "activate",
            manifest_args!(),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() }
    .clone()
}

/// Builds a transaction manifest for `FlexiFaucet::deactivate` and
/// runs it.
fn call_deactivate(test_runner: &mut TestRunner,
                   flexifaucet: ComponentAddress,
                   notary: &User,
                   user: &NonFungibleGlobalId,
                   succeed: bool)
            -> CommitResult {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            notary.account,
            user.resource_address(),
            &BTreeSet::from([user.local_id().clone()]))
        .call_method(
            flexifaucet,
            "deactivate",
            manifest_args!(),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() }
    .clone()
}

/// Builds a transaction manifest for `FlexiFaucet::active` and runs
/// it, returning its return value.
fn call_active(test_runner: &mut TestRunner,
               flexifaucet: ComponentAddress,
               notary: &User)
            -> (bool, CommitResult) {
    let manifest = ManifestBuilder::new()
        .call_method(
            flexifaucet,
            "active",
            manifest_args!(),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    let result = 
        receipt.expect_commit_success();

    (result.output(1), result.clone())
}

/// Builds a transaction manifest for
/// `FlexiFaucet::minting_badge_as_fungible` and runs it, returning
/// its return value if successful.
fn call_minting_badge_as_fungible(
    test_runner: &mut TestRunner,
    flexifaucet: ComponentAddress,
    notary: &User)
    -> (Option<(ResourceAddress, Decimal)>, CommitResult)
{
    let manifest = ManifestBuilder::new()
        .call_method(
            flexifaucet,
            "minting_badge_as_fungible",
            manifest_args!(),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    let result = 
        receipt.expect_commit_success();

    (result.output::<Option<(ResourceAddress, Decimal)>>(1)
     .map(|(addr, amount)| (addr, amount)),
     result.clone())
}

/// Builds a transaction manifest for
/// `FlexiFaucet::minting_badge_as_non_fungible` and runs it,
/// returning its return value. Always expects success.
fn call_minting_badge_as_non_fungible(
    test_runner: &mut TestRunner,
    flexifaucet: ComponentAddress,
    notary: &User)
    -> (Option<(ResourceAddress, BTreeSet<NonFungibleLocalId>)>,
        CommitResult)
{
        let manifest = ManifestBuilder::new()
        .call_method(
            flexifaucet,
            "minting_badge_as_non_fungible",
            manifest_args!(),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    let result = 
        receipt.expect_commit_success();

    (result.output::<Option<(ResourceAddress, BTreeSet<NonFungibleLocalId>)>>(1)
     .map(|(addr, amount)| (addr, amount)),
     result.clone())
}

/// Builds a transaction manifest for `FlexiFaucet::set_minting_badge`
/// to clear the minting badge for the faucet, and runs it.
fn call_set_minting_badge_to_none(test_runner: &mut TestRunner,
                                  flexifaucet: ComponentAddress,
                                  notary: &User,
                                  user: &NonFungibleGlobalId,
                                  succeed: bool)
                            -> CommitResult {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            notary.account,
            user.resource_address(),
            &BTreeSet::from([user.local_id().clone()]))
        .call_method(
            flexifaucet,
            "set_minting_badge",
            manifest_args!(None::<String>))
        .deposit_batch(notary.account)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() }
    .clone()
}

/// Builds a transaction manifest for `FlexiFaucet::set_minting_badge`
/// to set a fungible minting badge, and runs it.
fn call_set_minting_badge_f(test_runner: &mut TestRunner,
                            flexifaucet: ComponentAddress,
                            notary: &User,
                            user: &NonFungibleGlobalId,
                            minting_badge: (ResourceAddress, Decimal),
                            succeed: bool)
                            -> CommitResult {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            notary.account,
            user.resource_address(),
            &BTreeSet::from([user.local_id().clone()]))
        .withdraw_from_account(
            notary.account,
            minting_badge.0,
            minting_badge.1)
        .take_all_from_worktop(minting_badge.0,
                               "minting_badge_bucket")
        .call_method_with_name_lookup(
            flexifaucet,
            "set_minting_badge",
            |lookup| manifest_args!(Some(lookup.bucket("minting_badge_bucket"))))
        .deposit_batch(notary.account)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() }
    .clone()
}

/// Builds a transaction manifest for `FlexiFaucet::set_minting_badge`
/// to set a non-fungible minting badge, and runs it.
fn call_set_minting_badge_nf(test_runner: &mut TestRunner,
                             flexifaucet: ComponentAddress,
                             notary: &User,
                             user: &NonFungibleGlobalId,
                             minting_badge: &NonFungibleGlobalId,
                             succeed: bool)
                            -> CommitResult {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            notary.account,
            user.resource_address(),
            &BTreeSet::from([user.local_id().clone()]))
        .withdraw_non_fungibles_from_account(
            notary.account,
            minting_badge.resource_address(),
            &BTreeSet::from([minting_badge.local_id().clone()]))
        .take_all_from_worktop(minting_badge.resource_address(),
                               "minting_badge_bucket")
        .call_method_with_name_lookup(
            flexifaucet,
            "set_minting_badge",
            |lookup| manifest_args!(Some(lookup.bucket("minting_badge_bucket"))))
        .deposit_batch(notary.account)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() }
    .clone()
}

/// Builds a transaction manifest for
/// `FlexiFaucet::treasury_as_fungible`and runs it, returning the
/// return value. Always expects success.
fn call_treasury_as_fungible(test_runner: &mut TestRunner,
                             flexifaucet: ComponentAddress,
                             notary: &User)
                             -> (Decimal, CommitResult) {
    let manifest = ManifestBuilder::new()
        .call_method(
            flexifaucet,
            "treasury_as_fungible",
            manifest_args!(),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    let result =  receipt.expect_commit_success();

    (result.output(1), result.clone())
}

/// Builds a transaction manifest for
/// `FlexiFaucet::treasury_as_non_fungible`and runs it, returning the
/// return value. Always expects success.
fn call_treasury_as_non_fungible(
    test_runner: &mut TestRunner,
    flexifaucet: ComponentAddress,
    notary: &User)
    -> (BTreeSet<NonFungibleLocalId>,
        CommitResult)
{
    let manifest = ManifestBuilder::new()
        .call_method(
            flexifaucet,
            "treasury_as_non_fungible",
            manifest_args!(),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    let result =  receipt.expect_commit_success();

    (result.output(1), result.clone())
}

/// Builds a transaction manifest for `FlexiFaucet::fund_treasury`
/// with fungible tokens, and runs it.
fn call_fund_treasury_f(test_runner: &mut TestRunner,
                        flexifaucet: ComponentAddress,
                        notary: &User,
                        owner: &NonFungibleGlobalId,
                        funds: (ResourceAddress, Decimal),
                        succeed: bool)
                        -> CommitResult {
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(
            notary.account,
            funds.0,
            funds.1)
        .take_all_from_worktop(funds.0,
                               "funds_bucket")
        .create_proof_from_account_of_non_fungibles(
            notary.account,
            owner.resource_address(),
            &BTreeSet::from([owner.local_id().clone()]))
        .call_method_with_name_lookup(
            flexifaucet,
            "fund_treasury",
            |lookup| manifest_args!(lookup.bucket("funds_bucket")))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() }
    .clone()
}

/// Builds a transaction manifest for `FlexiFaucet::fund_treasury`
/// with non-fungible tokens, and runs it.
fn call_fund_treasury_nf(test_runner: &mut TestRunner,
                         flexifaucet: ComponentAddress,
                         notary: &User,
                         owner: &NonFungibleGlobalId,
                         funds: (ResourceAddress, BTreeSet<NonFungibleLocalId>),
                         succeed: bool)
                         -> CommitResult {
    let manifest = ManifestBuilder::new()
        .withdraw_non_fungibles_from_account(
            notary.account,
            funds.0,
            &funds.1)
        .take_all_from_worktop(funds.0,
                               "funds_bucket")
        .create_proof_from_account_of_non_fungibles(
            notary.account,
            owner.resource_address(),
            &BTreeSet::from([owner.local_id().clone()]))
        .call_method_with_name_lookup(
            flexifaucet,
            "fund_treasury",
            |lookup| manifest_args!(lookup.bucket("funds_bucket")))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() }
    .clone()
}

/// Builds a transaction manifest for
/// `FlexiFaucet::defund_treasury_fungible` and runs it.
fn call_defund_treasury_fungible(
    test_runner: &mut TestRunner,
    flexifaucet: ComponentAddress,
    notary: &User,
    user: &NonFungibleGlobalId,
    amount: Decimal,
    succeed: bool)
    -> CommitResult {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            notary.account,
            user.resource_address(),
            &BTreeSet::from([user.local_id().clone()]))
        .call_method(
            flexifaucet,
            "defund_treasury_fungible",
            manifest_args!(amount))
        .deposit_batch(notary.account)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() }
    .clone()
}

/// Builds a transaction manifest for
/// `FlexiFaucet::defund_treasury_non_fungible` and runs it.
fn call_defund_treasury_non_fungible(
    test_runner: &mut TestRunner,
    flexifaucet: ComponentAddress,
    notary: &User,
    user: &NonFungibleGlobalId,
    nflids: BTreeSet<NonFungibleLocalId>,
    succeed: bool)
    -> CommitResult {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            notary.account,
            user.resource_address(),
            &BTreeSet::from([user.local_id().clone()]))
        .call_method(
            flexifaucet,
            "defund_treasury_non_fungible",
            manifest_args!(nflids))
        .deposit_batch(notary.account)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() }
    .clone()
}

/// Builds a transaction manifest for `FlexiFaucet::empty_treasury`
/// and runs it.
fn call_empty_treasury(
    test_runner: &mut TestRunner,
    flexifaucet: ComponentAddress,
    notary: &User,
    user: &NonFungibleGlobalId,
    succeed: bool)
    -> CommitResult {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            notary.account,
            user.resource_address(),
            &BTreeSet::from([user.local_id().clone()]))
        .call_method(
            flexifaucet,
            "empty_treasury",
            manifest_args!())
        .deposit_batch(notary.account)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() }
    .clone()
}

/// Builds a transaction manifest for `FlexiFaucet::tap` and runs it.
fn call_tap(
    test_runner: &mut TestRunner,
    flexifaucet: ComponentAddress,
    notary: &User,
    badge: Option<BadgeSpec>,
    amount: Decimal,
    succeed: bool)
    -> CommitResult {
    let mut manifest = ManifestBuilder::new();
    if let Some(user) = &badge {
        manifest =
            match user {
                BadgeSpec::NonFungible(nfgid) => manifest
                    .create_proof_from_account_of_non_fungibles(
                        notary.account,
                        nfgid.resource_address(),
                        &BTreeSet::from([nfgid.local_id().clone()])),

                BadgeSpec::Fungible(resource, amount) => manifest
                    .create_proof_from_account_of_amount(
                        notary.account,
                        resource.clone(),
                        *amount)
            }
        .pop_from_auth_zone("user_proof")

    }
    let manifest = manifest
        .call_method_with_name_lookup(
            flexifaucet,
            "tap",
            |lookup| manifest_args!(
                if badge.is_some() { Some(lookup.proof("user_proof")) } else { None },
                amount))
        .deposit_batch(notary.account)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&notary.pubkey)],
    );

    if succeed != receipt.is_commit_success() {
        println!("RECEIPT: {:?}", receipt);
    }
    
    if succeed { receipt.expect_commit_success() }
    else { receipt.expect_commit_failure() }
    .clone()
}

static mut ROUND_COUNTER: u64 = 2;

/// Changes the test runner clock.
fn set_test_runner_clock(
    test_runner: &mut TestRunner,
    time_secs: i64) {
    unsafe {
        test_runner.advance_to_round_at_timestamp(
            Round::of(ROUND_COUNTER), // must be incremented each time
            time_secs * 1000,
        );
        ROUND_COUNTER += 1;
    }
}

/// Helper function to try to change rules and metadata in a resource
/// to see if we can, and asserts that we cannot for those
/// rules/metadata that are supposed to be locked.
///
/// The `succeed` parameter is subdivided as follows:
///
/// .0 Should change of `name` metadata succeed?
///
/// .1 Should change of ``description` metadata succeed?
///
/// .2 Should change of non-recall/freeze access rules succeed?
///
/// .3 Should change of recall and freeze access rules succeed?
fn assert_metadata_and_access_update(test_runner: &mut TestRunner,
                                     token_resaddr: ResourceAddress,
                                     manifest_preamble: impl Fn() -> ManifestBuilder,
                                     execution_proofs: Vec<NonFungibleGlobalId>,
                                     succeed: ( bool, bool, bool, bool )) {
    for (name, value, succeed) in
        [
            ("name", "a name", succeed.0 ),
            ("description", "a description", succeed.1 )
        ]
    {
        let manifest = manifest_preamble()
            .set_metadata(token_resaddr, name, value)
            .build();
        test_runner.execute_manifest_ignoring_fee(manifest, execution_proofs.clone())
            .expect_commit(succeed);
    }

    for (succeed, role) in
        [
            (succeed.2, MINTER_ROLE),
            (succeed.2, MINTER_UPDATER_ROLE),
            (succeed.2, BURNER_ROLE),
            (succeed.2, BURNER_UPDATER_ROLE),
            (succeed.2, WITHDRAWER_ROLE),
            (succeed.2, WITHDRAWER_UPDATER_ROLE),
            (succeed.2, DEPOSITOR_ROLE),
            (succeed.2, DEPOSITOR_UPDATER_ROLE),
            (succeed.3, RECALLER_ROLE),
            (succeed.3, RECALLER_UPDATER_ROLE),
            (succeed.3, FREEZER_ROLE),
            (succeed.3, FREEZER_UPDATER_ROLE),
            (succeed.2, NON_FUNGIBLE_DATA_UPDATER_ROLE),
            (succeed.2, NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE),
        ]
    {
            let manifest = manifest_preamble()
                .update_role(
                    token_resaddr, ObjectModuleId::Main, RoleKey::new(role),
                    rule!(allow_all))
                .build();
            test_runner.execute_manifest_ignoring_fee(manifest,
                                                      execution_proofs.clone())
                .expect_commit(succeed);
    }
}

/// Tests that `new_non_fungible_resource` when invoked to create a
/// fully flexible resource actually creates a resource the roles and
/// metadata of which can then be changed.
#[test]
fn test_new_non_fungible_resource_fully_flexible() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let ((owner_badges_resaddr, owner_badge_nfgid),
         (minting_badges_resaddr, _),
         (unrelated_badges_resaddr, unrelated_badge_nfgid)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    // Create a fully flexible resource
    let (token_resaddr, _) =
        call_new_non_fungible_resource(&mut test_runner,
                                       package,
                                       &alice,
                                       &owner_badge_nfgid,
                                       minting_badges_resaddr,
                                       true,
                                       None,
                                       None);


    // First we will try to change all the things using the wrong
    // proof. This should never work.
    
    // Making this a closure so we can use it many times (because
    // ManifestBuilder can't be cloned)
    let manifest_preamble = || ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            alice.account,
            unrelated_badges_resaddr,
            &BTreeSet::from([unrelated_badge_nfgid.local_id().clone()]));
    let execution_proofs = vec![NonFungibleGlobalId::from_public_key(&alice.pubkey)];

    assert_metadata_and_access_update(&mut test_runner,
                                      token_resaddr,
                                      manifest_preamble,
                                      execution_proofs.clone(),
                                      (false, false, false, false));

    // Now change all the things using the proper proof. This should
    // work.
    let manifest_preamble = || ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            alice.account,
            owner_badges_resaddr,
            &BTreeSet::from([owner_badge_nfgid.local_id().clone()]));

    assert_metadata_and_access_update(&mut test_runner,
                                      token_resaddr,
                                      manifest_preamble,
                                      execution_proofs,
                                      (true, true, true, true));
}

/// Tests that `new_non_fungible_resource` when invoked to create a
/// resource with locked metadata and locked recall/freeze actually
/// creates a resource on which those aspects cannot be changed.
#[test]
fn test_new_non_fungible_resource_locked_roles() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let ((owner_badges_resaddr, owner_badge_nfgid),
         (minting_badges_resaddr, _),
         (unrelated_badges_resaddr, unrelated_badge_nfgid)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    // Create a resource with locked name and description metadata,
    // and disabled recall/freeze access
    let (token_resaddr, _) =
        call_new_non_fungible_resource(&mut test_runner,
                                       package,
                                       &alice,
                                       &owner_badge_nfgid,
                                       minting_badges_resaddr,
                                       false,
                                       Some("my name".to_owned()),
                                       Some("my description".to_owned()));

    
    // First try with a badge that's not allowed
    let manifest_preamble = || ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            alice.account,
            unrelated_badges_resaddr,
            &BTreeSet::from([unrelated_badge_nfgid.local_id().clone()]));
    let execution_proofs = vec![NonFungibleGlobalId::from_public_key(&alice.pubkey)];

    assert_metadata_and_access_update(&mut test_runner,
                                      token_resaddr,
                                      manifest_preamble,
                                      execution_proofs.clone(),
                                      (false, false, false, false));

    // And then the proper proof. It should be able to change
    // everything except the locked metadata and the disabled access
    // rules.
    let manifest_preamble = || ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            alice.account,
            owner_badges_resaddr,
            &BTreeSet::from([owner_badge_nfgid.local_id().clone()]));

    assert_metadata_and_access_update(&mut test_runner,
                                      token_resaddr,
                                      manifest_preamble,
                                      execution_proofs.clone(),
                                      (false, false, true, false));

}

/// Tests that `new_non_fungible_resource` when invoked to create a
/// resource with only one locked metadata field ("name") creates a
/// resource on which that field cannot be changed.
#[test]
fn test_new_non_fungible_resource_lock_name_metadata_field() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);
    let execution_proofs = vec![NonFungibleGlobalId::from_public_key(&alice.pubkey)];

    let ((owner_badges_resaddr, owner_badge_nfgid),
         (minting_badges_resaddr, _)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    // Create a resource with locked name and description metadata,
    // and disabled recall/freeze access
    let (token_resaddr, _) =
        call_new_non_fungible_resource(&mut test_runner,
                                       package,
                                       &alice,
                                       &owner_badge_nfgid,
                                       minting_badges_resaddr,
                                       true,
                                       Some("my name".to_owned()),
                                       None);

    // We should be able to change everything except "name" metadata
    // field
    let manifest_preamble = || ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            alice.account,
            owner_badges_resaddr,
            &BTreeSet::from([owner_badge_nfgid.local_id().clone()]));

    assert_metadata_and_access_update(&mut test_runner,
                                      token_resaddr,
                                      manifest_preamble,
                                      execution_proofs.clone(),
                                      (false, true, true, true));
}

/// Tests that `new_non_fungible_resource` when invoked to create a
/// resource with only one locked metadata field ("description")
/// creates a resource on which that field cannot be changed.
#[test]
fn test_new_non_fungible_resource_lock_description_metadata_field() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);
    let execution_proofs = vec![NonFungibleGlobalId::from_public_key(&alice.pubkey)];

    let ((owner_badges_resaddr, owner_badge_nfgid),
         (minting_badges_resaddr, _)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    // Create a resource with locked name and description metadata,
    // and disabled recall/freeze access
    let (token_resaddr, _) =
        call_new_non_fungible_resource(&mut test_runner,
                                       package,
                                       &alice,
                                       &owner_badge_nfgid,
                                       minting_badges_resaddr,
                                       true,
                                       None,
                                       Some("my description".to_owned()));

    // We should be able to change everything except "description"
    // metadata field
    let manifest_preamble = || ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            alice.account,
            owner_badges_resaddr,
            &BTreeSet::from([owner_badge_nfgid.local_id().clone()]));

    assert_metadata_and_access_update(&mut test_runner,
                                      token_resaddr,
                                      manifest_preamble,
                                      execution_proofs.clone(),
                                      (true, false, true, true));
}

/// Tests that `new_non_fungible_resource` creates a resource that can
/// thereafter have tokens minted for it.
#[test]
fn test_new_non_fungible_resource_minting() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let ((_, owner_badge_nfgid),
         (minting_badges_resaddr, minting_badge_nfgid),
         (unrelated_badges_resaddr, unrelated_badge_nfgid)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    // Testing mint

    // Create a resource (flexible or not doesn't matter)
    let (token_resaddr, _) =
        call_new_non_fungible_resource(&mut test_runner,
                                       package,
                                       &alice,
                                       &owner_badge_nfgid,
                                       minting_badges_resaddr,
                                       true,
                                       None,
                                       None);
    let execution_proofs = vec![NonFungibleGlobalId::from_public_key(&alice.pubkey)];


    // Mint with unrelated badge should fail
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            alice.account,
            unrelated_badges_resaddr,
            &BTreeSet::from([unrelated_badge_nfgid.local_id().clone()]))
        .mint_ruid_non_fungible(token_resaddr, [FaucetNfData{
            faucet:alice.account, attributes:HashMap::new()}])
        .deposit_batch(alice.account)
        .build();
    test_runner.execute_manifest_ignoring_fee(manifest, execution_proofs.clone())
        .expect_commit(false);
    

    // Mint with minting badge should succeed
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            alice.account,
            minting_badges_resaddr,
            &BTreeSet::from([minting_badge_nfgid.local_id().clone()]))
        .mint_ruid_non_fungible(token_resaddr, [FaucetNfData{
            faucet:alice.account, attributes:HashMap::new()}])
        .deposit_batch(alice.account)
        .build();
    test_runner.execute_manifest_ignoring_fee(manifest, execution_proofs.clone())
        .expect_commit(true);
}

/// Tests that `activate`, `deactivate` and `active` all do what
/// they're supposed to.
#[test]
fn test_activate_deactivate() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let (_, owner_nfgid) =
        create_nf_resource_and_badge(&mut test_runner, &alice);

    let (flexifaucet, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 &owner_nfgid,
                 RADIX_TOKEN,
                 None, None, None, None);

    assert!(!call_active(&mut test_runner, flexifaucet, &alice).0,
            "Faucet should start inactive");
    
    call_activate(&mut test_runner,
                  flexifaucet,
                  &alice,
                  &owner_nfgid,
                  true);

    assert!(call_active(&mut test_runner, flexifaucet, &alice).0,
            "Faucet should now be active");

    call_activate(&mut test_runner,
                  flexifaucet,
                  &alice,
                  &owner_nfgid,
                  true);

    assert!(call_active(&mut test_runner, flexifaucet, &alice).0,
            "Faucet should still be active");

    call_deactivate(&mut test_runner,
                    flexifaucet,
                    &alice,
                    &owner_nfgid,
                    true);
    
    assert!(!call_active(&mut test_runner, flexifaucet, &alice).0,
            "Faucet should be inactive again");

    call_deactivate(&mut test_runner,
                    flexifaucet,
                    &alice,
                    &owner_nfgid,
                    true);
    
    assert!(!call_active(&mut test_runner, flexifaucet, &alice).0,
            "Faucet should still be inactive");
}


/// Tests that a faucet that starts out with a non-fungible minting
/// badge can have its minting badge successfully altered into other
/// non-fungibles, into a fungible, and also into not having a minting
/// badge.
#[test]
fn test_set_minting_badge_initial_non_fungible() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    // Start with non-fungible badge
    let nfbadges_resaddr =
        test_runner.create_non_fungible_resource(alice.account);
    let owner_badge_nfgid =
        NonFungibleGlobalId::new(nfbadges_resaddr, 1.into());
    let minting_badge1_nfgid =
        NonFungibleGlobalId::new(nfbadges_resaddr, 2.into());
    let minting_badge2_nfgid =
        NonFungibleGlobalId::new(nfbadges_resaddr, 3.into());

    let (flexifaucet1, _) =
        call_new_nf(&mut test_runner,
                    package,
                    &alice,
                    &owner_badge_nfgid,
                    RADIX_TOKEN,
                    None, None, None, None,
                    &minting_badge1_nfgid);

    let (minting_badge, _) =
        call_minting_badge_as_non_fungible(&mut test_runner, flexifaucet1, &alice);

    let minting_badge = minting_badge.unwrap();
    assert_eq!(minting_badge1_nfgid.resource_address(),
               minting_badge.0,
               "Minting badge should be correct resource");
    assert_eq!(1,
               minting_badge.1.len(),
               "Minting badge should have exactly one nflid");
    assert_eq!(minting_badge1_nfgid.local_id(),
               minting_badge.1.first().unwrap(),
               "Minting badge should be correct nflid");

    // Second nf badge resource
    call_set_minting_badge_nf(&mut test_runner,
                              flexifaucet1,
                              &alice,
                              &owner_badge_nfgid,
                              &minting_badge2_nfgid,
                              true);

    let (minting_badge, _) =
        call_minting_badge_as_non_fungible(&mut test_runner, flexifaucet1, &alice);

    let minting_badge = minting_badge.unwrap();
    assert_eq!(minting_badge2_nfgid.resource_address(),
               minting_badge.0,
               "Minting badge should be correct resource");
    assert_eq!(1,
               minting_badge.1.len(),
               "Minting badge should have exactly one nflid");
    assert_eq!(minting_badge2_nfgid.local_id(),
               minting_badge.1.first().unwrap(),
               "Minting badge should be correct nflid");
    
    // Switch to fungible badge resource
    let fbadges1_resaddr =
        test_runner.create_fungible_resource(dec!(10), 0, alice.account);

    call_set_minting_badge_f(&mut test_runner,
                             flexifaucet1,
                             &alice,
                             &owner_badge_nfgid,
                             (fbadges1_resaddr, dec!(1)),
                             true);

    let (minting_badge, _) =
        call_minting_badge_as_fungible(&mut test_runner, flexifaucet1, &alice);

    let minting_badge = minting_badge.unwrap();
    assert_eq!(fbadges1_resaddr,
               minting_badge.0,
               "Minting badge should be correct resource");
    assert_eq!(dec!(1),
               minting_badge.1,
               "Minting badge should be correct amount");

    // Switch to no badge
    call_set_minting_badge_to_none(&mut test_runner,
                                   flexifaucet1,
                                   &alice,
                                   &owner_badge_nfgid,
                                   true);

    let (minting_badge, _) =
        call_minting_badge_as_fungible(&mut test_runner, flexifaucet1, &alice);

    assert!(minting_badge.is_none(),
            "Should be no minting badge");
}

/// Tests that a faucet that starts out with a fungible minting
/// badge can have its minting badge successfully altered into other
/// fungibles, into a non-fungible, and also into not having a minting
/// badge.
#[test]
fn test_set_minting_badge_initial_fungible() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let (nfbadges_resaddr, owner_badge_nfgid) =
        create_nf_resource_and_badge(&mut test_runner, &alice);

    let minting_badge2_nfgid =
        NonFungibleGlobalId::new(nfbadges_resaddr, 3.into());

    // Start with fungible badge

    // First f-badge
    let fbadges1_resaddr =
        test_runner.create_fungible_resource(dec!(10), 0, alice.account);

    let (flexifaucet, _) =
        call_new_f(&mut test_runner,
                   package,
                   &alice,
                   &owner_badge_nfgid,
                   RADIX_TOKEN,
                   None, None, None, None,
                   (fbadges1_resaddr, dec!(3)));

    let (minting_badge, _) =
        call_minting_badge_as_fungible(&mut test_runner, flexifaucet, &alice);

    let minting_badge = minting_badge.unwrap();
    assert_eq!(fbadges1_resaddr,
               minting_badge.0,
               "Minting badge should be correct resource");
    assert_eq!(dec!(3),
               minting_badge.1,
               "Minting badge should be correct amount");

    
    // Second f-badge
    let fbadges2_resaddr =
        test_runner.create_fungible_resource(dec!(10), 0, alice.account);

    call_set_minting_badge_f(&mut test_runner,
                             flexifaucet,
                             &alice,
                             &owner_badge_nfgid,
                             (fbadges2_resaddr, dec!(2)),
                             true);

    let (minting_badge, _) =
        call_minting_badge_as_fungible(&mut test_runner, flexifaucet, &alice);

    let minting_badge = minting_badge.unwrap();
    assert_eq!(fbadges2_resaddr,
               minting_badge.0,
               "Minting badge should be correct resource");
    assert_eq!(dec!(2),
               minting_badge.1,
               "Minting badge should be correct amount");

    
    // Third f-badge
    let fbadges3_resaddr =
        test_runner.create_fungible_resource(dec!(10), 0, alice.account);

    call_set_minting_badge_f(&mut test_runner,
                             flexifaucet,
                             &alice,
                             &owner_badge_nfgid,
                             (fbadges3_resaddr, dec!(5)),
                             true);

    let (minting_badge, _) =
        call_minting_badge_as_fungible(&mut test_runner, flexifaucet, &alice);

    let minting_badge = minting_badge.unwrap();
    assert_eq!(fbadges3_resaddr,
               minting_badge.0,
               "Minting badge should be correct resource");
    assert_eq!(dec!(5),
               minting_badge.1,
               "Minting badge should be correct amount");


    // Then no badge
    call_set_minting_badge_to_none(&mut test_runner,
                                   flexifaucet,
                                   &alice,
                                   &owner_badge_nfgid,
                                   true);

    let (minting_badge, _) =
        call_minting_badge_as_fungible(&mut test_runner, flexifaucet, &alice);

    assert!(minting_badge.is_none(),
            "Should be no minting badge");


    // Fourth f-badge
    let fbadges4_resaddr =
        test_runner.create_fungible_resource(dec!(10), 0, alice.account);

    call_set_minting_badge_f(&mut test_runner,
                             flexifaucet,
                             &alice,
                             &owner_badge_nfgid,
                             (fbadges4_resaddr, dec!(7)),
                             true);

    let (minting_badge, _) =
        call_minting_badge_as_fungible(&mut test_runner, flexifaucet, &alice);

    let minting_badge = minting_badge.unwrap();
    assert_eq!(fbadges4_resaddr,
               minting_badge.0,
               "Minting badge should be correct resource");
    assert_eq!(dec!(7),
               minting_badge.1,
               "Minting badge should be correct amount");

    // Switch to nf minting badge.
    call_set_minting_badge_nf(&mut test_runner,
                              flexifaucet,
                              &alice,
                              &owner_badge_nfgid,
                              &minting_badge2_nfgid,
                              true);

    let (minting_badge, _) =
        call_minting_badge_as_non_fungible(&mut test_runner, flexifaucet, &alice);

    let minting_badge = minting_badge.unwrap();
    assert_eq!(minting_badge2_nfgid.resource_address(),
               minting_badge.0,
               "Minting badge should be correct resource");
    assert_eq!(1,
               minting_badge.1.len(),
               "Minting badge should have exactly one nflid");
    assert_eq!(minting_badge2_nfgid.local_id(),
               minting_badge.1.first().unwrap(),
               "Minting badge should be correct nflid");
}


/// Tests `fund_treasury`, `defund_treasury_fungible`,
/// `empty_treasury` and `treasury_as_fungible` on a fungible
/// treasury.
#[test]
fn test_fund_defund_treasury_fungible() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let (owner_badge_resaddr, owner_badge_nfgid) =
        create_nf_resource_and_badge(&mut test_runner, &alice);

    let token_resaddr = create_mintable_fungible_resource(
        &mut test_runner,
        &alice,
        owner_badge_resaddr,
        Some(dec!(1000)));
    
    let (flexifaucet, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 &owner_badge_nfgid,
                 token_resaddr,
                 None, None, None, None);

    // Testing f and nf read of empty treasury
    assert_eq!(dec!(0),
               call_treasury_as_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Starting treasury should be empty");
    assert_eq!(0,
               call_treasury_as_non_fungible(&mut test_runner, flexifaucet, &alice)
               .0.len(),
               "Starting treasury should be empty");

    // Funding with wrong token should fail
    call_fund_treasury_f(&mut test_runner,
                         flexifaucet,
                         &alice,
                         &owner_badge_nfgid,
                         (RADIX_TOKEN, dec!(100)),
                         false);

    // Funding with correct token should succeed
    call_fund_treasury_f(&mut test_runner,
                         flexifaucet,
                         &alice,
                         &owner_badge_nfgid,
                         (token_resaddr, dec!(100)),
                         true);

    assert_eq!(dec!(100),
               call_treasury_as_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now have funds");

    // Adding more funds should succeed
    call_fund_treasury_f(&mut test_runner,
                         flexifaucet,
                         &alice,
                         &owner_badge_nfgid,
                         (token_resaddr, dec!(10)),
                         true);

    assert_eq!(dec!(110),
               call_treasury_as_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now have more funds");

    // Removing funds should succeed
    call_defund_treasury_fungible(&mut test_runner,
                                  flexifaucet,
                                  &alice,
                                  &owner_badge_nfgid,
                                  dec!(50),
                                  true);
    assert_eq!(dec!(60),
               call_treasury_as_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now have less funds");

    // Removing non-fungible funds should fail
    call_defund_treasury_non_fungible(&mut test_runner,
                                      flexifaucet,
                                      &alice,
                                      &owner_badge_nfgid,
                                      BTreeSet::from(
                                          [owner_badge_nfgid.local_id().clone()]),
                                      false);

    // Removing everything should succeed
    call_empty_treasury(&mut test_runner,
                        flexifaucet,
                        &alice,
                        &owner_badge_nfgid,
                        true);

    assert_eq!(dec!(0),
               call_treasury_as_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now be empty");
}

/// Tests `fund_treasury`, `defund_treasury_non_fungible`,
/// `empty_treasury`, `treasury_as_non_fungible` and
/// `treasury_as_fungible` on a non-fungible treasury.
#[test]
fn test_fund_defund_treasury_non_fungible() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let ((_, owner_badge_nfgid),
         (token_resaddr, _),
         (other_nf_resaddr, _)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    let (flexifaucet, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 &owner_badge_nfgid,
                 token_resaddr,
                 None, None, None, None);

    // Testing f and nf read of empty treasury
    assert_eq!(dec!(0),
               call_treasury_as_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Starting treasury should be empty");
    assert_eq!(0,
               call_treasury_as_non_fungible(&mut test_runner, flexifaucet, &alice)
               .0.len(),
               "Starting treasury should be empty");

    // Funding with wrong token should fail
    call_fund_treasury_nf(&mut test_runner,
                          flexifaucet,
                          &alice,
                          &owner_badge_nfgid,
                          (other_nf_resaddr, BTreeSet::from([1.into()])),
                          false);

    // Funding with correct token should succeed
    call_fund_treasury_nf(&mut test_runner,
                          flexifaucet,
                          &alice,
                          &owner_badge_nfgid,
                          (token_resaddr, BTreeSet::from([1.into()])),
                          true);

    assert_eq!(dec!(1),
               call_treasury_as_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now have funds");
    assert_eq!(BTreeSet::from([1.into()]),
               call_treasury_as_non_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now have funds");

    // Adding more funds should succeed
    call_fund_treasury_nf(&mut test_runner,
                          flexifaucet,
                          &alice,
                          &owner_badge_nfgid,
                          (token_resaddr, BTreeSet::from([2.into(), 3.into()])),
                          true);

    assert_eq!(dec!(3),
               call_treasury_as_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now have funds");
    assert_eq!(BTreeSet::from([1.into(), 2.into(), 3.into()]),
               call_treasury_as_non_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now have funds");

    // Removing funds should succeed
    call_defund_treasury_non_fungible(&mut test_runner,
                                      flexifaucet,
                                      &alice,
                                      &owner_badge_nfgid,
                                      BTreeSet::from([1.into()]),
                                      true);
    assert_eq!(dec!(2),
               call_treasury_as_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now have less funds");
    assert_eq!(BTreeSet::from([2.into(), 3.into()]),
               call_treasury_as_non_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now have less funds");

    // Removing as fungible should also work
    call_defund_treasury_fungible(&mut test_runner,
                                  flexifaucet,
                                  &alice,
                                  &owner_badge_nfgid,
                                  dec!(1),
                                  true);
    assert_eq!(dec!(1),
               call_treasury_as_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now have even less funds");

    // Removing everything should succeed
    call_empty_treasury(&mut test_runner,
                        flexifaucet,
                        &alice,
                        &owner_badge_nfgid,
                        true);

    assert_eq!(dec!(0),
               call_treasury_as_fungible(&mut test_runner, flexifaucet, &alice).0,
               "Treasury should now be empty");
}


/// Tests that we can successfully tap from a fungibles faucet that
/// doesn't have a max per-tap limit. Tests tapping from treasury,
/// from minting, and from both combined. Also tests that we can
/// switch out the minting badge and tapping from mint still works.
#[test]
fn test_tap_fungible_with_no_limits() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let ((_, owner_badge_nfgid),
         (minting_badges_resaddr, minting_badge_nfgid)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    let token_resaddr =
        create_mintable_fungible_resource(&mut test_runner,
                                          &alice,
                                          minting_badges_resaddr,
                                          Some(dec!(1_000_000)));

    // This faucet has no restrictions on tap
    let (flexifaucet, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 &owner_badge_nfgid,
                 token_resaddr,
                 None, None, None, None);

    call_activate(&mut test_runner,
                  flexifaucet,
                  &alice,
                  &owner_badge_nfgid,
                  true);
    
    // Without treasury or mint, should fail
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(1),
             false);

    call_fund_treasury_f(&mut test_runner,
                         flexifaucet,
                         &alice,
                         &owner_badge_nfgid,
                         (token_resaddr, dec!(1000)),
                         true);

    // Should take from treasury
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 None,
                 dec!(10),
                 true);

    if let BalanceChange::Fungible(change) = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(dec!(10), *change, "Alice should be 10 tokens up");
        } else {
            panic!("Change should be fungible");
        }

    // Should fail because it's too much and we have no mint yet
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(1_000_000),
             false);

    // Start the mint
    call_set_minting_badge_nf(&mut test_runner,
                              flexifaucet,
                              &alice,
                              &owner_badge_nfgid,
                              &minting_badge_nfgid,
                              true);

    // And now we can tap a million easy

    let tokens_in_faucet = test_runner.get_component_resources(flexifaucet)
        .get(&token_resaddr).unwrap().clone();
    assert_eq!(dec!(990), tokens_in_faucet, "Treasury should be stocked");

    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 None,
                 dec!(1_000_000),
                 true);

    if let BalanceChange::Fungible(change) = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(dec!(1_000_000), *change, "Alice should be 1e6 tokens up");
        } else {
            panic!("Change should be fungible");
        }

    let tokens_in_faucet = test_runner.get_component_resources(flexifaucet)
        .get(&token_resaddr).unwrap().clone();
    assert_eq!(dec!(0), tokens_in_faucet, "Treasury should be empty");


    // Now we change the token resource from using nf badges for
    // minting to using a different, fungible, badge resource
    
    let minting_badges_resaddr =
        test_runner.create_fungible_resource(dec!(100), 0, alice.account);
    
    update_minting_role(&mut test_runner,
                        &alice,
                        token_resaddr,
                        minting_badges_resaddr);

    // Since we haven't given the faucet the new minting badge yet,
    // tap should fail as the old badge is bad.
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(10),
             false);

    // Give a new minting badge to the faucet
    call_set_minting_badge_f(&mut test_runner,
                             flexifaucet,
                             &alice,
                             &owner_badge_nfgid,
                             (minting_badges_resaddr, dec!(1)),
                             true);

    // And tap should work again
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 None,
                 dec!(25_000),
                 true);

    if let BalanceChange::Fungible(change) = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(dec!(25_000), *change, "Alice should be 25k tokens up");
        } else {
            panic!("Change should be fungible");
        }
}

/// Tests that we can successfully tap from a fungibles faucet that
/// has a max per-tap limit. Tests tapping from treasury, from
/// minting, and from both combined.
#[test]
fn test_tap_fungible_with_max_limit() {
    let mut test_runner = TestRunner::builder().build();

    let package = test_runner.compile_and_publish(this_package!());

    let alice = User::new(&mut test_runner);

    let ((_, owner_badge_nfgid),
         (minting_badges_resaddr, _)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    let token_resaddr =
        create_mintable_fungible_resource(&mut test_runner,
                                          &alice,
                                          minting_badges_resaddr,
                                          Some(dec!(1_000_000)));

    // This faucet has max 10 tokens per tap
    let (flexifaucet, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 &owner_badge_nfgid,
                 token_resaddr,
                 Some(dec!(10)),
                 None, None, None);

    call_activate(&mut test_runner,
                  flexifaucet,
                  &alice,
                  &owner_badge_nfgid,
                  true);
    
    call_fund_treasury_f(&mut test_runner,
                         flexifaucet,
                         &alice,
                         &owner_badge_nfgid,
                         (token_resaddr, dec!(30)),
                         true);

    // Should work
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 None,
                 dec!(1),
                 true);

    if let BalanceChange::Fungible(change) = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(dec!(1), *change, "Alice should be 1 token up");
        } else {
            panic!("Change should be fungible");
        }

    // Should work again
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 None,
                 dec!(10),
                 true);

    if let BalanceChange::Fungible(change) = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(dec!(10), *change, "Alice should be 10 coins up");
        } else {
            panic!("Change should be fungible");
        }

    // Should fail because it's beyond the max-per-tap limit of 10
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(11),
             false);

    // We want there to be 9 tokens left in the faucet
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(10),
             true);
    let tokens_in_faucet = test_runner.get_component_resources(flexifaucet)
        .get(&token_resaddr).unwrap().clone();
    assert_eq!(dec!(9), tokens_in_faucet, "Treasury must have 9 tokens");

    // And we give the faucet the ability to mint
    call_set_minting_badge_f(&mut test_runner,
                             flexifaucet,
                             &alice,
                             &owner_badge_nfgid,
                             (minting_badges_resaddr, dec!(1)),
                             true);

    // A malfunctioning faucet might give us the 11 tokens we ask for
    // at this point (if it gets confused when it's taking both from
    // treasury and from mint) so we test this.
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(11),
             false);

    // Now lets empty out the treasury totally
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(10),
             true);

    // Again, it's important that the treasury is now empty
    let tokens_in_faucet = test_runner.get_component_resources(flexifaucet)
        .get(&token_resaddr).unwrap().clone();
    assert_eq!(dec!(0), tokens_in_faucet, "Treasury must be empty");

    // And we check that a minting-only faucet also enforces the
    // max-per-tap limit
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(11),
             false);
}

/// Tests that we can successfully tap from a non-fungibles faucet
/// that doesn't have a max per-tap limit. Tests tapping from
/// treasury, from minting, and from both combined.
#[test]
fn test_tap_non_fungible_with_no_limits() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let ((_, owner_badge_nfgid),
         (minting_badges_resaddr, minting_badge_nfgid)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    let (token_resaddr, _) =
        call_new_non_fungible_resource(&mut test_runner,
                                       package,
                                       &alice,
                                       &owner_badge_nfgid,
                                       minting_badges_resaddr,
                                       false,
                                       None,
                                       None);

    // This faucet has no restrictions on tap
    let (flexifaucet, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 &owner_badge_nfgid,
                 token_resaddr,
                 None, None, None, None);

    mint_ruid_non_fungibles(&mut test_runner,
                            &alice,
                            token_resaddr,
                            &minting_badge_nfgid,
                            flexifaucet,
                            10000);

    call_activate(&mut test_runner,
                  flexifaucet,
                  &alice,
                  &owner_badge_nfgid,
                  true);
    
    // Without treasury or mint, should fail
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(1),
             false);

    call_fund_treasury_f(&mut test_runner,
                         flexifaucet,
                         &alice,
                         &owner_badge_nfgid,
                         (token_resaddr, dec!(100)),
                         true);

    // Should take from treasury
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 None,
                 dec!(60),
                 true);

    if let BalanceChange::NonFungible{ added, .. } = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(60, added.len(), "Alice should be 60 tokens up");
        } else {
            panic!("Change should be non-fungible");
        }

    // Should fail because it's too much and we have no mint yet
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(100),
             false);

    // Start the mint
    call_set_minting_badge_nf(&mut test_runner,
                              flexifaucet,
                              &alice,
                              &owner_badge_nfgid,
                              &minting_badge_nfgid,
                              true);

    // And now we can tap a hundred easy

    let faucet_vaults = test_runner.get_component_vaults(
        flexifaucet, token_resaddr);
    let tokens_in_faucet = faucet_vaults.iter()
        .map(|node| test_runner.inspect_non_fungible_vault(*node).unwrap().0)
        .sum();
    assert_eq!(dec!(40), tokens_in_faucet, "Treasury should be stocked");

    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 None,
                 dec!(100),
                 true);

    if let BalanceChange::NonFungible{ added, .. } = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(100, added.len(), "Alice should be 10 tokens up");
        } else {
            panic!("Change should be non-fungible");
        }

    let faucet_vaults = test_runner.get_component_vaults(
        flexifaucet, token_resaddr);
    let tokens_in_faucet = faucet_vaults.iter()
        .map(|node| test_runner.inspect_non_fungible_vault(*node).unwrap().0)
        .sum();
    assert_eq!(dec!(0), tokens_in_faucet, "Treasury should be empty");

    // We shouldn't be allowed to try to mint a non-whole number of
    // non-fungibles
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!("1.1"),
             false);

    
    // In a different test we checked that for a fungibles faucet we
    // could successfully change it mid-life from having a
    // non-fungible minting badge to having a fungible minting
    // badge. If it worked there then there is no reason it's not also
    // going to work for a non-fungibles faucet so we do not test it
    // again here.
}

    
/// Tests that we can successfully tap from a faucet that has a max
/// per-tap limit.
#[test]
fn test_tap_with_max_limit() {
    let mut test_runner = TestRunner::builder().build();
    let alice = User::new(&mut test_runner);
    assert_tap_with_max_limit(&mut test_runner, alice, None, None, None);
}

/// Tests that we can successfully tap from a faucet that has a max
/// per-tap limit and fungible taker badges.
#[test]
fn test_tap_with_max_limit_and_fungible_taker_badges() {
    let mut test_runner = TestRunner::builder().build();
    let alice = User::new(&mut test_runner);
    let taker_badges = test_runner.create_fungible_resource(
        dec!(1000), 0, alice.account);
    assert_tap_with_max_limit(&mut test_runner,
                              alice,
                              Some(taker_badges),
                              Some(dec!(3)),
                              Some(BadgeSpec::Fungible(taker_badges, dec!(3))));
}

/// Tests that we can successfully tap from a faucet that has a max
/// per-tap limit and non-fungible taker badges.
#[test]
fn test_tap_with_max_limit_and_non_fungible_taker_badges() {
    let mut test_runner = TestRunner::builder().build();
    let alice = User::new(&mut test_runner);
    let (taker_badges_resaddr, taker_badge_nfgid) =
        create_nf_resource_and_badge(&mut test_runner, &alice);
    assert_tap_with_max_limit(&mut test_runner,
                              alice,
                              Some(taker_badges_resaddr),
                              None,
                              Some(BadgeSpec::NonFungible(taker_badge_nfgid)));
}

/// Common implementation for testing simple max limit faucet. Tests
/// tapping from treasury, from minting, and from both combined.
fn assert_tap_with_max_limit(
    test_runner: &mut TestRunner,
    alice: User,
    taker_badges: Option<ResourceAddress>,
    taker_badge_amount: Option<Decimal>,
    badge_spec: Option<BadgeSpec>) {
    let package = test_runner.compile_and_publish(this_package!());

    let ((_, owner_badge_nfgid),
         (minting_badges_resaddr, minting_badge_nfgid)) =
        (create_nf_resource_and_badge(test_runner, &alice),
         create_nf_resource_and_badge(test_runner, &alice));

    let (token_resaddr, _) =
        call_new_non_fungible_resource(test_runner,
                                       package,
                                       &alice,
                                       &owner_badge_nfgid,
                                       minting_badges_resaddr,
                                       false,
                                       None,
                                       None);

    // This faucet has max 10 tokens per tap
    let (flexifaucet, _) =
        call_new(test_runner,
                 package,
                 &alice,
                 &owner_badge_nfgid,
                 token_resaddr,
                 Some(dec!(10)),
                 None,
                 taker_badges,
                 taker_badge_amount);

    mint_ruid_non_fungibles(test_runner,
                            &alice,
                            token_resaddr,
                            &minting_badge_nfgid,
                            flexifaucet,
                            10000);

    call_activate(test_runner,
                  flexifaucet,
                  &alice,
                  &owner_badge_nfgid,
                  true);
    
    call_fund_treasury_f(test_runner,
                         flexifaucet,
                         &alice,
                         &owner_badge_nfgid,
                         (token_resaddr, dec!(30)),
                         true);

    if badge_spec.is_some() {
        // Tapping without badge when badge is set should fail
        call_tap(test_runner,
                 flexifaucet,
                 &alice,
                 None,
                 dec!(1),
                 false);
    }

    // Should work
    let result =
        call_tap(test_runner,
                 flexifaucet,
                 &alice,
                 badge_spec.clone(),
                 dec!(1),
                 true);

    if let BalanceChange::NonFungible{ added, .. } = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(1, added.len(), "Alice should be 1 token up");
        } else {
            panic!("Change should be non-fungible");
        }

    // Should work again
    let result =
        call_tap(test_runner,
                 flexifaucet,
                 &alice,
                 badge_spec.clone(),
                 dec!(10),
                 true);

    if let BalanceChange::NonFungible{ added, .. } = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(10, added.len(), "Alice should be 10 coins up");
        } else {
            panic!("Change should be non-fungible");
        }

    // Should fail because it's beyond the max-per-tap limit of 10
    call_tap(test_runner,
             flexifaucet,
             &alice,
             badge_spec.clone(),
             dec!(11),
             false);

    // Let's clear out all but nine of the tokens in the faucet
    call_tap(test_runner,
             flexifaucet,
             &alice,
             badge_spec.clone(),
             dec!(10),
             true);

    // It's important that there's less then 10 left so we check
    let faucet_vaults = test_runner.get_component_vaults(
        flexifaucet, token_resaddr);
    let tokens_in_faucet = faucet_vaults.iter()
        .map(|node| test_runner.inspect_non_fungible_vault(*node).unwrap().0)
        .sum();
    assert_eq!(dec!(9), tokens_in_faucet, "Treasury must have 9 tokens");

    // And we give the faucet the ability to mint
    call_set_minting_badge_nf(test_runner,
                              flexifaucet,
                              &alice,
                              &owner_badge_nfgid,
                              &minting_badge_nfgid,
                              true);

    // A malfunctioning faucet might give us the 11 tokens we ask for
    // at this point (if it gets confused when it's taking both from
    // treasury and from mint) so we test this.
    call_tap(test_runner,
             flexifaucet,
             &alice,
             badge_spec.clone(),
             dec!(11),
             false);

    // Now lets empty out the treasury totally
    call_tap(test_runner,
             flexifaucet,
             &alice,
             badge_spec.clone(),
             dec!(10),
             true);

    // Again, it's important that the treasury is now empty
    let faucet_vaults = test_runner.get_component_vaults(
        flexifaucet, token_resaddr);
    let tokens_in_faucet = faucet_vaults.iter()
        .map(|node| test_runner.inspect_non_fungible_vault(*node).unwrap().0)
        .sum();
    assert_eq!(dec!(0), tokens_in_faucet, "Treasury must be empty");

    // And we check that a minting-only faucet also enforces the
    // max-per-tap limit
    call_tap(test_runner,
             flexifaucet,
             &alice,
             badge_spec.clone(),
             dec!(11),
             false);
}

/// Tests that when we set non-fungible taker badges, only holders of
/// such can tap.
#[test]
fn test_tap_with_non_fungible_taker_badges() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let ((_, owner_badge_nfgid),
         (taker_badges_resaddr, taker_badge_nfgid),
         (minting_badges_resaddr, minting_badge_nfgid),
         (_, unrelated_badge_nfgid)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    let (token_resaddr, _) =
        call_new_non_fungible_resource(&mut test_runner,
                                       package,
                                       &alice,
                                       &owner_badge_nfgid,
                                       minting_badges_resaddr,
                                       false,
                                       None,
                                       None);

    let (flexifaucet, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 &owner_badge_nfgid,
                 token_resaddr,
                 None, None,
                 Some(taker_badges_resaddr),
                 None);

    mint_ruid_non_fungibles(&mut test_runner,
                            &alice,
                            token_resaddr,
                            &minting_badge_nfgid,
                            flexifaucet,
                            100);

    call_activate(&mut test_runner,
                  flexifaucet,
                  &alice,
                  &owner_badge_nfgid,
                  true);

    call_fund_treasury_f(&mut test_runner,
                         flexifaucet,
                         &alice,
                         &owner_badge_nfgid,
                         (token_resaddr, dec!(30)),
                         true);

    // Presenting no badge shouldn't work
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(1),
             false);

    // Presenting an unrelated badge shouldn't work
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             Some(BadgeSpec::NonFungible(unrelated_badge_nfgid)),
             dec!(1),
             false);

    // But using an actual user badge should work
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 Some(BadgeSpec::NonFungible(taker_badge_nfgid)),
                 dec!(1),
                 true);

    if let BalanceChange::NonFungible{ added, .. } = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(1, added.len(), "Alice should be 1 token up");
        } else {
            panic!("Change should be non-fungible");
        }
}


/// Tests that when we set fungible taker badges, only holders of
/// sufficient such can tap.
#[test]
fn test_tap_with_fungible_taker_badges() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let ((_, owner_badge_nfgid),
         (_, unrelated_badge_nfgid)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    let token_resaddr = test_runner.create_fungible_resource(
        dec!(1000), 0, alice.account);

    let taker_badges_resaddr = test_runner.create_fungible_resource(
        dec!(1000), 0, alice.account);
    let unrelated_fungible_badge_resaddr = test_runner.create_fungible_resource(
        dec!(1000), 0, alice.account);

    // This faucet requires you to present at least 3 badge tokens to
    // tap.
    let (flexifaucet, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 &owner_badge_nfgid,
                 token_resaddr,
                 None, None,
                 Some(taker_badges_resaddr),
                 Some(dec!(3)));

    call_activate(&mut test_runner,
                  flexifaucet,
                  &alice,
                  &owner_badge_nfgid,
                  true);

    call_fund_treasury_f(&mut test_runner,
                         flexifaucet,
                         &alice,
                         &owner_badge_nfgid,
                         (token_resaddr, dec!(30)),
                         true);

    // Presenting no badge shouldn't work
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(1),
             false);

    // Presenting an unrelated fungible badge shouldn't work
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             Some(BadgeSpec::Fungible(unrelated_fungible_badge_resaddr, dec!(3))),
             dec!(1),
             false);

    // Presenting an unrelated non-fungible badge shouldn't work
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             Some(BadgeSpec::NonFungible(unrelated_badge_nfgid)),
             dec!(1),
             false);

    // Presenting too few of the correct badge shouldn't work
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             Some(BadgeSpec::Fungible(taker_badges_resaddr, dec!("2.9"))),
             dec!(1),
             false);

    // But using the correct number of badges should work
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 Some(BadgeSpec::Fungible(taker_badges_resaddr, dec!(3))),
                 dec!(1),
                 true);

    if let BalanceChange::Fungible(change) = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(dec!(1), *change, "Alice should be 1 token up");
        } else {
            panic!("Change should be fungible");
        }

    // using too many badges should also work
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 Some(BadgeSpec::Fungible(taker_badges_resaddr, dec!(30))),
                 dec!(1),
                 true);

    if let BalanceChange::Fungible(change) = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(dec!(1), *change, "Alice should be 1 token up");
        } else {
            panic!("Change should be fungible");
        }
}

/// Tests that when we set cooldown without taker badges we are
/// prevented from tapping while waiting for cooldown.
#[test]
fn test_tap_with_cooldown() {
    let mut test_runner = TestRunner::builder()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .without_trace()
        .build();
    let alice = User::new(&mut test_runner);

    assert_tap_with_cooldown(&mut test_runner,
                             alice,
                             None, None, None);
}

/// Tests that when we set cooldown with fungible taker badges we are
/// prevented from tapping while waiting for cooldown.
#[test]
fn test_tap_with_fungible_taker_badges_and_cooldown() {
    let mut test_runner = TestRunner::builder()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .without_trace()
        .build();
    let alice = User::new(&mut test_runner);
    let taker_badges_resaddr = test_runner.create_fungible_resource(
        dec!(1000), 0, alice.account);

    assert_tap_with_cooldown(&mut test_runner,
                             alice,
                             Some(taker_badges_resaddr),
                             Some(dec!(3)),
                             Some(BadgeSpec::Fungible(taker_badges_resaddr, dec!(3))));
}

/// Tests that when we set cooldown with non-fungible taker badges we
/// are prevented from tapping while waiting for cooldown.
#[test]
fn test_tap_with_non_fungible_taker_badges_and_cooldown() {
    let mut test_runner = TestRunner::builder()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .without_trace()
        .build();

    let alice = User::new(&mut test_runner);
    let (taker_badges_resaddr, taker_badge_nfgid) =
        create_nf_resource_and_badge(&mut test_runner, &alice);

    assert_tap_with_cooldown(&mut test_runner,
                             alice,
                             Some(taker_badges_resaddr),
                             None,
                             Some(BadgeSpec::NonFungible(taker_badge_nfgid)));
}

/// Implements test of tapping with cooldown without max limit.
fn assert_tap_with_cooldown(
    test_runner: &mut TestRunner,
    alice: User,
    taker_badges: Option<ResourceAddress>,
    taker_badge_amount: Option<Decimal>,
    badge_spec: Option<BadgeSpec>)
{
    let package = test_runner.compile_and_publish(this_package!());
    let (_, owner_badge_nfgid) =
        create_nf_resource_and_badge(test_runner, &alice);
    let token_resaddr = test_runner.create_fungible_resource(
        dec!(1000), 0, alice.account);

    // This faucet imposes a 600 second (10 minute) cooldown between
    // taps
    let (flexifaucet, _) =
        call_new(test_runner,
                 package,
                 &alice,
                 &owner_badge_nfgid,
                 token_resaddr,
                 None, Some(600),
                 taker_badges,
                 taker_badge_amount);

    call_activate(test_runner,
                  flexifaucet,
                  &alice,
                  &owner_badge_nfgid,
                  true);

    call_fund_treasury_f(test_runner,
                         flexifaucet,
                         &alice,
                         &owner_badge_nfgid,
                         (token_resaddr, dec!(30)),
                         true);

    if badge_spec.is_some() {
        // Shouldn't be able to take without a badge when badges have
        // been set
        call_tap(test_runner,
                 flexifaucet,
                 &alice,
                 None,
                 dec!(1),
                 false);
    }

    // First tap should work
    call_tap(test_runner,
             flexifaucet,
             &alice,
             badge_spec.clone(),
             dec!(1),
             true);

    // But then we're denied
    call_tap(test_runner,
             flexifaucet,
             &alice,
             badge_spec.clone(),
             dec!(1),
             false);

    // We advance time not quite to the next period
    set_test_runner_clock(test_runner, 599);

    // And get denied again
    call_tap(test_runner,
             flexifaucet,
             &alice,
             badge_spec.clone(),
             dec!(1),
             false);

    // Time advances to the next period
    set_test_runner_clock(test_runner, 600);

    // And we can tap again
    call_tap(test_runner,
             flexifaucet,
             &alice,
             badge_spec.clone(),
             dec!(1),
             true);
}

/// Tests that when we set taker badges, cooldown and a max limit,
/// taps are limited both in total tap per period.
#[test]
fn test_tap_with_non_fungible_taker_badges_and_limits_and_cooldown() {
    let mut test_runner = TestRunner::builder()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .without_trace()
        .build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);

    let ((_, owner_badge_nfgid),
         (taker_badges_resaddr, taker_badge1_nfgid),
         (minting_badges_resaddr, minting_badge_nfgid)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    let badge1_spec = Some(BadgeSpec::NonFungible(taker_badge1_nfgid));
    let badge2_spec = Some(BadgeSpec::NonFungible(NonFungibleGlobalId::new(
        taker_badges_resaddr, 2.into())));
    
    let (token_resaddr, _) =
        call_new_non_fungible_resource(&mut test_runner,
                                       package,
                                       &alice,
                                       &owner_badge_nfgid,
                                       minting_badges_resaddr,
                                       false,
                                       None,
                                       None);

    // This faucet gives max 3 tokens per 600 seconds per user.
    let (flexifaucet, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 &owner_badge_nfgid,
                 token_resaddr,
                 Some(dec!(3)),
                 Some(600),
                 Some(taker_badges_resaddr),
                 None);

    call_set_minting_badge_nf(&mut test_runner,
                              flexifaucet,
                              &alice,
                              &owner_badge_nfgid,
                              &minting_badge_nfgid,
                              true);

    call_activate(&mut test_runner,
                  flexifaucet,
                  &alice,
                  &owner_badge_nfgid,
                  true);

    // Presenting no badge shouldn't work
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             None,
             dec!(1),
             false);

    // badge1 does some tapping
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             badge1_spec.clone(),
             dec!(1),
             true);
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             badge1_spec.clone(),
             dec!(1),
             true);

    // Advance time a bit just because
    set_test_runner_clock(&mut test_runner, 240);

    // And this fills our quota for this period
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 badge1_spec.clone(),
                 dec!(1),
                 true);

    if let BalanceChange::NonFungible{ added, .. } = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(1, added.len(), "Alice should be 1 token up");
        } else {
            panic!("Change should be non-fungible");
        }

    // Another should not be allowed
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             badge1_spec.clone(),
             dec!(1),
             false);

    // But badge2 should still be allowed because it's not been used yet
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 badge2_spec.clone(),
                 dec!(3),
                 true);

    if let BalanceChange::NonFungible{ added, .. } = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(3, added.len(), "Alice should be 3 tokens up");
        } else {
            panic!("Change should be non-fungible");
        }

    // Again, another should not be allowed
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             badge2_spec.clone(),
             dec!(1),
             false);

    // Advance time to just before the end of badge1's period
    set_test_runner_clock(&mut test_runner, 599);

    // And this should still fail
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             badge1_spec.clone(),
             dec!(1),
             false);

    // Enter next period for badge1
    set_test_runner_clock(&mut test_runner, 600);

    // badge2 still can't tap
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             badge2_spec.clone(),
             dec!(1),
             false);

    // But badge1 can tap again
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 badge1_spec.clone(),
                 dec!(3),
                 true);

    if let BalanceChange::NonFungible{ added, .. } = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(3, added.len(), "Alice should be 3 tokens up");
        } else {
            panic!("Change should be non-fungible");
        }

    // And that's all 3 gone for badge1 so this now fails
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             badge1_spec.clone(),
             dec!(1),
             false);

    // Enter next period for badge2
    set_test_runner_clock(&mut test_runner, 840);

    // Can't tap 4 for badge1
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             badge1_spec.clone(),
             dec!(4),
             false);

    // But badge2 can now tap 3
    let result =
        call_tap(&mut test_runner,
                 flexifaucet,
                 &alice,
                 badge2_spec.clone(),
                 dec!(3),
                 true);

    if let BalanceChange::NonFungible{ added, .. } = result.balance_changes()
        .get(&GlobalAddress::from(alice.account)).unwrap()
        .get(&token_resaddr).unwrap() {
            assert_eq!(3, added.len(), "Alice should be 3 tokens up");
        } else {
            panic!("Change should be non-fungible");
        }

    // Can't tap another for badge2
    call_tap(&mut test_runner,
             flexifaucet,
             &alice,
             badge2_spec.clone(),
             dec!(1),
             false);
}

/// Tests that we can change the `funder` role on a faucet.
#[test]
fn test_change_fund_treasury_access_control() {
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish(this_package!());
    let alice = User::new(&mut test_runner);
    let execution_proofs = vec![NonFungibleGlobalId::from_public_key(&alice.pubkey)];

    let ((owner_badges_resaddr, owner_badge_nfgid),
         (_, unrelated_badge_nfgid)) =
        (create_nf_resource_and_badge(&mut test_runner, &alice),
         create_nf_resource_and_badge(&mut test_runner, &alice));

    let token_resaddr =
        test_runner.create_fungible_resource(dec!(1000), 18, alice.account);


    let (flexifaucet, _) =
        call_new(&mut test_runner,
                 package,
                 &alice,
                 &owner_badge_nfgid,
                 token_resaddr,
                 None, None, None, None);

    // Unrelated badges shouldn't be able to fund by default
    call_fund_treasury_f(&mut test_runner,
                         flexifaucet,
                         &alice,
                         &unrelated_badge_nfgid,
                         (token_resaddr, dec!(30)),
                         false);

    // Change the funder role to allow all
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            alice.account,
            owner_badges_resaddr,
            &BTreeSet::from([owner_badge_nfgid.local_id().clone()]))
        .update_role(flexifaucet,
                     ObjectModuleId::Main,
                     RoleKey::new("funder"),
                     rule!(allow_all))
        .build();
    test_runner.execute_manifest_ignoring_fee(manifest, execution_proofs.clone())
        .expect_commit(true);
    
    // Now anyone can fund us
    call_fund_treasury_f(&mut test_runner,
                         flexifaucet,
                         &alice,
                         &unrelated_badge_nfgid,
                         (token_resaddr, dec!(30)),
                         true);
}
