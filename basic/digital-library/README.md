# digital-library

This example implements a simple digital library of books.

There are two user types:

```rust
let librarian_badge_bucket = ResourceBuilder::new()
    .metadata("name", "Librarian Badge")
    .metadata("symbol", "LB")
    .new_badge_fixed(1);
```

- single librarian user
- can withdraw all funds from the library

```rust
let member_badges_bucket = ResourceBuilder::new()
    .metadata("name", "Library Membership Badge")
    .metadata("symbol", "LMB")
    .new_badge_fixed(member_badge_count);
```

- library member user
- member count determined by parameter passed into `new`
- can borrow, return and pay late fees on books

## Resources and Data

```rust
#[derive(TypeId, Encode, Decode)]
struct Book {
    title: String,
    author: String,
}
```

```rust
#[derive(TypeId, Encode, Decode)]
struct BorrowedBook {
    // when the user borrowed the book
    epoch: u64,
    user_id: Address,
}
```

## Methods and Functions

```
resim call-function $package Library new <member_badge_count> <membership_price> <borrow_epochs>
```

- creates a new library component
  - `member_badge_count`: number of members allowed
  - `membership_price`: price in XRD to join the library
  - `borrow_epochs`: number of epochs a user can borrow a book for

```
resim call-method $lib register 1,$xrd
```

- registers a new user to the library
- requires payment in XRD to join
- returns the badge in a bucket

```
resim call-method $lib print_library
```

- prints the current status of the library including:
  - current epoch (used for determing late returns)
  - membership price and remaining badge count
  - all books
  - borrowed books

### Member methods

All the below methods require the user to pass a member's badge.

Note: to make a library book require a late fee call `resim set-current-epoch <epoch>`

```
resim call-method $lib borrow_book <isbn> 1,$lmb
```

- borrows a book from the library
  - `isbn`: book to borrow
- internally stores the user ID and epoch of when the book was borrowed

```
resim call-method $lib return_book <isbn> 1,$lmb
```

- returns a book to the library
  - `isbn`: book to return
- fails if the book requires a late fee (user must cal `pay_fee` instead)

```
resim call-method $lib pay_fee <isbn> <late_fee> 1,$lmb
```

- pays for the late fee and returns the book
  - `isbn`: book to return
  - `late_fee`: fee to be paid in XRD

### Librarian Methods

The below method requires the user to pass the librarian badge.

```
resim call-method $lib withdraw_fees 1,$lb
```

- withdraws all membership and late fees from the library
