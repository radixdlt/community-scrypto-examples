## Readme

This example provides a basic implementation of the native OneResourcePool with some additional features.

The primary objective of Radix native pools is to offer a straightforward method for creating a resource pool that can be utilized by multiple applications. In this example, we aim to demonstrate that although all use cases might not be covered, incorporating some extra features can extend the coverage to more common scenarios.

- The first feature involves accommodating resources that are temporarily outside the pool. This might appear unusual, but it can be beneficial in situations like lending and market-making, where resources outside the pool can be employed for a certain period, like a loan, and then reintegrated into the pool when their use is complete.

- The second feature is the ability to activate flash loans. This feature is valuable when you intend to use the pool as a provider of flash loans. Flash loans are specific to DeFi and could become quite prevalent for dApps with liquidity pools, as activating this feature could generate additional revenue for liquidity providers.

Incorporating these features was relatively straightforward. The first feature introduces an 'external_liquidity_amount' variable that indicates the amount of liquidity being utilized outside the pool, along with a couple of methods that assist in keeping this variable updated. This variable is then used in the calculation of the pool units.
The second feature utilizes the implementation of flash loans from an official example.
