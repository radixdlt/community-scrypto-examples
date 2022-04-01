# Order book DEX

This package is an implementation of a simple order book based DEX. Users can create limit as well as market orders.
Market orders are filled immediately and do not require any trust. Limit orders lock a user's funds in the component and
thus require some trust in the component's code.

The package contains two blueprints, the `Dex` blueprint and the `TradingPair` blueprint.

- A `TradingPair` blueprint/component is instantiated for every pair of resources that can be traded against each other,
  e.g. XRD/rUSD or rBTC/rETH. The first resource in the trading pair is referred to as the base resource while the
  second one is referred to as the quote resource. The `TradingPair`component provides the actual methods for creating
  limit and market orders for the given trading pair.
- The `Dex` component is the central component of the system. It is used to manage trading pairs and allows users to
  discover what trading pairs are available.

Note that for the sake of simplicity fees have not been implemented for this DEX. This is left for future work.

## Creating limit orders

Limit order can be created by calling the `new_limit_order` on a `TradingPair` component. This method requires two
parameters:

- funds: Bucket - The funds the user wants to trade out. The side of the order (ask or bid) is determined by the type of
  resource held in this bucket. If it contains the base resource, an ask order is inferred. Conversely, if it contains
  the quote resource, a bid order is inferred.
- price: Decimal - The price at which the user is willing to trade the resources. Note that the price specifies how much
  of the quote resource must be exchanged in order to obtain one unit of the base resource. This is always the case
  regardless of whether the user sends a bucket of the base resource or the quote resource.

The method returns a non-fungible resource (NFR) that represents the limit order. The user can use this NFR to track
their order and to query to which extent it has been filled. They also need this NFR to cancel the order and to redeem
their resources after a trade.  
A limit will never be executed directly against other limit orders. If the limit order's price is such that it would be
a market order, it will fail.

## Creating market orders

Users can create market orders by calling the `new_market_order` method on a `TradingPair` component. This method
requires on parameter:

- funds: Bucket - A bucket with the funds the user wants to exchange via this order. As for the limit order, the side of
  the order (ask or bid) is determined by the type of resource held in this bucket.

The method returns two buckets:

1. Any unspent funds that remain after executing the market order (dust).
2. The traded resource that the user received in exchange for the funds they provided.

The user is well advised to specify (via the transaction manifest) a minimum amount of resources they are willing to
accept for the funds that they are putting up. This is especially important for larger orders that may travel down the
order book multiple price levels.  
A market order will always be filled completely or not at all. A market order might fail if there is not enough
liquidity in the order book and the market order cannot be matched to enough limit orders.

## Closing limit orders

In contrast to market orders that are filled immediately, limit orders are filled asynchronously (from the makers point
of view). They might also be filled only partially or not at all. A user can choose to close a limit order at any time
by calling the `close_limit_order` method on the `TradingPair` component. This method takes one parameter:

- order_bucket: Bucket - A bucket containing the order NFR, identifying the user as the creator of the limit order and
  the legitimate owner of all funds that are associated with it.

The method returns two buckets:

1. A refund of the resources that the user initially supplied with their limit order.
    - If the order has been filled completely, this bucket will be empty.
    - If the order has been filled partially, this bucket will contain some resources the user has put up but not all.
    - If the order has not been filled at all, this bucket will contain all resources the user has put up.
2. The resources that the user has received in exchange for the resources they have put up.
    - If the order has been filled completely, this bucket will contain an amount proportional to all the funds the user
      has put up given the price they specified when creating the order.
    - If the order has been filled partially, the amount of resources in this bucket will be proportional to the amount
      to which the order has been filled, given the order price.
    - If the order has not been filled at all, this bucket will be empty.

If the order has not been filled completely, it will be canceled and removed from the order book. The order NFR is
always burned.

## Usage

Setup a test scenario

```sh
# Reset the simulator
resim reset

# Create an admin account
# Account address: 0293c502780e23621475989d707cd8128e4506362e5fed6ac0c00a
# Public key: 005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9
resim new-account

# Create an account for a simulated market maker
# Account address: 0236ca00316c8eb5ad51b0cb5e3f232cb871803a85ec3847b36bb4
# Public key: 00d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35
resim new-account

# Create an account for a simulated taker
# Account address: 02a2a79aa613da237bcda37fd79af36e09eadd195976092cb24696
# Public key: 004b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a
resim new-account

# Create a new token that can be traded against XRD and send it to the market taker user
# rUSD: 03ad8ad4fa972bca8ee488458f0c67a1dc95e91ced95e3e0e70634
resim run transactions/create-token-rusd.rtm
resim run transactions/send-rusd-to-market-maker.rtm

# Publish the package
# Package: 011773788de8e4d2947d6592605302d4820ad060ceab06eb2d4711
resim publish .

# Instantiate the Dex component
# Dex component: 02467d8a533602e8cba096a92098b42f1a3c00e764bccee4ac1b63
# Admin badge: 03aedb7960d1f87dc25138f4cd101da6c98d57323478d53c5fb951
resim call-function 011773788de8e4d2947d6592605302d4820ad060ceab06eb2d4711 Dex instantiate

# Create a trading pair for XRD/rUSD
# Parameters are 1) XRD address 2) rUSD address 3) admin badge bucket
# TradingPair component: 02b81699c913b9b5fb1887b498bca59e964f9f38c916faa585da32
# Limit order NFR: 0399831190d6da2f44d863596f4fbd8daacb9715575bb640259d8c
resim call-method 02467d8a533602e8cba096a92098b42f1a3c00e764bccee4ac1b63 add_trading_pair \
  030000000000000000000000000000000000000000000000000004 03ad8ad4fa972bca8ee488458f0c67a1dc95e91ced95e3e0e70634 \
  1,03aedb7960d1f87dc25138f4cd101da6c98d57323478d53c5fb951
```

With a test scenario set up, simulate a market maker creating three orders

```sh
# Act as the market maker
resim set-default-account 0236ca00316c8eb5ad51b0cb5e3f232cb871803a85ec3847b36bb4 \
  00d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35

# Create three limit ask orders:
# - 1000 XRD @100   --> #9579600d4528eaca55babc22aff72510,0399831190d6da2f44d863596f4fbd8daacb9715575bb640259d8c
# - 1000 XRD @200   --> #075e21bf38d517c0b0dfcb6047dd71c8,0399831190d6da2f44d863596f4fbd8daacb9715575bb640259d8c
# - 1000 XRD @1000  --> #065e65613d8cd6adfe0e342a72356ce7,0399831190d6da2f44d863596f4fbd8daacb9715575bb640259d8c
resim run transactions/new-limit-orders.rtm

# Take a look at the market maker's account and observe that it now holds three NFRs representing the newly created
# limit orders. As each of the three offers is for 1000 XRD, a total of 3000 have been take out of the maker's account.
resim show 0236ca00316c8eb5ad51b0cb5e3f232cb871803a85ec3847b36bb4
```

Now, simulate someone filling these limit orders

```sh
# Act as the taker
resim set-default-account 02a2a79aa613da237bcda37fd79af36e09eadd195976092cb24696 \
  004b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a

# Create a new market order
# The market order is for 200,000 rUSD. This means:
# - the first market order will be filled completely, exchanging 100,000 rUSD for 1000 XRD at a price of 100
# - the second market order will only be filled to 50%, because the remaining 100,000 XRD (in the market order) 
#   are exchanged for 500 XRD at a price of 200
resim run transactions/new-market-order-1.rtm

# Take a look at the taker's account and observe that 200,000 rUSD have indeed been taken out of the account and
# 1,500 XRD have been received. Please note that the user paid a much higher price than they might have
# expected. While the best limit order sat at a price of 100, the second, which was also needed to fill the market
# order, sat at a price of 200, yielding an average price of approx. 133. 
resim show 02a2a79aa613da237bcda37fd79af36e09eadd195976092cb24696

# Create a second market order for another 200,000 rUSD.
# Suppose that the taker has accepted the price of 200 and is even willing to pay a bit (but not much) more than that.
# Their limit will be 250. Note the assertion in the transaction manifest, ensuring that at least 800 XRD are received
# in exchange for the 200,000 rUSD (200000 rUSD / 250 = 800 XRD).
# Observe that the transaction fails because this assertion is violated:
# - The user would have filled the second limit order at a price of 200, exchanging 100,000 rUSD for 500 XRD.
# - The user would have partially filled the third limit order at a price of 1000, exchanging the remaining 100,000
#   rUSD for just 100 XRD.
# - In total the user would have received 600 XRD in exchange for their 200,000 rUSD, which is why the assertion
# (requiring 800 XRD to be received) fails.
resim run transactions/new-market-order-2.rtm
```

Finally, simulate the market maker coming back to close their orders

```sh
# Act as the market maker
resim set-default-account 0236ca00316c8eb5ad51b0cb5e3f232cb871803a85ec3847b36bb4 \
  00d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35

# First take another look at their account
# Observe that they own 997,000 XRD and they still own the three order NFRs. Looking at those NFRs, also note that
# - the first limit order (#9579600d4528eaca55babc22aff72510) has been filled completely (1000/1000)
# - the second limit order (#075e21bf38d517c0b0dfcb6047dd71c8) has been filled to 50% (500/1000)
# - the third limit order (#065e65613d8cd6adfe0e342a72356ce7) has not been filled at all (0/1000)
resim show 0236ca00316c8eb5ad51b0cb5e3f232cb871803a85ec3847b36bb4

# Simulate that the market maker closes their first limit order, which has been filled completely.
# This will redeem the 100,000 rUSD that were traded successfully.
resim run transactions/close-limit-order-1.rtm

# Show the maker's account.
# The first limit order NFR is now gone and the maker's account has received 100,000 rUSD
resim show 0236ca00316c8eb5ad51b0cb5e3f232cb871803a85ec3847b36bb4

# Simulate that the market maker closes their second limit order, which has been filled to 50%.
# As it has not been filled completely, it will be canceled. This will redeem another 100,000 rUSD,
# which have been traded successfully and will also reclaim the 500 XRD that were not traded. 
resim run transactions/close-limit-order-2.rtm

# Show the maker's account.
# The second limit order NFR is also gone and the account has received another 100,000 rUSD as well as 500 XRD
resim show 0236ca00316c8eb5ad51b0cb5e3f232cb871803a85ec3847b36bb4

# Simulate that the market maker closes their third limit order, which has not been filled tat all.
# This will cancel the order and reclaim the 1000 XRD that were deposited for that order. No rUSD will be received.
resim run transactions/close-limit-order-3.rtm

# Show the maker's account one last time.
# The third limit order NFR is also gone and the account has been refunded 1000 XRD.
resim show 0236ca00316c8eb5ad51b0cb5e3f232cb871803a85ec3847b36bb4
```
