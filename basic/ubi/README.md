
# UBI Token Example

A UBI token implementation. Tokenomics are 1 token allocated per epoch and a 20% burn for transactions.

## Usage

1. Publish the package from a clean slate.

```
resim reset
resim new-account
resim publish <path_to_ubi>
```

2. Note the `<package_address>` of the published package and the `<admin_account_address>` of the new account.

```
resim call-function <package_address> UBI new
```

3. Note the `<ubi_token_address>`, `<admin_badge_address>`, `<person_badge_address>` and `<ubi_component>`. Register the admin account as a person.

```
resim call-method <ubi_component> register <admin_account_address> 1,<admin_badge_address>
```

4. Advance the epoch and collect some UBI.

```
resim set-current-epoch 10
resim call-method <ubi_component> available_tokens <person_badge_non_fungible_key>
resim call-method <ubi_component> collect_ubi 1,<person_badge_address>
resim call-method <ubi_component> available_tokens 1,<person_badge_address>
resim show <admin_account_address>
```

5. Create another account.

```
resim new-account
```

6. Note the `<account_address>` and `<account_pubkey>`. Send tokens to the new account (with burn).

```
resim call-method <ubi_component> send_tokens <account_address> 10,<ubi_token_address>
resim show <account_address>
resim show <admin_account_address>
```

7. Freely register other accounts and collect UBI with them!

```
resim call-method <ubi_component> register <account_address> 1,<admin_badge_address>
resim set-default-account <account_address> <account_pubkey>
resim set-current-epoch 20
resim call-method <ubi_component> collect_ubi 1,<person_badge_address>
```

## Notes

Once `RECALLABLE` is implemented, the ubi token can be made `RESTRICTED_TRANSFER` forcing the 20% burn in (a modified version of) `send_tokens`.