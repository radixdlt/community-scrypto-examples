# airdrop

Example for an airdrop component that demonstrates how tokens can be sent to account addresses from within a component.
Additional the code in `tests/libs.rs` shows how token balances can be retrieved in unit tests.

The component is implemented in its simplest form to showcase the above two points. Authorization is not implemented.

# Implementation

An airdrop component is instantiated without any arguments. It only holds one field, which is a vector that tracks the
recipients of an airdrop:

```rust
pub fn new() -> Component {
    Self {
        recipients: Vec::new(),
    }
        .instantiate()
}
```

Before distributing an airdrop, one needs to first add at least on recipient to the component:

```rust
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
    let last_address = self.recipients.get(num_recipients - 1).unwrap();
    Account::from(*last_address).deposit(tokens);
}
```