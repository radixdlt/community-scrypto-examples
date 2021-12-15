# TokenBurn
Implementation of a component that "burns" all tokens that are sent to it.

The component burns tokens by putting them into a vault to which no one holds a reference - not even the component itself.
The tokens still exist on the ledger but are not accessible. Thus, they are effectively removed from circulation.

```rust
pub fn burn_tokens(&mut self, tokens_to_burn: Bucket) {
    self.burnt_amount += tokens_to_burn.amount();
    Vault::new(ResourceDef::from(self.token_address)).put(tokens_to_burn);
}
```