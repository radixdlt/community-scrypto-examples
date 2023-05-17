# Access Pass
## Problem
The prevailing approach to access control in the Radix system demonstrates a lack of composable elegance in comparison to other aspects of the platform. With multiple actions potentially necessitating the implementation of the same complex AccessRule, the repetition of this process can contribute to a decrease in efficiency. As the DeFi propels innovation, the complexity of decentralized applications will inevitably surge, one can imagine the next generation of insurance or lending dApps applying increasingly complex rules thereby exacerbating the challenges associated with access control.

Currently, there is an absence of established standards to streamline composable access control across a single dApp or the entire network. Moreover, no standard blueprint exists to facilitate the construction of maintained or dedicated Access Rule components that other projects can readily employ. To the extent that composable access control may be present within the existing paradigm, it is crucial to note that components are neither easily trackable nor indexable, thus further complicating implementation and management.
## Solution
This project explores a novel modular access rules design pattern that takes advantage of Radix's _atomic composability_. Introducing Access Pass, a standard blueprint for composable access control.

Constructing a new Access Pass creates two addressable entities: the Access Pass Manager and the Access Pass Resource, along with an optional owner badge. The owner badge allows the holder to mutate the pass' AccessRule if not locked upon instantiation. The manager holds the pass resource and grants a `Proof` when the **`get_pass`** method is called, which can then be used for the rest of the transaction. The `get_pass` method is protected by the AccessRule you'd like to be composable, which can be either mutable or immutable.

Instantiating an Access Pass offers an efficient way to apply the AccessRule just once. When creating dApps, it's useful to abstract away complicated rules and replace them with a single pass. With this, you can set up a reusable mini regulatory environment that can be applied to multiple blueprints throughout your dApp.

There's potential to utilize another dApp's Access Pass or even a standalone Access Pass (perhaps maintained by a DAO, dedicated to providing high-quality regulatory environments for dApps like yours). Access Passes could rely on other Access Passes, creating more possibilities! Additionally, dividing large access rules into separate rule sets can allow you to manage efficiency more effectively. The creator of the Access Pass can set royalty fees on instantiation if desired.

These reusable AccessRule modules from the same package or blueprint can be easily cataloged, further enhancing composability.
# Methods
## "`new`"
Constructs a new Access Pass.
#### Arguments
- `AccessRule` for the `get_pass` method
- `bool` for specifying if the access pass is locked or not. If `true` an owner badge will not be created.
- `u32` for specifying the component royalty amount. Pass 0 if no royalty.
- `Option<Address>` for optional Access Manager dApp definition.
- `Option<String>` for optional Access Pass name.
- `Option<String>` for optional Access Pass description.
#### Returns
- `ComponentAddress` of new Access Pass Manager.
- `ResourceAddress` of new Pass Resource.
- `Option<Bucket>` an optional owner badge.

## "`get_pass`"
Returns a `Proof` which can be used for other components that utilize the pass.
#### Access Control
Protected by the custom rule set by the owner.
