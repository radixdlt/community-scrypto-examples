# minimal-e-commerce

The goal of this blueprint is to provide a simple and naive example of an e-commerce platform.

## Catalog

The catalog holds the article's reference data, the article's stock and the cashier.

Each reference of the catalog has metadata such as its ID in the catalog, its name, its unit price,... _but not its quantity_. Indeed, the stock is not part of a reference metadata.

On the other hand, each article is a physical projection of the reference and have a stock, which can be refilled or burned (imagine a perishable good). Each article in the stock is identified by a single ID among the reference.

> Example:
>
> AdvInk is an industrial ink provider.
> It provides Blue and Black ink bottles.
> Blue ink has a unit price of 10 XRD/bottle, and Black ink has a unit price of 2 XRD/bottle.
> The current state of its stock is 1 Blue ink and 3 Black ink.
> The catalog would look like:
> - references:
>   - Blue ink bottle: 10 XRD
>   - Black ink bottle: 2 XRD
> - stock:
>   - Blue ink bottles:
>     - 1: Blue ink bottle: 10 XRD
>   - Black ink bottles:
>     - 1: Black ink bottle: 2 XRD
>     - 2: Black ink bottle: 2 XRD
>     - 3: Black ink bottle: 2 XRD
> - cashier: 0 XRD

### Access rules

In this catalog, we are able to:

<style type="text/css">
.tg  {border-collapse:collapse;border-spacing:0;}
.tg td{border-color:black;border-style:solid;border-width:1px;font-family:Arial, sans-serif;font-size:14px;
  overflow:hidden;padding:10px 5px;word-break:normal;}
.tg th{border-color:black;border-style:solid;border-width:1px;font-family:Arial, sans-serif;font-size:14px;
  font-weight:normal;overflow:hidden;padding:10px 5px;word-break:normal;}
.tg .tg-c3ow{border-color:inherit;text-align:center;vertical-align:top}
.tg .tg-dvpl{border-color:inherit;text-align:right;vertical-align:top}
</style>
<table class="tg">
<thead>
  <tr>
    <th class="tg-dvpl"><br><span style="font-weight:bold">Role</span><br><span style="font-weight:bold">Action&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;</span></th>
    <th class="tg-c3ow"><span style="font-weight:bold"><br/>Owner</span></th>
    <th class="tg-c3ow"><span style="font-weight:bold"><br/>Employee</span></th>
    <th class="tg-c3ow"><span style="font-weight:bold"><br/>Customer</span></th>
  </tr>
</thead>
<tbody>
  <tr>
    <td class="tg-dvpl">Become employee</td>
    <td class="tg-c3ow">✅</td>
    <td class="tg-c3ow">❌</td>
    <td class="tg-c3ow">❌</td>
  </tr>
  <tr>
    <td class="tg-dvpl">Add new references</td>
    <td class="tg-c3ow">❌</td>
    <td class="tg-c3ow">✅</td>
    <td class="tg-c3ow">❌</td>
  </tr>
  <tr>
    <td class="tg-dvpl">Fill stocks</td>
    <td class="tg-c3ow">❌</td>
    <td class="tg-c3ow">✅</td>
    <td class="tg-c3ow">❌</td>
  </tr>
  <tr>
    <td class="tg-dvpl">Buy products</td>
    <td class="tg-c3ow">✅</td>
    <td class="tg-c3ow">✅</td>
    <td class="tg-c3ow">✅</td>
  </tr>
  <tr>
    <td class="tg-dvpl">Collect cashier</td>
    <td class="tg-c3ow">✅</td>
    <td class="tg-c3ow">❌</td>
    <td class="tg-c3ow">❌</td>
  </tr>
</tbody>
</table>

## Technical

### Bootstrap

To bootstrap the project, assuming you have `resim` installed, you can simply run a `source init.sh`.

<details>
<summary>Output example</summary>

```console
me@os:~$ source init.sh
    Finished release [optimized] target(s) in 0.04s
Data directory cleared.
======================================
========= BEFORE WITHDRAWING =========
=========      CATALOG       =========
=========                    =========
Component: 0235dd53bb575710382be3706f20562e86bd9fd191211007fde499
Blueprint: { package_address: 01fe7b134365efaae977274d150d36b6355a7592ad52423a80e9c2, blueprint_name: "Catalog" }
Authorization
├─ "become_minter" => Protected(ProofRule(Require(StaticResource(03d3a74e295e581fde00024b1339798052d5c1ffb324d497297488))))
├─ "withdraw" => Protected(ProofRule(Require(StaticResource(03d3a74e295e581fde00024b1339798052d5c1ffb324d497297488))))
├─ "add_stock_to_reference" => Protected(ProofRule(Require(StaticResource(03687672452e3cca5d202e36a587bb5b3f5cf471c873964bc84486))))
└─ "register_reference" => Protected(ProofRule(Require(StaticResource(03687672452e3cca5d202e36a587bb5b3f5cf471c873964bc84486))))
State: Struct(1u32, HashMap<U32, Struct>(1u32, Struct(1u32, 5u32, "Black ink 100 mL", Decimal("150"))), HashMap<U32, Vault>(1u32, Vault("f4131de2f69e0428b77d0abc549cc0aca280b4281fe980b0c84efa7dbe3a760802040000")), Vault("6cc2eb7f95c40e1c7ec2881eea746d3b63fcb790091d42ac93c6ce1dbe42966704040000"), Vault("6cc2eb7f95c40e1c7ec2881eea746d3b63fcb790091d42ac93c6ce1dbe42966705040000"), ResourceAddress("03d3a74e295e581fde00024b1339798052d5c1ffb324d497297488"))
Resources:
├─ { amount: 300, resource address: 030000000000000000000000000000000000000000000000000004, name: "Radix", symbol: "XRD" }
├─ { amount: 3, resource address: 03aaea37ea01de894e47d28ded9398ac3c2c662270f38e1c44c2c4, name: "Article Black ink 100 mL" }
│  ├─ NON_FUNGIBLE { id: 00000003, immutable_data: Struct("Black ink 100 mL", Decimal("150")), mutable_data: Struct() }
│  ├─ NON_FUNGIBLE { id: 00000004, immutable_data: Struct("Black ink 100 mL", Decimal("150")), mutable_data: Struct() }
│  └─ NON_FUNGIBLE { id: 00000005, immutable_data: Struct("Black ink 100 mL", Decimal("150")), mutable_data: Struct() }
└─ { amount: 1, resource address: 03687672452e3cca5d202e36a587bb5b3f5cf471c873964bc84486, name: "Article minter" }
=========                    =========
=========       OWNER        =========
=========                    =========
Component: 020d3869346218a5e8deaaf2001216dc00fcacb79fb43e30ded79a
Blueprint: { package_address: 010000000000000000000000000000000000000000000000000003, blueprint_name: "Account" }
Authorization
├─ "deposit" => AllowAll
└─ "deposit_batch" => AllowAll
State: Struct(LazyMap("bc417218214859fbbf019072394c50cc53d5419f4acd7a660dc7c880f0cce31a02040000"))
Lazy Map: 020d3869346218a5e8deaaf2001216dc00fcacb79fb43e30ded79a(bc417218214859fbbf019072394c50cc53d5419f4acd7a660dc7c880f0cce31a, 1026)
├─ ResourceAddress("030000000000000000000000000000000000000000000000000004") => Vault("bc417218214859fbbf019072394c50cc53d5419f4acd7a660dc7c880f0cce31a03040000")
├─ ResourceAddress("03687672452e3cca5d202e36a587bb5b3f5cf471c873964bc84486") => Vault("f15da727a2b1e762ae529b050bffc06b44d2a0f78d5384b3fdedd8918a57328605040000")
├─ ResourceAddress("03aaea37ea01de894e47d28ded9398ac3c2c662270f38e1c44c2c4") => Vault("6aa932923235e3105c58ba293da6f40daef80e7c1f75d112c9fc4869a3d76f5c06040000")
└─ ResourceAddress("03d3a74e295e581fde00024b1339798052d5c1ffb324d497297488") => Vault("6cc2eb7f95c40e1c7ec2881eea746d3b63fcb790091d42ac93c6ce1dbe42966708040000")
Resources:
├─ { amount: 1, resource address: 03d3a74e295e581fde00024b1339798052d5c1ffb324d497297488, name: "Owner of catalog" }
├─ { amount: 999700, resource address: 030000000000000000000000000000000000000000000000000004, name: "Radix", symbol: "XRD" }
├─ { amount: 1, resource address: 03687672452e3cca5d202e36a587bb5b3f5cf471c873964bc84486, name: "Article minter" }
└─ { amount: 2, resource address: 03aaea37ea01de894e47d28ded9398ac3c2c662270f38e1c44c2c4, name: "Article Black ink 100 mL" }
   ├─ NON_FUNGIBLE { id: 00000001, immutable_data: Struct("Black ink 100 mL", Decimal("150")), mutable_data: Struct() }
   └─ NON_FUNGIBLE { id: 00000002, immutable_data: Struct("Black ink 100 mL", Decimal("150")), mutable_data: Struct() }
=========                    =========
========= BEFORE WITHDRAWING =========
======================================
======================================
========= AFTER  WITHDRAWING =========
=========      CATALOG       =========
=========                    =========
Component: 0235dd53bb575710382be3706f20562e86bd9fd191211007fde499
Blueprint: { package_address: 01fe7b134365efaae977274d150d36b6355a7592ad52423a80e9c2, blueprint_name: "Catalog" }
Authorization
├─ "register_reference" => Protected(ProofRule(Require(StaticResource(03687672452e3cca5d202e36a587bb5b3f5cf471c873964bc84486))))
├─ "add_stock_to_reference" => Protected(ProofRule(Require(StaticResource(03687672452e3cca5d202e36a587bb5b3f5cf471c873964bc84486))))
├─ "become_minter" => Protected(ProofRule(Require(StaticResource(03d3a74e295e581fde00024b1339798052d5c1ffb324d497297488))))
└─ "withdraw" => Protected(ProofRule(Require(StaticResource(03d3a74e295e581fde00024b1339798052d5c1ffb324d497297488))))
State: Struct(1u32, HashMap<U32, Struct>(1u32, Struct(1u32, 5u32, "Black ink 100 mL", Decimal("150"))), HashMap<U32, Vault>(1u32, Vault("f4131de2f69e0428b77d0abc549cc0aca280b4281fe980b0c84efa7dbe3a760802040000")), Vault("6cc2eb7f95c40e1c7ec2881eea746d3b63fcb790091d42ac93c6ce1dbe42966704040000"), Vault("6cc2eb7f95c40e1c7ec2881eea746d3b63fcb790091d42ac93c6ce1dbe42966705040000"), ResourceAddress("03d3a74e295e581fde00024b1339798052d5c1ffb324d497297488"))
Resources:
├─ { amount: 3, resource address: 03aaea37ea01de894e47d28ded9398ac3c2c662270f38e1c44c2c4, name: "Article Black ink 100 mL" }
│  ├─ NON_FUNGIBLE { id: 00000003, immutable_data: Struct("Black ink 100 mL", Decimal("150")), mutable_data: Struct() }
│  ├─ NON_FUNGIBLE { id: 00000004, immutable_data: Struct("Black ink 100 mL", Decimal("150")), mutable_data: Struct() }
│  └─ NON_FUNGIBLE { id: 00000005, immutable_data: Struct("Black ink 100 mL", Decimal("150")), mutable_data: Struct() }
├─ { amount: 0, resource address: 030000000000000000000000000000000000000000000000000004, name: "Radix", symbol: "XRD" }
└─ { amount: 1, resource address: 03687672452e3cca5d202e36a587bb5b3f5cf471c873964bc84486, name: "Article minter" }
=========                    =========
=========       OWNER        =========
=========                    =========
Component: 020d3869346218a5e8deaaf2001216dc00fcacb79fb43e30ded79a
Blueprint: { package_address: 010000000000000000000000000000000000000000000000000003, blueprint_name: "Account" }
Authorization
├─ "deposit_batch" => AllowAll
└─ "deposit" => AllowAll
State: Struct(LazyMap("bc417218214859fbbf019072394c50cc53d5419f4acd7a660dc7c880f0cce31a02040000"))
Lazy Map: 020d3869346218a5e8deaaf2001216dc00fcacb79fb43e30ded79a(bc417218214859fbbf019072394c50cc53d5419f4acd7a660dc7c880f0cce31a, 1026)
├─ ResourceAddress("030000000000000000000000000000000000000000000000000004") => Vault("bc417218214859fbbf019072394c50cc53d5419f4acd7a660dc7c880f0cce31a03040000")
├─ ResourceAddress("03687672452e3cca5d202e36a587bb5b3f5cf471c873964bc84486") => Vault("f15da727a2b1e762ae529b050bffc06b44d2a0f78d5384b3fdedd8918a57328605040000")
├─ ResourceAddress("03d3a74e295e581fde00024b1339798052d5c1ffb324d497297488") => Vault("6cc2eb7f95c40e1c7ec2881eea746d3b63fcb790091d42ac93c6ce1dbe42966708040000")
└─ ResourceAddress("03aaea37ea01de894e47d28ded9398ac3c2c662270f38e1c44c2c4") => Vault("6aa932923235e3105c58ba293da6f40daef80e7c1f75d112c9fc4869a3d76f5c06040000")
Resources:
├─ { amount: 1000000, resource address: 030000000000000000000000000000000000000000000000000004, name: "Radix", symbol: "XRD" }
├─ { amount: 2, resource address: 03aaea37ea01de894e47d28ded9398ac3c2c662270f38e1c44c2c4, name: "Article Black ink 100 mL" }
│  ├─ NON_FUNGIBLE { id: 00000001, immutable_data: Struct("Black ink 100 mL", Decimal("150")), mutable_data: Struct() }
│  └─ NON_FUNGIBLE { id: 00000002, immutable_data: Struct("Black ink 100 mL", Decimal("150")), mutable_data: Struct() }
├─ { amount: 1, resource address: 03d3a74e295e581fde00024b1339798052d5c1ffb324d497297488, name: "Owner of catalog" }
└─ { amount: 1, resource address: 03687672452e3cca5d202e36a587bb5b3f5cf471c873964bc84486, name: "Article minter" }
=========                    =========
========= AFTER  WITHDRAWING =========
======================================
```

</details>

You should then have multiple manifests in the `manifests` folder and few variables exported in your current terminal:

- `owner_account`: the owner account as retured by resim (containing all data)
- `owner`: the owner account address
- `owner_private_key`: the owner private key (if you want to change it using resim)
- `package`: the package address
- `xrd`: the XRD resource address
- `catalog`: the created catalog
- `owner_badge`: the created catalog's owner badge resource address
- `reference_minter_badge`: the reference minter's badge resource address
- `new_reference`: the ID of the created reference

From there you can easily run every manifests and enjoy creation of new references, new employees and selling your products!

> Note: Everything is currently sold in XRD, but it can easily be changed either on a per product or per reference basis.

### ABI

To see the package ABI, you can simply:

```shell
source init.sh > /dev/null && resim export-abi $package Catalog
```

<details>
<summary>Click to show current ABI</summary>

```json
{
  "package_address": "01857013be72fbded2ca3b56863f27171d5b92a8bd763b7f18329b",
  "blueprint_name": "Catalog",
  "functions": [
    {
      "name": "new",
      "inputs": [],
      "output": {
        "type": "Tuple",
        "elements": [
          {
            "type": "Custom",
            "name": "ComponentAddress",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          }
        ]
      }
    }
  ],
  "methods": [
    {
      "name": "become_minter",
      "mutability": "Mutable",
      "inputs": [],
      "output": {
        "type": "Custom",
        "name": "Bucket",
        "generics": []
      }
    },
    {
      "name": "register_reference",
      "mutability": "Mutable",
      "inputs": [
        {
          "type": "String"
        },
        {
          "type": "Custom",
          "name": "Decimal",
          "generics": []
        }
      ],
      "output": {
        "type": "U32"
      }
    },
    {
      "name": "add_stock_to_reference",
      "mutability": "Mutable",
      "inputs": [
        {
          "type": "U32"
        },
        {
          "type": "U64"
        }
      ],
      "output": {
        "type": "Custom",
        "name": "Decimal",
        "generics": []
      }
    },
    {
      "name": "purchase_article",
      "mutability": "Mutable",
      "inputs": [
        {
          "type": "U32"
        },
        {
          "type": "U64"
        },
        {
          "type": "Custom",
          "name": "Bucket",
          "generics": []
        }
      ],
      "output": {
        "type": "Tuple",
        "elements": [
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          }
        ]
      }
    },
    {
      "name": "withdraw",
      "mutability": "Mutable",
      "inputs": [],
      "output": {
        "type": "Custom",
        "name": "Bucket",
        "generics": []
      }
    }
  ]
}
```

</details>

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