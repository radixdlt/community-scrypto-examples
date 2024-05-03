# Radix Stablecoin Vault

## Description

A Stablecoin vault blueprint that can

- mint tokens to the vault or to the admin badge holder
- withdraw tokens from the vault to admin bucket
- deposit tokens from admin bucket into the vault
- burn tokens inside the vault or from admin bucket
- get the vault status data: version, token amount, total supply
- set version to mark future upgrades

## Installation

The following steps assume working in a Linux environment.

See official installation guide for other operation systems:
https://docs-babylon.radixdlt.com/main/getting-started-developers/first-component/install-scrypto.html

Install build dependencies

```bash
sudo apt install clang build-essential llvm
```

### Install Rust

https://www.rust-lang.org/tools/install

If you already have Rust:

```bash
rustup update stable
```

OR install it the first time

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Add WebAssembly target

```bash
rustup target add wasm32-unknown-unknown
```

Add Rust in your PATH:

```
export PATH="~/.cargo/bin:$PATH"
```

To configure your current shell and confirm Rust installation:

```bash
source $HOME/.cargo/env
rustc --version
```

### Install Scrypto

```bash
git clone https://github.com/radixdlt/radixdlt-scrypto.git
cd radixdlt-scrypto
cargo install --path ./simulator
resim -V
```

### Install VS Code Extensions

- Rust-Analyzer
- Radix Transaction Manifest Support

```bash
code --install-extension rust-lang.rust-analyzer
code --install-extension RadixPublishing.radix-transaction-manifest-support
```

### Install Repository Dependencies

```bash
git clone https://github.com/AuroraLantean/radix_stablecoin_vault
cd radix_stablecoin_vault
scrypto build
```

This is only needed for the first time to install all Rust dependencies. For subsequent building, the above command can be omitted as running tests will automatically build everything again.

### Run Automatic Tests

Adding `-- --nocapture` will force logs to show even there is no error.

```bash
scrypto test
scrypto test -- --nocapture
```

### Run Manual Tests

1. Create a new account, and save the account address

```
resim new-account
```

2. Publish the package, and save the package address:

```
resim publish .
```

3. Call the `new` function to instantiate a component, and save the component address:

```
resim call-function <PACKAGE_ADDRESS> StableCoinVault new
```

4. Call the `mint_to_bucket` method of the component:

```
resim call-method <COMPONENT_ADDRESS> mint_to_bucket 100
```

5. Check out our balance

```
resim show <ACCOUNT_ADDRESS>
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