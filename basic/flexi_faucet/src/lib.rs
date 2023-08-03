//! A faucet blueprint with very flexible configuration options.
//!
//! A faucet is a service that gives away some amount of tokens to
//! users, for free. It can typically be provided for marketing
//! purposes, for fun and games, or even perhaps out of sheer
//! generosity.
//!
//! The FlexiFaucet is able to hand out fungible or non-fungible
//! tokens. It can be configured to be open to the public, or it can
//! be limited to only holders of a specific badge resource. Those
//! badges can be fungible or non-fungible. Users can be limited in
//! how much they are able to tap, and how often a tap can happen. If
//! non-fungible user badges are employed then these limitations are
//! applied per-badge so that users have individually tracked limits.
//!
//! # Public interface
//!
//! These are the functions and methods this blueprint exposes to be
//! called from a transaction manifest.
//!
//! ## Creation functions
//!
//! - [new] Creates a new `FlexiFaucet` instance.
//!
//! - [new_non_fungible_resource] Creates a non-fungible resource
//! suitable to use as a non-fungible token that will be minted by
//! this faucet.
//!
//! ## Activation
//!
//! - [activate] Turns on the faucet, allowing users to start calling
//! [tap].
//!
//! - [deactivate] Turns off the faucet, causing subsequent calls to
//! [tap] to panic.
//!
//! - [active] Determines whether the faucet is currently active.
//!
//! ## Minting
//!
//! - [set_minting_badge] Establishes or changes the minting badge to
//! use. So long as a minting badge is set the faucet will attempt to
//! mint tokens to cover any shortfall in the treasury.
//!
//! - [minting_badge_as_fungible] Retrieves information on the current
//! minting badge as if it were a fungible, which is to say it returns
//! the resource address and how many tokens are held.
//!
//! - [minting_badge_as_non_fungible] Retrieves information about the
//! current non-fungible minting badge.
//!
//! ## Treasury
//!
//! - [fund_treasury] Puts tokens into the faucet's treasury, which
//! will then be used to fund [tap] calls from users.
//!
//! - [defund_treasury_fungible] Takes tokens out of the treasury as
//! if they were fungible, which is to say it pulls them out by
//! amount.
//!
//! - [defund_treasury_non_fungible] Takens non-fungible tokens out of
//! the treasury.
//!
//! - [empty_treasury] Completely empties out the treasury.
//!
//! - [treasury_as_fungible] Retrieves information about the current
//! treasury as if it were fungible, which is to say it returns the
//! amount.
//!
//! - [treasury_as_non_fungible] Retrieves information about the
//! current non-fungible treasury.
//!
//! ## The main job
//!
//! - [tap] This method is called to access its main function, namely
//! to distribute tokens to our users.
//! 
//! # Roles and access control
//!
//! The blueprint sets a fixed owner role, and assigns the owner as
//! the only one with access to the two roles we define. If you want
//! `admin` or `funder` roles to be accessible to others, you can
//! change those roles to your suiting. See [Treasury](#treasury-1)
//! for an example of how to do this.
//!
//! We define the following roles:
//!
//! - `admin` Can call the administrative methods of the
//! blueprint. The owner can change this role.
//!
//! - `funder` Can call the [fund_treasury] method. An admin can
//! change this role.
//!
//! Beyond this, anyone can call the creation functions and the query
//! methods.
//!
//! The [tap] method is open for everyone to call but depending on how
//! you have configured the faucet, there is internal access checking
//! that will stop unwanted users.
//!
//! # Typical call sequences
//!
//! ## Creating a faucet
//!
//! When initially setting up a faucet you will want to first have
//! created the token resource that the faucet is going to hand out,
//! you will want to have created any taker badge resource you intend
//! to use, and if you want the faucet to mint you also want to have
//! the minting badge resource ready. You can inspect the test suite
//! to see how this may be done.
//!
//! 1. Call [new_non_fungible_resource] if you want the faucet to be
//! minting non-fungible tokens to hand out.
//!
//! 2. Call [new] to create the new faucet.
//!
//! 3. Call [set_minting_badge] if you want the faucet to be able to
//! mint tokens.
//!
//! 4. Call [fund_treasury] if you want the faucet to hand out tokens
//! that have already been created.
//!
//! 5. Call [activate] to open up for business.
//!
//! ## Taking a tap from the faucet
//!
//! 1. Call [tap].
//!
//! 2. That's it. There's no 2. Goto 1!
//!
//! # Faucet tap limits
//!
//! You can limit how much and how frequently the faucet can be used.
//!
//! The token limit per tap restricts how many tokens can be returned
//! from a single call to the [tap] method. Set this to e.g. 5 tokens
//! and anyone trying to call it to tap 6 will face an error message.
//!
//! The cooldown period restricts how frequently the [tap] method can
//! be called. Set this to e.g. 600 seconds and the faucet cannot be
//! tapped more than once every ten minutes. (Take note that the
//! ledger cannot tell time with higher precision than one full
//! minute, so if you set cooldown periods less than about two minutes
//! you may start seeing funny rounding effects.)
//!
//! Normally these two parameters work globally for all users, such
//! that if Alice has exceeded the limits with her call then Bob also
//! has to deal with the consequences of that.
//!
//! If you restrict tapping to owners of non-fungible taker badges
//! however, each user (actually each badge) tracks its limits
//! individually so that even if Alice has reached her limit, Bob can
//! still get *his* tap in. In this case, the token limit per tap is
//! modified to instead be a token limit per cooldown period such that
//! if Bob taps 2 tokens initially against a max limit of 5, then he
//! can still tap another 3 within the same cooldown period.
//!
//! # Taker badges
//!
//! User badges, which we also call taker badges, can be of any
//! resource: They can be a specifically badge-like resource
//! (i.e. divisibility 0 and typically minted in very low supply), or
//! they can be anything else. You can use XRD as your badge resource
//! if you like, or VKC, or RaDragon NFTs, or whatever. So you can
//! make a faucet for holders of your NFT collection as a service to
//! your community, for example; or you can make a faucet for "anyone
//! who holds at least 1000 VKC," if you're looking to reward
//! high-stake holders in your community. If you want to target
//! whales, you could make a faucet for "anyone who holds at least 1
//! million XRD" -- and if you're catering even to smaller fry you
//! could allow "anyone who holds at least 0.01 BTC."
//!
//! If you use fungible badges (or no badges at all, meaning the
//! faucet is open to everyone) then the tap limit and cooldowns you
//! set apply globally. That is, if Alice taps at time t=100 then
//! *everyone* has to wait for the cooldown to run out at e.g. time
//! t=200 before anyone can tap again.
//!
//! If you use non-fungible badges however then max tap limits and
//! cooldowns are tracked per badge. In this case, if Alice taps at
//! time t=100 then *she* has to wait until t=200 to tap again; but
//! Bob could tap at time t=101 unaffected by Alice's activity (he
//! would then have to wait until t=201 to tap again).
//!
//! Note that as we do not use self-produced NFTs as badges, we do not
//! store cooldown period info inside non-fungible data. Instead we
//! can use *any* NFT series as badges and we store cooldown periods
//! inside a `KeyValueStore` in component state. This enables us to
//! market this faucet towards e.g. all the various NFT collections
//! such that everyone can use their favourite Undying Viking or
//! whatever to tap magic internet money from the faucet.
//!
//! # Token sources
//!
//! Tokens for tapping must come from one of two sources (or both):
//! Either the treasury, or from minting.
//!
//! The faucet always tries to take from treasury first, and only when
//! treasury runs out does it resort to minting new tokens.
//!
//! ## Treasury
//!
//! You can fund the treasury by calling [fund_treasury] and passing
//! in a pile of tokens.
//!
//! Note that in the default configuration that method can only be
//! called by the owner. If you want the general public to be able to
//! fund your faucet, you can change the access control of that method
//! by running a transaction manifest built along these lines:
//!
//! ```text
//! ManifestBuilder::new()
//!     ...
//!     .create_proof_from_account_of_non_fungibles(
//!         owner_account,
//!         owner_badge.resource_address(),
//!         &BTreeSet::from([owner_badge.local_id().clone()]))
//!     .update_role(flexifaucet,
//!                  ObjectModuleId::Main,
//!                  RoleKey::new("funder"),
//!                  rule!(allow_all))
//!     ...
//! ```
//!
//! You can see the above in action in
//! `test_change_fund_treasury_access_control()` in the test suite.
//! (Of course, you don't have to set `allow_all` in there -- you can
//! set any rule you want.)
//!
//! In the case of drawing from treasury, if the treasury runs out
//! (and you haven't configured minting) then attempts to [tap] will
//! panic.
//!
//! ## Mint
//!
//! If you give the faucet a minting badge for its token resource then
//! it will be able to create new tokens to cover demand.
//!
//! When it comes to fungibles, we can mint anything you can provide a
//! minting badge for. For non-fungibles however we can only mint
//! resources that have been created with our own
//! [new_non_fungible_resource] function. The faucet can support other
//! types of non-fungibles than this but they will have to be provided
//! via treasury only.
//!
//! Minting can panic if the minting badge is wrong or, notably, in
//! the event that you run into cost unit limits while minting
//! non-fungibles. The latter can happen when trying to mint a large
//! number of non-fungible tokens in the same transaction.
//!
//! # Making your non-fungible resource
//!
//! For this faucet to be able to mint non-fungible tokens, they need
//! to be based on a non-fungible struct we know at compile-time. For
//! this reason the faucet can only mint non-fungibles that are based
//! on its own `FaucetNfData` non-fungible data structure.
//!
//! You can use the [new_non_fungible_resource] function to make a
//! resource like that.
//!
//! That function has been designed such that it gives you complete
//! flexibility in how to configure the new resource after it has been
//! made. The intention is that you first create the resource, then
//! you configure it with all the access rules, metadata, etc., that
//! you want, then you pass the new resource into the faucet.
//!
//! In `assert_metadata_and_access_update()` in the test suite there
//! is a transaction manifest which demonstrates how to change these
//! access rules.
//!
//! Also note that the [new_non_fungible_resource] function has a few
//! convenience parameters that I hope will make it feasible for
//! *most* people to receive an immediately usable resource out of the
//! function without having to do a lot of fiddly follow-up
//! configuration of it.
//!
//! # Making your fungible resource
//!
//! You can use absolutely any fungible resource, and the faucet will
//! be able to mint it so long as you can deliver it a minting badge.
//!
//! In the test suite there is an example of a transaction manifest
//! for building a fungible resource with custom roles in
//! `create_mintable_fungible_resource()` -- you may find this useful
//! if you're creating such a resource for use in this faucet.
//!
//! Also, there is a transaction manifest for *changing* the minting
//! role of a resource in `update_minting_role()`. This is maybe less
//! routinely useful but nevertheless it's there if you need it.
//!
//! # About minting badge vault handling
//!
//! In this component we allow admins to replace the minting badge in
//! use. The idea is that the resources we operate on can have their
//! roles changed and their badges replaced (perhaps in response to a
//! security threat) and the faucet should support such an
//! eventuality.
//!
//! A complication that arises is that when you replace the minting
//! badge with one of a wholly different resource, you cannot just
//! re-use the vault of the previous badge because the vault is locked
//! to the resource address it originally had. Instead you need to
//! make an entirely new vault to put the new badge in, and you need
//! to discard the old vault.
//!
//! Scrypto however doesn't support the discarding of vaults: you
//! always need to hang on to them. In order to accommodate this we
//! put our vaults in a `KeyValueStore` that is keyed by an
//! ever-increasing index and in which the highest index entry is the
//! vault that is in use. All other vaults are old discarded ones.
//!
//! You can see the handling of this by inspecting our use of the
//! `minting_badge` field of our component state.
//!
//! (This would have been a lot easier to deal with if Vault had had a
//! `drop_empty_vault` method -- then I probably could have let
//! `minting_badge` be an actual Vault and just use
//! `std::mem::replace` to switch it out. Here's hoping for Scrypto
//! v0.12!)
//!
//! # Test suite
//!
//! In the `tests` directory you will find a comprehensive test suite
//! for the faucet that runs through all its functionality. This can
//! be instructive both in demonstrating various ways in which the
//! faucet can be used, in showing how to build the various
//! transaction manifests needed, and of course in how to write tests
//! with `TestRunner`.
//!
//! ## Transaction manifests
//!
//! Transaction manifests for all publicly exposed methods are built
//! as part of the test suite. Look for the following methods in the
//! `tests/lib.rs` file:
//!
//! | Scrypto function                | Transaction manifest in |
//! |---------------------------------|-------------------------|
//! | [new]                           | `call_new()`
//! | [new_non_fungible_resource]     | `call_new_non_fungible_resource()`
//! | [activate]                      | `call_activate()`
//! | [deactivate]                    | `call_deactivate()`
//! | [active]                        | `call_active()`
//! | [set_minting_badge]             | `call_set_minting_badge_f()`, `call_set_minting_badge_f()`
//! | [minting_badge_as_fungible]     | `call_minting_badge_as_fungible()`
//! | [minting_badge_as_non_fungible] | `call_minting_badge_as_non_fungible()`
//! | [fund_treasury]                 | `call_fund_treasury_f()`, `call_fund_treasury_nf()`
//! | [defund_treasury_fungible]      | `call_defund_treasury_fungible()`
//! | [defund_treasury_non_fungible]  | `call_defund_treasury_non_fungible()`
//! | [empty_treasury]                | `call_empty_treasury()`
//! | [treasury_as_fungible]          | `call_treasury_as_fungible()`
//! | [treasury_as_non_fungible]      | `call_treasury_as_non_fungible()`
//! | [tap]                           | `call_tap()`
//!
//! Note that the transaction manifests built in the test suite are
//! all run without charging transaction fees, and so none of them
//! bother to lock fees. If you intend to use them towards an actual
//! ledger you will want to add a `lock_fee` instruction at the top.
//!
//! ## Clock manipulation
//!
//! When setting the system clock in the test runner, note my use of
//! the global variable `ROUND_COUNTER` to make sure I always increase
//! the consensus round when changing the clock. A better way to do
//! this (nobody likes global variables) may have been to pass the
//! round counter around together with the test runner, but that is
//! a problem to solve for a future evolution of my test harness.
//!
//! # About this blueprint
//!
//! This project has been developed on Ubuntu Linux and while the
//! author expects everything in here to work on other platforms, if
//! you're having weird problems with it maybe this is the reason.
//!
//! This project has been developed for Scrypto v0.11.0 and was
//! submitted to the Scrypto developer incentive program which at time
//! of writing is available
//! [here.](https://www.radixdlt.com/blog/scrypto-developer-program-incentives)
//!
//! The author can be reached at `scryptonight@proton.me`
//!
//!
//! [new]: crate::flexi_faucet::FlexiFaucet::new
//! [new_non_fungible_resource]: crate::flexi_faucet::FlexiFaucet::new_non_fungible_resource
//! [active]: crate::flexi_faucet::FlexiFaucet::active 
//! [minting_badge_as_fungible]: crate::flexi_faucet::FlexiFaucet::minting_badge_as_fungible 
//! [minting_badge_as_non_fungible]: crate::flexi_faucet::FlexiFaucet::minting_badge_as_non_fungible 
//! [treasury_as_fungible]: crate::flexi_faucet::FlexiFaucet::treasury_as_fungible 
//! [treasury_as_non_fungible]: crate::flexi_faucet::FlexiFaucet::treasury_as_non_fungible 
//! [activate]: crate::flexi_faucet::FlexiFaucet::activate 
//! [deactivate]: crate::flexi_faucet::FlexiFaucet::deactivate 
//! [set_minting_badge]: crate::flexi_faucet::FlexiFaucet::set_minting_badge 
//! [defund_treasury_fungible]: crate::flexi_faucet::FlexiFaucet::defund_treasury_fungible 
//! [defund_treasury_non_fungible]: crate::flexi_faucet::FlexiFaucet::defund_treasury_non_fungible 
//! [empty_treasury]: crate::flexi_faucet::FlexiFaucet::empty_treasury 
//! [fund_treasury]: crate::flexi_faucet::FlexiFaucet::fund_treasury 
//! [tap]: crate::flexi_faucet::FlexiFaucet::tap 
use scrypto::prelude::*;

#[derive(ScryptoSbor)]
struct CooldownPeriod {
    /// Unix-era seconds.
    period_start: Option<i64>,
    /// How much has been tapped this period.
    tap_total: Decimal,
}

#[derive(ScryptoSbor, ManifestSbor, NonFungibleData)]
pub struct FaucetNfData {
    pub faucet: ComponentAddress,
    #[mutable]
    pub attributes: HashMap<String, String>,
}

#[blueprint]
mod flexi_faucet {

    enable_method_auth! {
        roles {
            admin => updatable_by: [OWNER];
            funder => updatable_by: [admin];
        },
        methods {
            active => PUBLIC;
            minting_badge_as_fungible => PUBLIC;
            minting_badge_as_non_fungible => PUBLIC;
            treasury_as_fungible => PUBLIC;
            treasury_as_non_fungible => PUBLIC;
            activate => restrict_to: [admin];
            deactivate => restrict_to: [admin];
            set_minting_badge => restrict_to: [admin];
            defund_treasury_fungible => restrict_to: [admin];
            defund_treasury_non_fungible => restrict_to: [admin];
            empty_treasury => restrict_to: [admin];

            // If you want others than admins to be able to fund,
            // change the funder role.
            fund_treasury => restrict_to: [funder];

            // The below method employs intent-based access control:
            // It is restricted to our taker_badges if those have been
            // set to Some(...), or else if they have been set to None
            // then the tap is open to the public.
            tap => PUBLIC;
        }
    }

    struct FlexiFaucet {
        active: bool,
        token: ResourceAddress,
        max_per_tap: Option<Decimal>,
        tap_cooldown_secs: Option<u64>,
        last_tap: Option<i64>,
        cooldown_periods: KeyValueStore<NonFungibleGlobalId, CooldownPeriod>,
        taker_badges: Option<ResourceAddress>,
        minimum_fungible_badge_tokens : Option<Decimal>,
        minting_badge: KeyValueStore<u128, Option<Vault>>,
        minting_badge_index: u128,
        treasury: Option<Vault>,
    }

    impl FlexiFaucet {
        /// Creates a new faucet and returns it.
        ///
        /// The new component will be owned by `owner` and will be
        /// able to emit `token` type tokens. Its treasury and/or
        /// minting will be for that resource.
        ///
        /// You may optionally specify a `max_per_tap` that limits how
        /// many tokens can be extracted by each call of the `tap`
        /// method.
        ///
        /// You may also optionally specify `tap_cooldown_secs` which
        /// enforces a delay between subsequent taps.
        ///
        /// The faucet will be available to everyone unless you
        /// specify `taker_badges`. With those specified only someone
        /// who can prove such a badge may use the `tap` method.
        ///
        /// If you specify fungible `taker_badges` then you must also
        /// specify how many tokens of that resource are necessary to
        /// qualify for using the faucet. (If your `taker_badges` are
        /// non-fungible then the amount needed is always exactly 1.)
        ///
        /// If you specify all three of `max_per_tap`,
        /// `tap_cooldown_secs` and `taker_badges` then the maximum
        /// per tap is treated as a maximum total tap within a single
        /// cooldown period. For example, if you have 5 max and
        /// cooldown 10, then you could tap 1 token at time 0, 2 more
        /// tokens at time 7, a final 3 tokens at time 9; and then
        /// you're all tapped out until next period starts (in this
        /// case, at time 10).
        ///
        /// If you provide a `minting_badge` then the faucet will mint
        /// tokens if necessary to service users.
        ///
        /// This function panics if it detects a problem with the
        /// input arguments.
        pub fn new(owner: NonFungibleGlobalId,
                   token: ResourceAddress,
                   max_per_tap: Option<Decimal>,
                   tap_cooldown_secs: Option<u64>,
                   taker_badges: Option<ResourceAddress>,
                   minimum_fungible_badge_tokens: Option<Decimal>,
                   minting_badge: Option<Bucket>)
                   -> Global<FlexiFaucet>
        {
            if let Some(taker_badges) = taker_badges {
                assert!(!taker_badges.is_fungible()
                        || minimum_fungible_badge_tokens.is_some(),
                        "fungible taker badges must have a minimum amount set");
            } else {
                assert!(minimum_fungible_badge_tokens.is_none(),
                        "don't set minimum badge tokens without badges");
            }
            
            let minting_badge_store = KeyValueStore::new();
            if let Some(bucket) = minting_badge {
                minting_badge_store.insert(0,
                                           Some(Vault::with_bucket(bucket)));
            } else {
                minting_badge_store.insert(0, None);
            }
            Self {
                active: false,
                token,
                max_per_tap,
                tap_cooldown_secs,
                last_tap: None,
                cooldown_periods: KeyValueStore::new(),
                taker_badges,
                minimum_fungible_badge_tokens,
                minting_badge: minting_badge_store,
                minting_badge_index: 0,
                treasury: None,
            }
            .instantiate()
                .prepare_to_globalize(
                    OwnerRole::Fixed(rule!(require(owner.clone()))))
                .roles(roles!(
                    admin => rule!(require(owner.clone()));
                    funder => rule!(require(owner));
                ))
                .globalize()
        }

        /// If you want this faucet to be able to mint non-fungible
        /// tokens to provide to your users then the token resource
        /// used needs to be created using this function.
        ///
        /// This function creates a new non-fungible resource. By
        /// default we create one that is maximally flexible so that
        /// you, the owner of the resource, can afterwards use normal
        /// ledger calls (e.g. `update_role`, `set_metadata`) to
        /// configure it to your liking. The main unique feature the
        /// resource has from being created by us is that it uses our
        /// `FaucetNfData` non-fungible data structure, and *that* you
        /// cannot change.
        ///
        /// Access to configure and use the new resource is mostly
        /// (see below) governed by the `owner_rule` you specify. As
        /// this is an `AccessRule` it can be as simple or as complex
        /// as you need it to be. When building your transaction
        /// manifest it can look something like this:
        ///
        /// ```text
        /// manifest_args!(rule!(require(resource_address)), ...)
        /// ```
        ///
        /// Where `resource_address` is the badge resource you want to
        /// use for ownership.
        ///
        /// The resource created is a RUID-based non-fungible. New
        /// tokens created on this resource will receive arbitrarily
        /// selected non-fungible IDs, but be aware that the user
        /// composing the transaction is effectively in some control
        /// of what that ID will be. These IDs are *not* strongly
        /// random. Don't use them as a random-source for a lottery.
        ///
        /// In order for the default resource we create to be
        /// maximally configurable by you, the owner, it starts out
        /// being freezable and recallable. Many owners may find this
        /// to be undesirable (because holders tend to distrust such
        /// tokens), and in order to make life easier for you there is
        /// a `recallable` argument to the function that you can set
        /// to `false` to cause the resource to *not* be freezable or
        /// recallable and for this not to be changeable.
        ///
        /// Likewise, the default resource starts without any metadata
        /// but many owners will want to set the `name` and
        /// `description` of their NFT resource, and to make life a
        /// little bit easier in this regard you can optionally supply
        /// a `name` and/or `description` as arguments to the
        /// function.
        ///
        /// Notice that our `FaucetNfData` struct has a mutable
        /// `attributes` map that you can use to store per-NFT data.
        /// You would need to do this on your own after each token has
        /// been minted, the faucet doesn't do it for you.
        ///
        /// # Our default role rules
        ///
        /// - The owner can change the owner role.
        /// - Everyone can deposit or withdraw the token.
        /// - `minting_badge` (only) can mint the token.
        /// - The owner can burn, recall or freeze the token as well as
        /// update its metadata.
        /// - The owner can change all these rules.
        /// - **Except:** if `recallable` was `false` then the owner
        /// *cannot* recall or freeze and *cannot* change the recall
        /// or freeze rules.
        ///
        /// # Our default metadata
        ///
        /// - If you provide a `name` then this will be the `name`
        /// metadata field and it will be locked.
        /// - If you provide a `description` then this will be the
        /// `descripton` metadata field and it will be locked.
        /// - Any of those you do not provide, as well as all other
        /// metadata fields, you can set later yourself using your
        /// owner role.
        pub fn new_non_fungible_resource(owner_rule: AccessRule,
                                         minting_badge: ResourceAddress,
                                         recallable: bool,
                                         name: Option<String>,
                                         description: Option<String>)
                                         -> ResourceAddress
        {
            let recaller_rule = if recallable { owner_rule.clone() }
                                         else { rule!(deny_all) };

            let mut builder =
                ResourceBuilder::new_ruid_non_fungible::<FaucetNfData>(
                    OwnerRole::Updatable(owner_rule.clone()))
                .mint_roles(mint_roles!(
                    minter => rule!(require(minting_badge));
                    minter_updater => owner_rule.clone();
                ))
                .burn_roles(burn_roles!(
                    burner => owner_rule.clone();
                    burner_updater => owner_rule.clone();
                ))
                .withdraw_roles(withdraw_roles!(
                    withdrawer => rule!(allow_all);
                    withdrawer_updater => owner_rule.clone();
                ))
                .deposit_roles(deposit_roles!(
                    depositor => rule!(allow_all);
                    depositor_updater => owner_rule.clone();
                ))
                .recall_roles(recall_roles!(
                    recaller => recaller_rule.clone();
                    recaller_updater => recaller_rule.clone();
                ))
                .freeze_roles(freeze_roles!(
                    freezer => recaller_rule.clone();
                    freezer_updater => recaller_rule;
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => owner_rule.clone();
                    non_fungible_data_updater_updater => owner_rule;
                ));

            if name.is_some() || description.is_some() {
                let mut metadata = MetadataInit::new();
                if let Some(name) = name {
                    metadata.set_and_lock_metadata(
                        "name", name);
                }
                if let Some(description) = description {
                    metadata.set_and_lock_metadata(
                        "description", description);
                }
                builder = builder.metadata(ModuleConfig {
                    init: metadata,
                    roles: RolesInit::new(),
                });
            }
            
            builder
                .create_with_no_initial_supply()
                .address()
        }

        /// Activates the faucet, meaning that the `tap` method
        /// becomes available.
        pub fn activate(&mut self) {
            self.active = true;
        }

        /// Deactivates the faucet, causing calls to the `tap` method
        /// to start panicking.
        pub fn deactivate(&mut self) {
            self.active = false;
        }

        /// Determines whether the faucet is currently active or not.
        pub fn active(&self) -> bool {
            self.active
        }

        /// Changes the badge to use for minting tokens. You call this
        /// for one of three reasons:
        ///
        /// - To initially set a minting badge, passing a bucket
        /// holding the minting badge.
        ///
        /// - To turn off minting by passing it `None`.
        ///
        /// - To change the minting badge (possibly because the
        /// resource itself had its minting badge replaced) into a
        /// different one.
        ///
        /// If a minting badge was already set then the old minting
        /// badge bucket will be returned out of the method.
        pub fn set_minting_badge(&mut self, minting_badge: Option<Bucket>)
                                 -> Option<Bucket>
        {
            let orig_index = self.minting_badge_index;

            // First store the new minting badge
            self.minting_badge_index += 1;
            self.minting_badge.insert(
                self.minting_badge_index,
                minting_badge.map(|b| Vault::with_bucket(b)));

            // Then fetch the previous minting badge and return it to
            // the user. This leaves its empty vault behind which we
            // will never use again.
            //
            // (This whole thing would be a lot easier to deal with if
            // there was a Vault::drop_empty_vault method that would
            // let us simply delete a vault when it's empty.)
            let mut old_badge = self.minting_badge.get_mut(&orig_index);
            let old_badge = old_badge.as_mut().unwrap();
            if old_badge.is_some() {
                let v = old_badge.as_mut().unwrap();
                Some(v.take_all())
            } else {
                None
            }
        }

        /// Determines the identity of the current minting badge(s),
        /// returning the resource address and all the non-fungible
        /// local ids in our minting badge bucket.
        ///
        /// If the current minting badge is fungible then this method
        /// will panic.
        pub fn minting_badge_as_non_fungible(&self) ->
            Option<(ResourceAddress, BTreeSet<NonFungibleLocalId>)>
        {
            let old_badge = self.minting_badge.get(&self.minting_badge_index);
            let old_badge = old_badge.as_ref().unwrap();
            if old_badge.is_some() {
                let v = old_badge.as_ref().unwrap();
                Some(( v.resource_address(),
                  v.as_non_fungible().non_fungible_local_ids() ))
            } else {
                None
            }
        }

        /// Determines the identity of the current minting badge(s),
        /// returning the resource address and quantity of tokens in
        /// our minting badge bucket.
        ///
        /// This method works equally well whether our minting badge
        /// is fungible or non-fungible, but asking for the amount is
        /// a fungible-ish activity hence the method name.
        pub fn minting_badge_as_fungible(&self) ->
            Option<(ResourceAddress, Decimal)>
        {
            let old_badge = self.minting_badge.get(&self.minting_badge_index);
            let old_badge = old_badge.as_ref().unwrap();
            if old_badge.is_some() {
                let v = old_badge.as_ref().unwrap();
                Some(( v.resource_address(),
                       v.amount() ))
            } else {
                None
            }
        }

        /// Adds tokens to our treasury. If the input bucket holds
        /// tokens other than our intended token type this method
        /// panics.
        pub fn fund_treasury(&mut self, funds: Bucket) {
            assert_eq!(self.token, funds.resource_address(),
                       "wrong resource");
            
            if let Some(ref mut vault) = self.treasury {
                vault.put(funds);
            } else {
                self.treasury = Some(Vault::with_bucket(funds));
            }
        }

        /// Takes tokens out of our treasury. This works for fungibles
        /// and non-fungibles alike.
        pub fn defund_treasury_fungible(&mut self, amount: Decimal) -> Bucket {
            self.treasury.as_mut().unwrap().take(amount)
        }
        
        /// Takes tokens out of our treasury.
        ///
        /// If our treasury tokens are fungible then this method
        /// panics.
        pub fn defund_treasury_non_fungible(&mut self,
                                            nflids: BTreeSet<NonFungibleLocalId>)
                                            -> Bucket {
            self.treasury.as_mut().unwrap().as_non_fungible()
                .take_non_fungibles(&nflids)
                .into()
        }

        /// Takes all tokens out of our treasury.
        pub fn empty_treasury(&mut self) -> Bucket {
            self.treasury.as_mut().unwrap().take_all()
        }

        /// Determines the contents of our treasury. This method works
        /// equally well for fungible and non-fungible treasuries.
        pub fn treasury_as_fungible(&self) -> Decimal {
            if let Some(vault) = &self.treasury {
                vault.amount()
            } else {
                Decimal::ZERO
            }
        }

        /// Determines the contents of our treasury.
        ///
        /// If our treasury tokens are fungible then this method
        /// panics.
        pub fn treasury_as_non_fungible(&self) -> BTreeSet<NonFungibleLocalId> {
            if let Some(vault) = &self.treasury {
                vault.as_non_fungible().non_fungible_local_ids()
            } else {
                BTreeSet::new()
            }
        }

        /// Retrieves funds from the faucet, as per the ruleset
        /// specified for it. This is the method our users call to
        /// make use of the faucet service.
        ///
        /// This method can panic for a few main reasons:
        ///
        /// - The faucet isn't currently active.
        ///
        /// - You asked for 0 tokens
        ///
        /// - You're not authorized to tap.
        ///
        /// - You are trying to tap beyond the limit restriction.
        ///
        /// - You are asking us to mint a non-whole number of
        /// non-fungibles.
        ///
        /// - The treasury has insufficient tokens and we cannot mint
        /// to cover the shortfall.
        pub fn tap(&mut self, taker: Option<Proof>, amount: Decimal) -> Bucket {
            assert!(self.active, "faucet not active");

            // We don't want requests for zero tokens to enter into
            // our logic below, resetting period counters etc., so we
            // just shortcut it here. (An alternative way of handling
            // it would have been to return an empty bucket.)
            assert!(!amount.is_zero(), "ask for more than zero tokens");
            
            // If these get set to false it's because tapping got
            // limit-checked by NFT-local restrictions and further
            // checking is not needed.
            let mut use_global_cooldown_limit = true;
            let mut use_global_max_tap_limit = true;

            let now = unix_time_now();

            if let Some(taker_badges) = self.taker_badges {
                // This is our intent-based access check
                let taker = taker.unwrap().check(taker_badges);

                if taker_badges.is_fungible() {
                    assert!(taker.amount()
                            >= self.minimum_fungible_badge_tokens.unwrap(),
                            "not enough badges");
                } else {
                    use_global_cooldown_limit = false;
                    let taker = taker.as_non_fungible();
                    let taker_nfgid = NonFungibleGlobalId::new(
                        taker.resource_address(), taker.non_fungible_local_id());

                    if let Some(tap_cooldown_secs) = self.tap_cooldown_secs {
                        use_global_max_tap_limit = false;
                        
                        if self.cooldown_periods.get(&taker_nfgid).is_none() {
                            // Baby's first cooldown period
                            self.cooldown_periods.insert(
                                taker_nfgid.clone(),
                                CooldownPeriod {
                                    period_start: None, // Will be filled in later
                                    tap_total: Decimal::ZERO,
                                });
                        }

                        let mut cooldown_period =
                            self.cooldown_periods.get_mut(&taker_nfgid).unwrap();

                        let start_new_period =
                            if let Some(period_start) = cooldown_period.period_start
                        {
                            info!("now {}, period_start {}, tap_cooldown_secs {}",
                                  now, period_start, tap_cooldown_secs);
                            now - period_start >= tap_cooldown_secs as i64
                        } else { true };

                        if start_new_period {
                            cooldown_period.period_start = Some(now);
                            cooldown_period.tap_total = Decimal::ZERO;
                        } else {
                            // If we're within the same period as the
                            // previous tap, we only allow this if
                            // there's a max-per-period limit to count
                            // it against.
                            assert!(self.max_per_tap.is_some(),
                                    "tap less often");
                        }
                        
                        if let Some(max_per_tap) = self.max_per_tap {
                            // Check remaining limit
                            assert!(max_per_tap
                                    >= cooldown_period.tap_total + amount,
                                    "you tap too much");
                        }
                        cooldown_period.tap_total += amount;
                    } 
                }
            } else {
                // There is no access check here because if we get
                // here the faucet is configured to be open for all.
            }

            if use_global_cooldown_limit {
                if let Some(tap_cooldown_secs) = self.tap_cooldown_secs {
                    if let Some(last_tap) = self.last_tap {
                        assert!(now - last_tap >= tap_cooldown_secs as i64,
                                "tap too frequent");
                    }
                }
            }

            if use_global_max_tap_limit {
                if let Some(max_per_tap) = self.max_per_tap {
                    assert!(max_per_tap >= amount, "tap too big");
                }
            }

            self.last_tap = Some(now);

            // If we got all this way without a panic then it's time
            // to pay out.
            self.pay_out(amount)
        }

        /// Helper method to generate the funds we need to give to the
        /// user. It primarily takes from treasury, and if the
        /// treasury isn't enough then it tries to mint. If it tries
        /// to mint but doesn't have a minting badge then it will
        /// panic.
        fn pay_out(&mut self, amount: Decimal) -> Bucket {
            let mut loot;
            if let Some(ref mut treasury) = self.treasury {
                if treasury.amount() >= amount {
                    return treasury.take(amount);
                }
                loot = treasury.take_all();
            } else {
                loot = Bucket::new(self.token);
            }

            // If we get here then the treasury was not enough so we
            // have to mint the rest.

            let minting_badge =
                self.minting_badge.get(&self.minting_badge_index).unwrap();

            let mint_amount = amount - loot.amount();
            let mut mint_fn = || loot.put(self.mint_loot(mint_amount));

            if minting_badge.is_none() {
                // Mint without auth, may well fail but worth trying.
                // (Note that if we used the SELF virtual badge to
                // guard minting, the following statement would result
                // in a successful mint.)
                mint_fn();
            } else {
                let minting_badge = minting_badge.as_ref().unwrap();
                // Mint with auth
                if minting_badge.resource_address().is_fungible() {
                    // Fungible mint should never fail
                    minting_badge.as_fungible().authorize_with_amount(
                        minting_badge.amount(),
                        || mint_fn());
                } else {
                    let minting_badge = minting_badge.as_non_fungible();
                    minting_badge.authorize_with_non_fungibles(
                        &minting_badge.non_fungible_local_ids(),
                        || mint_fn());
                }
            }
            loot
        }

        /// Helper method to mint the tokens we give to our users.
        /// Authorization to mint must already have been established.
        fn mint_loot(&self, amount: Decimal) -> Bucket
        {
            if self.token.is_fungible() {
                ResourceManager::from(self.token).mint(amount)
            } else {
                // In an ideal world, counter would be an integer type.
                let mut counter = amount.floor();
                assert!(counter == amount,
                        "ask for whole number of non-fungibles");
                let mut loot_bucket = Bucket::new(self.token);
                
                // Note that non-fungible mint may fail if we mint too
                // many tokens, because of cost unit limits. Last time
                // this was tested - with Scrypto v0.11 - we could
                // mint about 160 before things started getting dicey.
                while counter >= Decimal::ONE {
                    loot_bucket.put(
                        ResourceManager::from(self.token).mint_ruid_non_fungible(
                            FaucetNfData {
                                faucet: Runtime::global_address(),
                                attributes: HashMap::new(),
                            }));
                    // This might be a very expensive way to do loop
                    // management.
                    counter -= Decimal::ONE;
                }
                loot_bucket
            }
        }
    }
}

/// Helper function to determine the current UNIX-era time.
fn unix_time_now() -> i64 {
    Clock::current_time_rounded_to_minutes().seconds_since_unix_epoch
}
