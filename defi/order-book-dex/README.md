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
# Account address: 020d3869346218a5e8deaaf2001216dc00fcacb79fb43e30ded79a
# Private key: 7c9fa136d4413fa6173637e883b6998d32e1d675f88cddff9dcbcf331820f4b8
resim new-account

# Create an account for a simulated market maker
# Account address: 02e0905317d684478c275540e2ed7170f217e0c557805f7fd2a0d3
# Private key: 35be322d094f9d154a8aba4733b8497f180353bd7ae7b0a15f90b586b549f28b
resim new-account

# Create an account for a simulated taker
# Account address: 02b61acea4378e307342b2b684fc35acf0238a4accb9f91e8a4364
# Private key: f13ee6ed54ea2aae9fc49a9faeb5da6e8ddef0e12ed5d30d35a624ae813e0485
resim new-account

# Create a new token that can be traded against XRD and send it to the market taker user
# rUSD: 0346c82723645afa14855dba6592974ca2ec943c0cb965cb5f43ad
resim run transactions/create-token-rusd.rtm
resim run transactions/send-rusd-to-market-taker.rtm

# Publish the package
# Package: 0136a78b993b4ac430392eccbfdcf61407f4acba48e995dc14a57c
resim publish .

# Instantiate the Dex component
# Dex component: 0246f768fdf369942e0c7f6d6db43463df67d16a03cec713136d4b
# Admin badge: 034815cac149c68b5a7d2706105feeb7ad0b59df16cdc0c3648b03
resim call-function 0136a78b993b4ac430392eccbfdcf61407f4acba48e995dc14a57c Dex instantiate

# Create a trading pair for XRD/rUSD
# Parameters are 1) XRD address 2) rUSD address 3) admin badge bucket
# TradingPair component: 02b9af5270cc62a2c357ec93bbc231aa910443ac04d05ad16f942a
# Limit order NFR: 03a4a76be1dc1d3fc343c82d082eb8de0fb45821b807cbbeab1922
resim run add_trading_pair.rtm
```

With a test scenario set up, simulate a market maker creating three orders

```sh
# Act as the market maker
resim set-default-account 02e0905317d684478c275540e2ed7170f217e0c557805f7fd2a0d3 \
  35be322d094f9d154a8aba4733b8497f180353bd7ae7b0a15f90b586b549f28b

# Create three limit ask orders:
# - 1000 XRD @100   --> #90d8174e822df9566ef9632b9f4fed2c,03a4a76be1dc1d3fc343c82d082eb8de0fb45821b807cbbeab1922
# - 1000 XRD @200   --> #e19b92dec12d0af397999be7ae946776,03a4a76be1dc1d3fc343c82d082eb8de0fb45821b807cbbeab1922
# - 1000 XRD @1000  --> #191ce90531e2e59a2790c00930a3f501,03a4a76be1dc1d3fc343c82d082eb8de0fb45821b807cbbeab1922
resim run transactions/new-limit-orders.rtm

# Take a look at the market maker's account and observe that it now holds three NFRs representing the newly created
# limit orders. As each of the three offers is for 1000 XRD, a total of 3000 have been take out of the maker's account.
resim show 02e0905317d684478c275540e2ed7170f217e0c557805f7fd2a0d3
```

Now, simulate someone filling these limit orders

```sh
# Act as the taker
resim set-default-account 02b61acea4378e307342b2b684fc35acf0238a4accb9f91e8a4364 \
  f13ee6ed54ea2aae9fc49a9faeb5da6e8ddef0e12ed5d30d35a624ae813e0485

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
resim show 02b61acea4378e307342b2b684fc35acf0238a4accb9f91e8a4364

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
resim set-default-account 02e0905317d684478c275540e2ed7170f217e0c557805f7fd2a0d3 \
  35be322d094f9d154a8aba4733b8497f180353bd7ae7b0a15f90b586b549f28b

# First take another look at their account
# Observe that they own 997,000 XRD and they still own the three order NFRs. Looking at those NFRs, also note that
# - the first limit order (#90d8174e822df9566ef9632b9f4fed2c) has been filled completely (1000/1000)
# - the second limit order (#e19b92dec12d0af397999be7ae946776) has been filled to 50% (500/1000)
# - the third limit order (#191ce90531e2e59a2790c00930a3f501) has not been filled at all (0/1000)
resim show 02e0905317d684478c275540e2ed7170f217e0c557805f7fd2a0d3

# Simulate that the market maker closes their first limit order, which has been filled completely.
# This will redeem the 100,000 rUSD that were traded successfully.
resim run transactions/close-limit-order-1.rtm

# Show the maker's account.
# The first limit order NFR is now gone and the maker's account has received 100,000 rUSD
resim show 02e0905317d684478c275540e2ed7170f217e0c557805f7fd2a0d3

# Simulate that the market maker closes their second limit order, which has been filled to 50%.
# As it has not been filled completely, it will be canceled. This will redeem another 100,000 rUSD,
# which have been traded successfully and will also reclaim the 500 XRD that were not traded. 
resim run transactions/close-limit-order-2.rtm

# Show the maker's account.
# The second limit order NFR is also gone and the account has received another 100,000 rUSD as well as 500 XRD
resim show 02e0905317d684478c275540e2ed7170f217e0c557805f7fd2a0d3

# Simulate that the market maker closes their third limit order, which has not been filled tat all.
# This will cancel the order and reclaim the 1000 XRD that were deposited for that order. No rUSD will be received.
resim run transactions/close-limit-order-3.rtm

# Show the maker's account one last time.
# The third limit order NFR is also gone and the account has been refunded 1000 XRD.
resim show 02e0905317d684478c275540e2ed7170f217e0c557805f7fd2a0d3
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