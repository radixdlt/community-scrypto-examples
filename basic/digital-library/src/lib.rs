use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Encode, Decode, Describe)]
struct Book {
    title: String,
    author: String,
}

#[derive(TypeId, Encode, Decode, Describe)]
struct BorrowedBook {
    // when the user borrowed the book
    epoch: u64,
    user_id: ResourceAddress,
}

blueprint! {
    struct Library {
        // librarian (admin) badge
        librarian_badge_def: ResourceAddress,

        // holds the library funds, from late fees and memberships
        fees: Vault,

        // maps ISBN numbers to books
        books: HashMap<String, Book>,

        // member badge
        member_badges: Vault,
        member_badge_def: ResourceAddress,
        // membership cost in XRD
        membership_price: Decimal,

        // maps ISBN numbers to membership badge of borrower
        borrowed_books: HashMap<String, BorrowedBook>,
        // the number of epochs books can be borrowed for
        borrow_epochs: u64
    }

    impl Library {
        // the library requires:
        // - member_badge_count: number of members allowed to join at once
        // - membership_price: the cost in XRD to join the library
        // - borrow_epochs: the number of epochs books are borrowed for
        // the librarian badge is returned to the caller
        pub fn new(member_badge_count: u32, membership_price: Decimal, borrow_epochs: u64) -> (ComponentAddress, Bucket) {
            let librarian_badge_bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Librarian Badge")
                .metadata("symbol", "LB")
                .initial_supply(1);

            let mut books = HashMap::new();
            let book_one = Book { author: String::from("James S. A. Corey"), title: String::from("Leviathan Wakes") };
            let book_two = Book { author: String::from("Frank Herbert"), title: String::from("Dune") };
            let book_three = Book { author: String::from("Dan Abnett"), title: String::from("Horus Rising") };
            books.insert(String::from("9781611297560"), book_one);
            books.insert(String::from("9780450011849"), book_two);
            books.insert(String::from("9781844162949"), book_three);

            let member_badges_bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Library Membership Badge")
                .metadata("symbol", "LMB")
                .initial_supply(member_badge_count);
            let member_badge_def = member_badges_bucket.resource_address();

            let component = Self {
                librarian_badge_def: librarian_badge_bucket.resource_address(),
                fees: Vault::new(RADIX_TOKEN),
                books,
                member_badges: Vault::with_bucket(member_badges_bucket),
                member_badge_def,
                membership_price,
                borrowed_books: HashMap::new(),
                borrow_epochs: borrow_epochs
            }
            .instantiate();

            let access_rules = AccessRules::new()
                .method("withdraw_fees", rule!(require(librarian_badge_bucket.resource_address())))
                .default(rule!(allow_all));

            (component.add_access_check(access_rules).globalize(), librarian_badge_bucket)
        }

        // prints the state of the library
        pub fn print_library(&self) {
            info!("Current epoch, {}", {Runtime::current_epoch()});
            info!("Membership price: {}, memberships available: {}", self.membership_price, self.member_badges.amount());
            info!("All books:");
            for (isbn, book) in &self.books {
                info!("{}: {}, {}", isbn, book.title, book.author);
            };
            info!("Borrowed books:");
            for (isbn, borrowed_book) in &self.borrowed_books {
                info!("{}: {}, {}", isbn, borrowed_book.user_id, borrowed_book.epoch);
            }
        }

        // registers the account as a member of the library
        pub fn register(&mut self, payment: Bucket) -> Bucket {
            info!("Attempting to register user, membership badges remaining {}, payment amount {}", self.member_badges.amount(), payment.amount());
            assert!(!self.member_badges.is_empty(), "No memberships available");
            assert!(payment.amount() == self.membership_price, "Wrong amount sent");
            assert!(payment.resource_address() == RADIX_TOKEN, "Can only pay with XRD");

            // take the payment
            self.fees.put(payment);

            // return the membership badge
            info!("Successfully registered user");
            self.member_badges.take(1)
        }

        // borrow a book from the library
        pub fn borrow_book(&mut self, isbn: String, auth: Proof) {
            info!("Attempting to borrow book with ISBN {}", isbn);
            assert!(auth.resource_address() == self.member_badge_def);
            assert!(self.books.contains_key(&isbn), "Book not in library");
            assert!(!self.borrowed_books.contains_key(&isbn), "Book already borrowed");

            let book = self.books.get(&isbn).unwrap();
            info!("Book found (ISBN: {}, Title: {}, Author: {})", isbn, book.title, book.author);

            // create and save a record of the book being borrowed
            let borrowed_book = BorrowedBook { epoch: Runtime::current_epoch(), user_id: Self::get_user_id(&auth) };
            self.borrowed_books.insert(isbn, borrowed_book);
            info!("Book borrowed")
        }

        // return a borrowed book to the library
        pub fn return_book(&mut self, isbn: String, auth: Proof) {
            info!("Attempting to return book with ISBN {}", isbn);
            assert!(auth.resource_address() == self.member_badge_def);
            let borrowed_book = self.get_borrowed_book(&isbn, &auth);

            // check the book is not overdue
            let book_overdue = self.is_book_overdue(borrowed_book);
            assert!(!book_overdue, "Book is overdue");

            self.borrowed_books.remove(&isbn);
            info!("Book returned")
        }

        // pays the late fee and returhs the book to the library
        pub fn pay_fee(&mut self, isbn: String, payment: Bucket, auth: Proof)  {
            info!("Attempting to pay fee with payment amount: {}", payment.amount());
            assert!(auth.resource_address() == self.member_badge_def);
            let borrowed_book = self.get_borrowed_book(&isbn, &auth);

            // check the book is overdue
            let book_overdue = self.is_book_overdue(borrowed_book);
            assert!(book_overdue, "Book is not overdue");

            // check the payment is correct
            assert!(payment.amount() == Decimal::one(), "Wrong amount sent");
            assert!(payment.resource_address() == RADIX_TOKEN.into(), "Can only pay with XRD");

            // take the payment and remove the borrowed book record
            self.fees.put(payment);
            self.borrowed_books.remove(&isbn);
            info!("Late fee paid and book returned")
        }

        // time to get paid! withdraws all fees made by the library
        pub fn withdraw_fees(&mut self) -> Bucket {
            info!("Withdrawing all late fees: {}", self.fees.amount());
            self.fees.take_all()
        }

        // using an ISBN and user ID returns the borrowed book
        fn get_borrowed_book(&self, isbn: &String, badge: &Proof) -> &BorrowedBook {
            // check the book is borrowed and the borrower is the current user
            assert!(self.borrowed_books.contains_key(isbn), "Book not borrowed");
            let borrowed_book = self.borrowed_books.get(isbn).unwrap();
            let user_id = Self::get_user_id(&badge);
            assert!(borrowed_book.user_id == user_id, "Book not borrowed by this user");

            let book = self.books.get(isbn).unwrap();
            info!("Book found (ISBN: {}, Title: {}, Author: {})", isbn, book.title, book.author);
            return borrowed_book
        }

        // returns if the given borrowed book is currently overdue
        fn is_book_overdue(&self, borrowed_book: &BorrowedBook) -> bool {
            // book is overdue if current epoch is past when the book was borrowed plus the allowed borrow time
            let book_overdue = Runtime::current_epoch() > borrowed_book.epoch + self.borrow_epochs;
            info!("Book borrowed on epoch {}, current epoch {}, overdue = {}", borrowed_book.epoch, Runtime::current_epoch(), book_overdue);
            return book_overdue
        }

        // get user ID from the provided badge
        fn get_user_id(badge: &Proof) -> ResourceAddress {
            assert!(badge.amount() > 0.into(), "Invalid badge provided");
            return badge.resource_address();
        }
    }
}
