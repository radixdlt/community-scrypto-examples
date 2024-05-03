# digital-library

This example implements a simple digital library of books.

There are two user types:

```rust
let librarian_badge_bucket = ResourceBuilder::new_fungible()
  .divisibility(DIVISIBILITY_NONE)
  .metadata("name", "Librarian Badge")
  .metadata("symbol", "LB")
  .initial_supply(1);
```

- single librarian user
- can withdraw all funds from the library

```rust
let member_badges_bucket = ResourceBuilder::new_fungible()
  .divisibility(DIVISIBILITY_NONE)
  .metadata("name", "Library Membership Badge")
  .metadata("symbol", "LMB")
  .initial_supply(member_badge_count);
```

- library member user
- member count determined by parameter passed into `new`
- can borrow, return and pay late fees on books

## Resources and Data

```rust
#[derive(TypeId, Encode, Decode, Describe)]
struct Book {
    title: String,
    author: String,
}
```

```rust
#[derive(TypeId, Encode, Decode, Describe)]
struct BorrowedBook {
    // when the user borrowed the book
    epoch: u64,
    user_id: ResourceAddress,
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
resim call-method $lib register <membership_price>,$xrd
```

- registers a new user to the library
  - `membership_price`: price in XRD to join the library
- returns the badge in a bucket

```
resim call-method $lib print_library
```

- prints the current status of the library including:
  - current epoch (used for determing late returns)
  - membership price and remaining badge count
  - all books
  - borrowed books

### Library Member Methods

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

## Tests

To run the tests:

```
scrypto test
```

The tests only cover a few of the functions/methods for now.


# License

The Radix Community Scrypto Examples code is released under Radix Modified MIT License.

    Copyright 2024 Radix Publishing Ltd

    Permission is hereby granted, free of charge, to any person obtaining a copy of
    this software and associated documentation files (the "Software"), to deal in
    the Software for non-production informational and educational purposes without
    restriction, including without limitation the rights to use, copy, modify,
    merge, publish, distribute, sublicense, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    This notice shall be included in all copies or substantial portions of the
    Software.

    THE SOFTWARE HAS BEEN CREATED AND IS PROVIDED FOR NON-PRODUCTION, INFORMATIONAL
    AND EDUCATIONAL PURPOSES ONLY.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
    FOR A PARTICULAR PURPOSE, ERROR-FREE PERFORMANCE AND NONINFRINGEMENT. IN NO
    EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES,
    COSTS OR OTHER LIABILITY OF ANY NATURE WHATSOEVER, WHETHER IN AN ACTION OF
    CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
    SOFTWARE OR THE USE, MISUSE OR OTHER DEALINGS IN THE SOFTWARE. THE AUTHORS SHALL
    OWE NO DUTY OF CARE OR FIDUCIARY DUTIES TO USERS OF THE SOFTWARE.