//! An on-ledger append-only database for recording the evolution of
//! digital documents.
//!
//! Such a database can be useful for publishing the existence of new
//! editions of documents, for proving that a particular document
//! existed at a certain date, etc.
//!
//! Choosing a ledger-based application for this (rather than, for
//! example, an *actual* append-only database) provides a readily
//! available solution (anyone can use the ledger, and it's reachable
//! from anywhere) and one that can clearly be trusted to not cheat
//! with its contents. This drastically reduces the barrier of entry
//! as compared to contracting with traditional online database
//! suppliers for the same type of service.
//!
//! Note that this blueprint does not store the documents themselves,
//! it only stores information about the existence of the documents:
//! The ledger is not a suitable place for general file storage, and
//! it is recommended that the information you store here about each
//! document is kept as small as possible.
//!
//! # Public interface
//!
//! These are the methods exposed by the blueprint to the outside
//! world:
//!
//! - [new] Creates a new indexer instance. Can be called by anyone.
//!
//! - [add_document] Creates a new document index. Can ony be called
//! by authorized users.
//!
//! - [register_document_edition] Adds an edition for a document. Can
//! only be called by authorized users.
//!
//! - [organization] Returns the organization name of the indexer. Can
//! be called by anyone.
//!
//! - [edition_count] Returns the number of editions for a
//! document. Can be called by anyone.
//!
//! - [editions] Returns detailed edition info for a document. Can be
//! called by anyone.
//!
//! - [add_funds] Adds funding to the subsidy mechanism. Can be called
//! by anyone.
//!
//! - [remove_funds] Removes subsidy funding. Can only be called by
//! admins.
//!
//! ## A typical call sequence
//!
//! 1. Alice creates user and admin badges using general ledger tools.
//! 2. Alice creates a new indexer by calling [new] and gives Bob a user badge.
//! 3. Alice calls [add_funds] to provide funding for her users.
//! 4. Bob calls [add_document] to create an (empty) index for a new document.
//! 5. Bob calls [register_document_edition] to register the initial
//! version for that document.
//! 6. Bob (and other users) keep calling [register_document_edition]
//! for the document as it evolves.
//! 7. Any number of additional documents could be created and tracked
//! in the same manner.
//!
//! Throughout, anyone may be calling [editions] and [edition_count]
//! to follow the development of a document.
//!
//! # About digital hashes
//!
//! This blueprint (the indexer) presumes the use of a strong hash
//! function to identify each edition of a document. The exact use of
//! this hash function is outside the scope of the blueprint as it
//! never actually handles the document itself. It is expected that
//! users will employ some tool that produces hashes for their
//! documents, and this is then the hash that they upload to the
//! indexer.
//!
//! As an example, on Linux you can use the `sha256sum` tool to
//! compute a strong cryptographic hash for a document. Here it is,
//! used on the Bitcoin whitepaper:
//! ```text
//! $ sha256sum -b Bitcoin_whitepaper.pdf 
//! 4488b60b08c7d548162418148b3dbd794b27f441024a338fad3e25fed4a74057 *Bitcoin_whitepaper.pdf
//! ```
//!
//! So if you were to register the Bitcoin whitepaper with the
//! indexer, then that's the hash you might use.
//!
//! The blueprint itself will put no actual requirements on you as to
//! which hash function to use, or how to use it. That is your own
//! decision based on your own security needs. SHA256 is pretty good
//! though and it's *right there* is all I'm saying.
//!
//! # About user roles
//!
//! The blueprint has three distinct user roles.
//!
//! The Owner is final arbiter of all administrative decisions.
//!
//! The Admins have the power to remove funds from the component.
//!
//! The Users are able to add documents and register new editions of
//! such.
//!
//! Some may wish to allow only Admins to call the [add_document]
//! method. In this case, simply change it from `restrict_to: [user]`
//! to `restrict_to: [admin]`. Or if you want both users *and* admins
//! to be able to use it, change it to `restrict_to: [user, admin]`.
//!
//! # About funding and subsidies
//!
//! If you are using this blueprint for professional purposes then you
//! may be in a situation where your actual users---or even their
//! organization---don't want to have to think about the decentralized
//! ledger at all while using your software. To them, the fact that
//! your indexer runs on the ledger is a boring implementation detail
//! and they fell asleep two words into your explanation. They don't
//! want to know about the ledger, don't want to have to deal with the
//! ledger, and certainly have no interest in keeping track of gas
//! costs for adding document registrations.
//!
//! To support this type of use case we provide a subsidy function
//! which enables you to add a pile of XRD to your indexer instance
//! and this pile will then be paying the transaction costs of your
//! users when they add documents and document editions. They can run
//! their daily operations without caring about XRD balances, and
//! someone else can keep an eye on the subsidy account and top it up
//! if necessary. Note that there must be at least 10 XRD in the
//! component for subsidies to be active.
//!
//! Any user account that wants to make use of the subsidy must have a
//! small amount of XRD in that account, simply as collateral for the
//! ledger until transaction execution reaches the point where subsidy
//! is established. Generally speaking, and at current cost levels, 1
//! XRD should be sufficient for this.
//!
//! The current subsidy implementation uses the `lock_fee` method
//! which means that we end up subsidizing failed transactions. This
//! is exactly what you might want for when you have a contractually
//! regulated professional relationship with your users: You don't
//! expect such users to be deliberately spamming failing transactions
//! just to troll you, and if there is an abundance of failed
//! transactions then it's probably a training issue or at worst
//! something to bring up during your next contract negotations.
//!
//! If instead of this your user base is random internet trolls then
//! primarily we recommend not subsidizing them at all but if you
//! *must* do so then you may want to switch the implementation over
//! to using `lock_contingent_fee` to remove at least the most obvious
//! attack vector on your subsidy funds. Also you may want to reduce
//! the subsidy from the current very generous 10 XRD to something
//! much closer to the expected actual transaction cost of the call.
//!
//! Note that when this component subsidizes a call with 10 XRD, those
//! 10 XRD go towards covering not only the processing in this
//! component but also any other processing that happens in the
//! enclosing transaction manifest. So long as you have trustworthy
//! users who just run transaction manifests exactly as they are built
//! by your own front-end this isn't a problem, but if you have more
//! enterprising and opportunistic users then they may change those
//! transaction manifests to include other expensive calls of their
//! own which you are then paying for. Again, our advice is to only
//! subsidize users you can trust or else to reduce the subsidy amount
//! from 10 XRD down to the actual cost of the method being called
//! (calculating this lower cost is left as an exercise for the
//! reader).
//!
//! # Use Cases
//!
//! This blueprint primarily provides a means of documenting the
//! existence of digital documents. While it also allows you to
//! register other metadata about each document (as `attributes`) the
//! main points of interest are the following three data fields:
//!
//! - Hash value
//! - Registration timestamp
//! - Identity of user making the registration
//!
//! The hash value, if used with proper care, uniquely identifies your
//! document among all documents in existence and as such makes it
//! undisputable which document is covered in this registration. Note
//! that the component doesn't compel anyone to produce the *actual*
//! document at all unless they are in a situation where they need to
//! make that proof: At this point they produce the document, a
//! trusted party computes its hash value, and this result is then
//! compared to the registry on ledger.
//!
//! The timestamp is generated by the ledger and so provides an
//! undisputable time and date at which that document definitely
//! existed.
//!
//! The user identity is determined through the use of ledger-verified
//! badges and so long as the user's keys have been kept secure gives
//! an undisputable record of who was responsible for making this
//! registration.
//!
//! (Note that as the contents of the accompanying `attributes` is
//! entirely user determined, the blueprint can make no claim about
//! the correctness of *that* data. Only the above three claims can be
//! confidently made.)
//!
//! A fourth guarantee made by this blueprint is that a registration
//! can never be deleted or replaced. As such, what you are seeing
//! here is the complete history of registrations ever made. The
//! blueprint logic itself has been designed to be easy to read such
//! that you can assure yourself that this is true, and of course the
//! ledger guarantees that this cannot be compromised *outside* of the
//! blueprint.
//!
//! As such, what we provide is a permanent and unfalsifiable record
//! of registrations each of which can be tied to a unique digital
//! document out there in the world.
//!
//! This has obvious usefulness for such diverse applications as
//!
//! - Publishing official documents
//! - Exposing document flows to be audited by external parties
//! - Supply chain management
//! - Proof of existence at a certain date
//!   - For copyright claims
//!   - For legal documents
//!   - To prove this was not recently AI generated content
//!   - To prove Trotsky was originally in the photograph
//!   - Etc.
//!
//! (Note that while the immutability of the ledger has -- to the
//! author's knowledge -- yet to be fully accepted by the court of
//! law, having this as *one* of your pieces of evidence cannot
//! possibly hurt.)
//!
//! ## Publishing official documents
//!
//! While one might argue that government (and big corporations)
//! should provide their own software solutions for their publishing
//! needs, their eager adoption of services such as Facebook and
//! Twitter for more casual publishing would seem to suggest they are
//! open to basing themselves on solutions provided and controlled by
//! private industry. The open ledger, being controlled by no one and
//! everyone, will with some likelihood become very enticing for
//! governments looking to reduce operating costs while avoiding
//! another billion dollar failed IT project.
//!
//! Additionally, using this blueprint on the open ledger to announce
//! the existence of editions of e.g. laws and regulations would force
//! a level of transparency on governments which in many cases is
//! sorely lacking. There exists a possible future in which e.g.
//! international lending organizations require of countries they lend
//! money to, to adopt exactly this kind of transparent solution for
//! their government work.
//!
//! We can also look towards academia where fraudulent degrees remain
//! a problem. If academic degrees were published in a suitably
//! anonymized fashion---something which this blueprint very much
//! caters to---then it becomes possible for employers to conduct a
//! simple audit of an applicant's credentials; as well as for
//! academic institutions to conduct forensic analysis on them after
//! malfeasance has been discovered. (Some might recall that Cardano
//! pursued a project along these lines a few years back.) As a brief
//! example, if an indexer is known to be run by a given university
//! and the `registering_user` is a professor there then an entry that
//! says "student Alice has earned a doctorate in genetic engineering"
//! makes all of that professor's other on-indexer claims suspect when
//! it is later revealed that Alice knows nothing about genetic
//! engineering whatsoever.
//!
//! ## Exposing document flows to be audited by external parties
//!
//! If you're in a company or a larger project (perhaps a
//! multi-company effort) that produces a lot of technical documents,
//! then it can become an almost impossible task for outside auditors
//! to keep track of and audit all of those documents. In particular
//! if documents are being produced on different sites or by different
//! organizations they can have widely varying practices wrt
//! producing, approving and archiving them, and trying to dig into
//! this can become a bit of a nightmare. This outside auditor could
//! be your ultimate customer, or it could be an external company
//! auditing your ISO9001 compliance, etc.
//!
//! A fairly low effort but high gain requirement in such a situation
//! is to make all participants register any new document or new
//! document edition to a common external service. The auditor then
//! has one single place to look for documents that should exist, and
//! will know exactly what to ask for when digging into any particular
//! aspect of the overall documentation of the project. Also, if a
//! project participant starts presenting documents that *do not*
//! exist in the external index, the auditor knows to put some extra
//! special care into investigating their internal document handling
//! practices. (Such a service is one of the key deliverables of the
//! Triall project, at triall.io)
//!
//! ## Supply chain management
//!
//! When you ship ten thousand gizmos from your factory in Taiwan and
//! somehow only eight thousand made it to warehouses in Europe, you
//! are going to want to start sticking an RFID sticker on each of
//! those gizmos to figure out what's going on with that. Or if you're
//! sending perishables across the country and relying on third
//! parties to keep them cool en route you may want to attach
//! RFID-enabled temperature sensors to them to keep your contractors
//! honest.
//!
//! Those are only two of the myriad of reasons you may want to track
//! the goods you're moving around. And whenever one of those RFIDs
//! gets scanned, the event needs to be stored somewhere. You can use
//! the indexer for this which provides an ever-present and
//! incorruptible source of truth for when you sit down to investigate
//! what's going wrong in your supply chain.
//!
//! Some may recall that VeChain has been very serious about this sort
//! of application.
//!
//! ## Proof of existence at a certain date
//!
//! Since the ledger doesn't lie about the timestamps it sets you can
//! know that whatever document a hash is referring to, that document
//! is at least as old as the date on its registration. This has all
//! sorts of applications.
//!
//! Note that the legal weight of the ledger's inherent
//! trustworthiness is up in the air at the moment, consider the
//! following to be from a bright future in which on-ledger timestamps
//! are considered strong legal evidence.
//!
//! You could write a secret will, register its hash, and later when
//! the full document is revealed it becomes clear that it was written
//! by you (the tx is signed by your account) while you were still
//! sound of mind (proven by the indexer's timestamp). (We need to get
//! soap operas in on this.)
//!
//! You could register a digital work of art (a photo of a painting
//! perhaps, or digital art, a piece of music, a novel, etc.) and use
//! this to defend your copyright claim should it come under attack,
//! the indexer giving you an undisputable timestamp to work
//! with. This might similarly be useful also in trademark or patent
//! disputes.
//!
//! If you have the world's best most ingenious plan and you want to
//! wow your friends with it, you could write it down, register its
//! hash on the indexer, and keep the plan itself secret. And then ten
//! years later when everything played out exactly as you planned it
//! you can reveal the original document and use the registered hash
//! to prove to everyone you're a Mycroft Holmes level strategic
//! genius.
//!
//! # Technical features of note
//!
//! ## Test suite
//!
//! The `tests` directory has a fairly comprehensive test suite that
//! runs through the functionality of the blueprint.
//!
//! ### Transaction manifests
//!
//! Transaction manifests for all publicly exposed methods are built
//! as part of the test suite. Look for the following methds in the
//! `tests/lib.rs` file:
//!
//! | Scrypto function            | Transaction manifest in |
//! |-----------------------------|-------------------------|
//! | [new]                       | `call_new()`
//! | [add_document]              | `build_add_document_manifest()`
//! | [register_document_edition] | `build_register_document_edition_manifest()`
//! | [organization]              | `call_organization()`
//! | [edition_count]             | `call_edition_count()`
//! | [editions]                  | `call_editions()`
//! | [add_funds]                 | `call_add_funds()`
//! | [remove_funds]              | `call_remove_funds()`
//!
//! Note that `add_document` and `register_document_edition` are
//! variously called via `execute_manifest_ignoring_fee` or
//! `execute_manifest`, with the latter used when we're testing the
//! subsidy mechanism. This is because normally we don't want fee
//! deductions to clutter up our tests: they're mostly just noise that
//! we need to expend effort to look past, and at worst they'll make
//! us run out of XRD on bigger tests which is just annoying. We do
//! need fees to be deducted when testing subsidy however because
//! that's how we can check if the fees are being taken from the
//! correct vault.
//!
//! ## Access control
//!
//! We establish system-level access control using our three main
//! roles (owner, admin, user) as well as the PUBLIC role. This
//! provides overall security for the blueprint.
//!
//! Notice that in the [register_document_edition] method we accept a
//! non-fungible local id in as argument and take this to be the user
//! that called us. We verify that this is legitimate by checking
//! against the Auth Zone that the user specified is actually
//! there. This gives us confidence that the `registering_user` that
//! we store is correct.
//!
//! As a consequence of doing this, we now have the `user` resource
//! address stored both in component state as `users`, and also in the
//! access rule for the `user` role like this: `user =>
//! rule!(require(users))`. This redundancy is a bit unfortunate as we
//! would prefer for security sensitive information such as this to be
//! stored in a single place only, to avoid confusion. As a practical
//! matter, if we were to implement support for altering the `user`
//! resource address we would have to remember to change it in both
//! places which opens up for making a mistake that causes ambiguity
//! as to what the correct address is.
//!
//! ## KeyValueStore
//!
//! In this blueprint we use the `KeyValueStore` type extensively to
//! enable us to store any amount of data. A Rust programmer's normal
//! inclination might be to use `HashMap` instead but the problem with
//! this is that when first the component initializes it has to load
//! the entirety of its state, which means the entire HashMap. If that
//! HashMap is very big then you can run out into the transaction fee
//! limit before the component is fully loaded, and you will never be
//! able to run anything in your component. The `KeyValueStore` type
//! has been specifically written to provide a map that doesn't have
//! this problem in that its contents isn't loaded until it is
//! requested.
//!
//! In one case what we *really* need is a vector with the lazy-load
//! properties of `KeyValueStore` and we simulate this by putting a
//! KeyValueStore into a struct together with an index counter, see
//! the [Document] struct for this.
//!
//! Also notice that we *do* use a `HashMap` for index entry
//! `attributes`, since we expect this particular map to always be so
//! small as to not have a cost limit problem. (And if you *do* run
//! into this problem you're probably using the indexer for use cases
//! it just wasn't designed for.)
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
//!
//! [new]: crate::document_indexer::DocumentIndexer::new
//! [add_document]: crate::document_indexer::DocumentIndexer::add_document
//! [register_document_edition]: crate::document_indexer::DocumentIndexer::register_document_edition
//! [organization]: crate::document_indexer::DocumentIndexer::organization
//! [edition_count]: crate::document_indexer::DocumentIndexer::edition_count
//! [editions]: crate::document_indexer::DocumentIndexer::editions
//! [add_funds]: crate::document_indexer::DocumentIndexer::add_funds
//! [remove_funds]: crate::document_indexer::DocumentIndexer::remove_funds

#![allow(rustdoc::private_intra_doc_links)]

use scrypto::prelude::*;

/// An index entry contains information on a particular edition of a
/// document. Some documents may only exist in a single edition but
/// many see constant change; and every time a document officially
/// publishes in a new edition, it should be reflected by an index
/// entry such as this.
#[derive(ScryptoSbor, Clone)]
pub struct IndexEntry {
    /// The user id that was used to make this registration.
    pub registering_user: NonFungibleGlobalId,

    /// The time on the ledger when this registration was made.  This
    /// value was calculated by the smart contract and so is
    /// authoritative. It is given as UNIX-era time in seconds.
    pub registration_timestamp: i64,

    /// This is the cryptographic hash of this edition of the
    /// document. It is central to the intended functionality of this
    /// blueprint and care should be taken to get this exactly
    /// right. Any hash algorithm can be used, and it can be encoded
    /// in any way the organization finds useful. Our recommendation
    /// is to encode binary hashes as a long hexadecimal number.
    pub document_hash: String,

    /// Can hold other attributes about the document, as the user
    /// deems appropriate. Some suggestions are:
    ///
    /// - url: URL to the document
    ///
    /// - path: Path to the document (presumably in some
    /// organization-internal file or document system)
    ///
    /// - title: If the document's title changed with this new
    /// edition. (The original title remains in use in the outer
    /// documents mapping.)
    ///
    /// - version: The version number of this edition of the document.
    ///
    /// - author: Author or authors of this edition of the
    /// document. Alternatively separate multiple authors into
    /// 'author1', 'author2'; or 'author', 'co-author1', 'co-author2'
    /// etc.
    ///
    /// - date: Formal publishing date of the document. We recommend
    /// ISO8601 format but whatever works for you.
    ///
    /// Note that since this is stored as a `HashMap` you don't want
    /// to put absurdly large amounts of data here.
    pub attributes: HashMap<String, String>
}

/// Holds the registered editions of a document.
///
/// This structure's only real task is to make the `KeyValueStore`
/// inside behave as if it were a vector. We want to use a
/// `KeyValueStore` because we need lazy load behaviour of the
/// potentially large amount of data we may end up holding.
#[derive(ScryptoSbor)]
struct Document {
    /// This is a strictly increasing number starting on 0 which
    /// determines the map key to use for the next IndexEntry. (We
    /// only need it because KeyValueStore doesn't provide an ordering
    /// of or iteration over its elements.)
    next_ndx: u64,

    /// This is the versioning history of the document. The keys are
    /// sequential starting on 0 and you can obtain all editions by
    /// iterating from key 0 up to `next_ndx-1` (inclusive). (This
    /// would also reproduce the chronological order in which the
    /// entries were originally added.)
    editions: KeyValueStore<u64, IndexEntry>,
}

#[blueprint] mod document_indexer {

    enable_method_auth! {
        roles {
            admin => updatable_by: [OWNER];
            user => updatable_by: [];
        },
        methods {
            organization => PUBLIC;
            edition_count => PUBLIC;
            editions => PUBLIC;
            add_funds => PUBLIC;
            add_document => restrict_to: [user];
            register_document_edition => restrict_to: [user];
            remove_funds => restrict_to: [admin];
        }
    }

    struct DocumentIndexer {
        /// The name of the organization using this component instance
        /// (if they cared to share).
        organization: Option<String>,

        /// The resource that all our users must present as badges in
        /// order to update the document registry.
        users: ResourceAddress,

        /// Funds used for subsidizing transactions updating our
        /// registry.
        funds: Vault,

        /// We want to be able to store a large number of entries so
        /// we use KeyValueStore for this.
        ///
        /// The key is the name / title / id of the document,
        /// determined by whatever scheme is used by the owning
        /// organization. It must be unique among all documents
        /// managed by this instance.
        ///
        /// The value is the information we have on that document.
        documents: KeyValueStore<String, Document>,
    }

    impl DocumentIndexer {

        /// Creates a new indexer component. It can index any number
        /// of different documents.
        ///
        /// `organization` is an optional name of the organization or
        /// individual who maintains this indexer instance.
        ///
        /// The `owner` badge will have authority to update admin
        /// access to the component.
        ///
        /// The starting list of admin badges is given in `admins`.
        ///
        /// Our users must be issued badges of the `users` resource
        /// which must be non-fungible.
        pub fn new(organization: Option<String>,
                   owner: NonFungibleGlobalId,
                   admins: BTreeSet<NonFungibleGlobalId>,
                   users: ResourceAddress)
                   -> Global<DocumentIndexer>
        {
            assert!(!users.is_fungible(),
                    "The users resource must be non-fungible.");
                
            Self {
                organization,
                users,
                funds: Vault::new(RADIX_TOKEN),
                documents: KeyValueStore::new()
            }
            .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(owner))))
                .roles(roles!(
                    admin => rule!(require_any_of(
                        admins.into_iter().collect::<Vec<NonFungibleGlobalId>>()));
                    user => rule!(require(users));
                ))
                .globalize()
        }

        /// Returns the name of the organization that uses this
        /// component.
        pub fn organization(&self) -> Option<String> {
            self.organization.clone()
        }

        /// Adds knowledge of a new document to the indexer. The
        /// document starts without any versioning data.
        ///
        /// The `document_name` must be unique within this component
        /// instance and if it is not the method will panic.
        ///
        /// If there are sufficient funds available then this method
        /// call will receive transaction fee subsidy.
        pub fn add_document(&mut self, document_name: String)
        {
            self.subsidize();
            assert!(self.documents.get(&document_name).is_none(),
                    "This document already exists");
            
            self.documents.insert(document_name,
                                  Document {
                                      next_ndx: 0,
                                      editions: KeyValueStore::new(),
                                  });
        }

        /// Adds knowledge of a new edition of a document to the
        /// indexer.
        ///
        /// `user` must be the local-id of one of our users that is
        /// currently in the Auth Zone. The registry addition will be
        /// attributed to this user.
        ///
        /// The `document_name` must already exist within this
        /// component instance and if it does not the method will
        /// panic.
        ///
        /// The `document_hash` should be the cryptographic hash of
        /// the document.
        ///
        /// You may provide further data about the document in
        /// `attributes`, the exact usage of which is left to the
        /// owner of each component instance.
        ///
        /// If there are sufficient funds available then this method
        /// call will receive transaction fee subsidy.
        pub fn register_document_edition(&mut self,
                                         user: NonFungibleLocalId,
                                         document_name: String,
                                         document_hash: String,
                                         attributes: HashMap<String, String>)
        {
            // Note that we subsidize before we do the auth zone
            // checking, since system-level access control already
            // guarantees the caller is one of our users.
            self.subsidize();

            let registering_user = NonFungibleGlobalId::new(self.users, user);
            Runtime::assert_access_rule(rule!(require(registering_user.clone())));

            let mut doc = self.documents.get_mut(&document_name).expect(
                "This document does not exist");
            doc.editions.insert(doc.next_ndx,
                                IndexEntry {
                                    registering_user,
                                    registration_timestamp: unix_time_now(),
                                    document_hash,
                                    attributes,
                                });
            doc.next_ndx += 1;
        }

        /// Returns the number of editions registered for
        /// `document_name`.
        ///
        /// Panics if the named document does not exist.
        pub fn edition_count(&self, document_name: String) -> u64 {
            self.documents.get(&document_name).unwrap().next_ndx
        }

        /// Returns some or all of the editions registered for
        /// `document_name`. If that document does not exist the
        /// method panics.
        ///
        /// State the index at which to `start` and how many editions
        /// to return (`count`). If `count` takes us past the last
        /// edition then we simply return a shorter list than that
        /// value might otherwise suggest.
        ///
        /// In order to return all editions specify 0 for `start` and
        /// a very large number for `count`. 
        ///
        /// If you are making this call on a service that has some
        /// form of rate limiting a large return vector might cause
        /// your query to be aborted and in this case you want to
        /// specify a `count` that is low enough to pass the rate
        /// limit. You can then iteratively call the method,
        /// increasing `start` each time using it as a cursor.
        ///
        /// (As an example of a rate limited service, if you are
        /// calling this method from a smart contract then you will be
        /// contending with the Scrypto cost unit limits.)
        pub fn editions(&self, document_name: String,
                        start: u64, count: u64) -> Vec<IndexEntry>
        {
            let doc = self.documents.get(&document_name).expect(
                "This document does not exist");

            let mut retvec = Vec::new();
            for ndx in start..start+count {
                if let Some(entry) = doc.editions.get(&ndx) {
                    retvec.push((*entry).clone());
                } else {
                    break;
                }
            }
            retvec
        }

        /// Puts funds into the subsidy account. These funds will
        /// automatically be used to subsidize everyone's calls to
        /// `add_document` and `register_document_edition`.
        ///
        /// The `funds` bucket must be of XRD or we will panic.
        pub fn add_funds(&mut self, funds: Bucket) {
            self.funds.put(funds);
        }

        /// Removes funds from the subsidy account.
        pub fn remove_funds(&mut self, amount: Decimal) -> Bucket {
            self.funds.take(amount)
        }

        /// Implementation of our fee subsidy: If there are sufficient
        /// funds we will subsidize the current transaction.
        ///
        /// Note that we use `lock_fee` here and not
        /// `lock_contingent_fee`. This is on the presumption that our
        /// users are professionals with whom we have some form of
        /// contractual relationship. We therefore trust them not to
        /// abuse the subsidy mechanism by spamming failing
        /// transactions etc. (If they still end up doing that then
        /// this is presumably a matter of training them better or of
        /// building a better front-end for them to use.)
        ///
        /// If our users had been internet trolls who we
        /// kind-of-but-not-really trust with money we might have used
        /// `lock_contingent_fee` instead.
        fn subsidize(&mut self) {
            if self.funds.amount() >= dec!(10) {
                self.funds.as_fungible().lock_fee(Decimal::ONE);
            }
        }
    }
}

/// Helper function to determine the current UNIX-era time
fn unix_time_now() -> i64 {
    Clock::current_time_rounded_to_minutes().seconds_since_unix_epoch
}
