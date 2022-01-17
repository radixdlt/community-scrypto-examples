# Lending Platform

## About
Radix Scrypto blueprint for creating a lending platform

This blueprint allows you to perform the following actions:
- Add assets/tokens to the platform using an admin badge
- Create accounts on the lending platform
- Deposit multiple types of assets/tokens as 'deposits'
- Withdrawal deposited assets
- Borrow against deposited assets
- Repay loans

The platform currently tracks user balances w/ a HashMap and utilizes pre-defined LTV values when calculating collateral

Next features to add:
- Prices per asset (All assets currently have a 1:1 ratio with XRD)
- Price oracles (Calling from an external source instead of from within the component)

## Blueprint Functions
### Add assets/tokens to the platform using an admin badge
```bash
resim call-method "$lending_pool" new_asset "$xrd" 0.85 1,"$lending_platform_admin_badge"
```

### Create accounts on the lending platform
```bash
resim call-method "$lending_pool" new_user
```

### Deposit multiple types of assets/tokens as 'deposits'
```bash
resim call-method "$lending_pool" deposit_asset 1,"$lending_platform_badge_1" 10,"$xrd"
```

### Withdrawal deposited assets
```bash
resim call-method "$lending_pool" withdrawal_asset 1,"$lending_platform_badge_2" "$xrd" 27
```

### Borrow against deposited assets
```bash
resim call-method "$lending_pool" borrow_asset 1,"$lending_platform_badge_1" "$xrd" 15
```

### Repay loans
```bash
resim call-method "$lending_pool" repay_asset 1,"$lending_platform_badge_1" 20,"$xrd"
```

## Example Scenario
Included with this example is a file called `setup_and_build.sh` that performs the following:
- Creates a test admin account
- Creates 2 test user accounts
- Creates an instance of the lending platform
- Adds XRD to the lending pool with an LTV of 0.85
- Registers the 2 user accounts with the lending platform
- Deposits assets for both users
- Withdrawals assets for the 2nd user
- Borrows XRD against the collateral deposited by the 1st user
- Repays borrow XRD by the 1st user

To perform the above scenario, run the following: `source setup_and_build.sh`
