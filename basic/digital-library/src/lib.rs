use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Encode, Decode)]
struct Book {
    title: String,
    author: String,
}

blueprint! {
    struct Library {
        // librarian (admin) badge
        librarian_badge: ResourceDef,

        // holds the library funds, from late fees and memberships
        fees: Vault,

        // maps ISBN numbers to books
        books: HashMap<String, Book>,

        // each member receives a badge
        member_badges: Vault,
        member_badge_def: ResourceDef,

        // the cost of membership to the library
        membership_price: Decimal,

        // maps ISBN numbers to membership badge of borrower
        borrowed_books: HashMap<String, Address>
    }

    impl Library {
        // the library requires:
        // - member_badge_count: number of members allowed to join at once
        // - membership_price: the cost in XRD to join the library
        // the librarian badge is returned to the caller
        pub fn new(member_badge_count: u32, membership_price: Decimal) -> (Component, Bucket) {
            let librarian_badge_bucket = ResourceBuilder::new()
                .metadata("name", "Librarian Badge")
                .metadata("symbol", "LB")
                .new_badge_fixed(1);

            let mut books = HashMap::new();
            let book_one = Book { author: String::from("James S. A. Corey"), title: String::from("Leviathan Wakes") };
            let book_two = Book { author: String::from("Frank Herbert"), title: String::from("Dune") };
            let book_three = Book { author: String::from("Dan Abnett"), title: String::from("Horus Rising") };
            books.insert(String::from("9781611297560"), book_one);
            books.insert(String::from("9780450011849"), book_two);
            books.insert(String::from("9781844162949"), book_three);

            let member_badges_bucket = ResourceBuilder::new()
                .metadata("name", "Library Membership Badge")
                .metadata("symbol", "LMB")
                .new_badge_fixed(member_badge_count);
            let member_badge_def = member_badges_bucket.resource_def();

            let component = Self {
                librarian_badge: librarian_badge_bucket.resource_def(),
                fees: Vault::new(RADIX_TOKEN),
                books,
                member_badges: Vault::with_bucket(member_badges_bucket),
                member_badge_def,
                membership_price,
                borrowed_books: HashMap::new()
            }
            .instantiate();

            (component, librarian_badge_bucket)
        }

        // prints the state of the library
        pub fn print_library(&self) {
            info!("Membership price: {}, memberships available: {}", self.membership_price, self.member_badges.amount());
            info!("All books:");
            for (key, value) in &self.books {
                info!("{}: {}, {}", key, value.title, value.author);
            };
            info!("Borrowed books:");
            for (key, value) in &self.borrowed_books {
                info!("{}: {}", key, value);
            }
        }

        // registers the account as a member of the library
        pub fn register(&mut self, payment: Bucket) -> Bucket {
            info!("Attempting to register user, membership badges remaining {}, payment amount {}", self.member_badges.amount(), payment.amount());
            scrypto_assert!(!self.member_badges.is_empty(), "No memberships available");
            scrypto_assert!(payment.amount() == self.membership_price, "Wrong amount sent");
            scrypto_assert!(payment.resource_def() == RADIX_TOKEN.into(), "Can only pay with XRD");

            // take the payment
            self.fees.put(payment);

            // TODO: check for existing badge (maker_acct_addr: Address)

            // return the membership badge
            info!("Successfully registered user!");
            self.member_badges.take(1)
        }

        // borrow a book from the library
        #[auth(member_badge_def)]
        pub fn borrow_book(&mut self, isbn: String) {
            info!("Attempting to borrow book with ISBN {}", isbn);
            scrypto_assert!(self.books.contains_key(&isbn), "Book not in library");
            scrypto_assert!(!self.borrowed_books.contains_key(&isbn), "Book already borrowed");

            let book = self.books.get(&isbn).unwrap();
            info!("Book found (ISBN: {}, Title: {}, Author: {})", isbn, book.title, book.author);

            self.borrowed_books.insert(isbn, Self::get_user_id(&auth));
            info!("Book borrowed")
        }

        // return a borrowed book to the library
        #[auth(member_badge_def)]
        pub fn return_book(&mut self, isbn: String) {
            info!("Attempting to return book with ISBN {}", isbn);
            scrypto_assert!(self.borrowed_books.contains_key(&isbn), "Book not borrowed");
            let borrower_address = self.borrowed_books.get(&isbn).unwrap();
            let user_id = Self::get_user_id(&auth);
            info!("{} - {}", borrower_address, user_id);
            scrypto_assert!(borrower_address == &user_id, "Book not borrowed by this user");

            let book = self.books.get(&isbn).unwrap();
            info!("Book found (ISBN: {}, Title: {}, Author: {})", isbn, book.title, book.author);

            self.borrowed_books.remove(&isbn);
            info!("Book returned")
        }

        // pub fn pay_fee(&mut self, payment: Bucket)  {
        //     info!("Attempting to pay fee with payment amount: {}", payment.amount());
        //     // TODO: assert on amount
        //     self.fees.put(payment) // for now pay with any amount you want, no limitation
        // }

        // time to get paid! withdraws all fees made by the library
        #[auth(librarian_badge)]
        pub fn withdraw_fees(&mut self) -> Bucket {
            info!("Withdrawing all late fees: {}", self.fees.amount());
            self.fees.take_all()
        }

        // get user id from the provided badge
        fn get_user_id(badge: &BucketRef) -> Address {
            scrypto_assert!(badge.amount() > 0.into(), "Invalid badge provided");
            return badge.resource_address();
        }
    }
}
