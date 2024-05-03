# airdrop

Example for an airdrop component that demonstrates how tokens can be sent to account addresses from within a component.
Additional the code in `tests/libs.rs` shows how token balances can be retrieved in unit tests.

The component is implemented in its simplest form to showcase the above two points.

# Implementation

An airdrop component is instantiated without any arguments. It only holds two fields, which are a vector that tracks
the recipients of an airdrop and an admin badge that is used for authorization:

```rust
pub fn new() -> (Component, Bucket) {
    let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .initial_supply_fungible(1);

    let component = Self {
        admin_badge: admin_badge.resource_def(),
        recipients: Vec::new()
    }
    .instantiate();

    (component, admin_badge)
}
```

Before distributing an airdrop, one needs to first add at least on recipient to the component:

```rust
#[auth(admin_badge)]
pub fn add_recipient(&mut self, recipient: Address) {
    self.recipients.push(recipient);
}
```

When sending an airdrop, the component calculates the amount of tokens that every recipient should receive. Tokens are
distributed evenly between all recipient. Note, however, that the last recipient is handled a bit differently as they
are given the remaining tokens in the bucket after all other recipient have gotten their share. This is done so that no
tokens remain in the input bucket due to rounding errors. If we were to implement this naively, the Scrypto Engine would
stop us in certain scenarios.

```rust
#[auth(admin_badge)]
pub fn perform_airdrop(&self, tokens: Bucket) {
    let num_recipients = self.recipients.len();
    assert!(num_recipients > 0, "You must register at least one recipient before performing an airdrop");

    // Calculate the amount of tokens each recipient can receive
    let amount_per_recipient = tokens.amount() / Decimal::from(num_recipients as i128);

    // Send that amount to all but the last recipients
    for i in 0..(num_recipients - 1) {
        let address = self.recipients.get(i).unwrap();
        Account::from(*address).deposit(tokens.take(amount_per_recipient));
    }

    // Send the remaining tokens to the last recipient.
    // This should be almost exactly the calculated amount except for rounding errors. This way we can be sure
    // not to loose any tokens.
    let last_address = self.recipients.get(num_recipients-1).unwrap();
    Account::from(*last_address).deposit(tokens);
}
```


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