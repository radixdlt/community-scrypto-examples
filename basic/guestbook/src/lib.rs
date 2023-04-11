//! A guestbook that records visitors with a timestamp and a message
//! from the visitor.
//!
//! [instantiate_book] makes a new guestbook.
//!
//! [open_book] opens a book for visitors.
//!
//! [close_book] closes a book for visitors.
//!
//! [i_was_here] registers you as a visitor.
//!
//! [visitor_count] returns the number of visitors in the book.
//!
//! [who_was_there] returns a list of visitors.
//!
//! [instantiate_book]: crate::guestbook::Guestbook::instantiate_book
//! [open_book]: crate::guestbook::Guestbook::open_book
//! [close_book]: crate::guestbook::Guestbook::close_book
//! [i_was_here]: crate::guestbook::Guestbook::i_was_here
//! [visitor_count]: crate::guestbook::Guestbook::visitor_count
//! [who_was_there]: crate::guestbook::Guestbook::who_was_there

use scrypto::prelude::*;

/// This is just an alias we use for convenience, with details about a
/// visitor:
///
/// - .0 Timestamp (seconds since Unix epoch, minute accuracy)
///
/// - .1 Identity of visitor
///
/// - .2 A message from the visitor
type Visitor = (i64, NonFungibleGlobalId, String);

#[blueprint]
mod guestbook {
    struct Guestbook {
        /// Our visitors, with the most recent visitor at the end.
        visitors: Vec<Visitor>,

        /// Our admin resource address, restricted access to turning
        /// on and off the book
        admin_resource: ResourceAddress,

        /// Whether the book is currently accepting new visitors
        active: bool
    }

    impl Guestbook {

        /// Creates a new guestbook. A tuple is returned:
        ///
        /// - .0 is the component address of the new book
        ///
        /// - .1 is a bucket with the book's admin token. Use this to
        /// call `open_book` and `close_book`.
        ///
        /// Note that the book starts closed so for it to be useful
        /// you will need to call `open_book` at some point.
        pub fn instantiate_book() -> (ComponentAddress, Bucket) {

            let admin_badge = ResourceBuilder::new_fungible()
                .divisibility(0)
                .mint_initial_supply(1);
            
            let book =
                Self {
                    visitors: Vec::new(),
                    admin_resource: admin_badge.resource_address(),
                    active: false
                }
            .instantiate();

            let admin_res = admin_badge.resource_address();

            let access =
                AccessRulesConfig::new()
                .method("open_book",     rule!(require(admin_res)), LOCKED)
                .method("close_book",    rule!(require(admin_res)), LOCKED)
                .method("i_was_here",    rule!(allow_all), LOCKED)
                .method("visitor_count", rule!(allow_all), LOCKED)
                .method("who_was_there", rule!(allow_all), LOCKED)
                ;

            let book = book.globalize_with_access_rules(access);
            
            (book, admin_badge)
        }

        /// The administrator can use this to open the book for
        /// visitors. The admin badge must be in the auth zone.
        pub fn open_book(&mut self) {
            self.active = true;
        }
        
        /// The administrator can use this to close the book. It can
        /// be reopened again. The admin badge must be in the auth
        /// zone.
        pub fn close_book(&mut self) {
            self.active = false;
        }

        /// Adds a visit to the book. You must identify yourself with
        /// an NFT (any NFT) proof, and you may log a greeting message
        /// as well.
        pub fn i_was_here(&mut self, visitor: Proof, message: String) {
            assert!(self.active, "The book is closed.");

            // We don't care who it is, so long as it's anyone at
            // all. We will panic just below anyway if there's no one
            // in the proof.
            let visitor = visitor.unsafe_skip_proof_validation();
            self.visitors.push((
                Clock::current_time_rounded_to_minutes().seconds_since_unix_epoch,
                // This panics if the visitor proof doesn't contain an
                // NFT:
                NonFungibleGlobalId::new(visitor.resource_address(), visitor.non_fungible_local_id()),
                message));
        }

        /// Returns how many visitors we have in our book.
        pub fn visitor_count(&self) -> u64 {
            self.visitors.len() as u64
        }
        
        /// Returns the list of all our visitors if `count` is 0, or
        /// of just the `count` most recent ones if it is positive.
        /// Ordering of the returned elements is unspecified so if you
        /// need them sorted you must do so yourself.
        pub fn who_was_there(&self, count: u64) -> Vec<Visitor> {
            let count = count as usize;
            if count == 0 || count > self.visitors.len() {
                return self.visitors.clone();
            }
            return self.visitors.iter().rev().take(count).map(|t|t.clone()).collect();
        }
    }
}
