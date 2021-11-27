# Options Market

## What are options?

Options are contracts that give the bearer the right—but not the obligation—to either buy or sell an amount of some underlying asset at a predetermined price at or before the contract expires.

Options can be used for speculation, income generation and portfolio protection.
A popular example would be using options to hedge against falling stock prices.

## What is the goal?

The goal is to create a decentralized options protocol for crypto assets and/or synthetics.
Created option tokens are fully collateralized by the underlying asset and can be traded on any dex that supports those.

## How it works?

An option writer locks up the underlying asset and receives two tokens.
The option token can be sold for a premium. The writer token gives the hodler the right to access the underlying asset post expiration.

An option has an expiration date and a strike price. The hodler of the option token can exercise on a trade until the option expires and swap the underlying asset with the quoted asset for the strike price.

### Example

Options pair: BTC/USDC
Quote asset: USDC
Underlying asset: BTC
Strike price: $55000
Premium: $200
Break even (strike price + premium): $55200
Expiration: 2021-12-31

1. Alice writes a call option and locks up one BTC as underlying asset.
2. Bob buys the call option from Alice for the $200 premium which gives him the right to buy the BTC from Alice for the price of $55000.
3. Two scenarios:
    1. Price of BTC goes up to $60000 until the expiration date. Bob now buys BTC with a discount of $5000 and makes a profit of $4800 considering the previously paid premium of $200.
    2. Price of BTC goes down and stays below the break even point of $55200. The option expires worthless.
