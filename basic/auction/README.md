# How to use
1. You may start by creating a new resource which will be offered on an auction.

2. Start a new auction:
```
resim call-function <package> Auction new <offering> <duration> <payment_resource> <reserve_price> <bid_bond>
```
Where:
   * `package` - package address
   * `offering` - a bucket with what you intend to sell on the auction
   * `duration` - in what time (in epochs) the auction will end
   * `payment_resource` - in which resource you want to get the payment (e.g. XRD)
   * `reserve_price` - minimum price at which you are willing to sell the item
   * `bid_bond` - a collateral from bidders who wish to participate in the auction, if the item is sold but not paid for, the winning bidder's bid bond will be kept by the auctioneer, all other bidders can reclaim their bid bonds as soon as the auction ends

In return you will receive the auctioneer badge.

3. You may create some new accounts and switch to them to participate in the auction

4. As a bidder, register for the auction
```
resim call-method <auction> register <bid_bond>
```
Where:
   * `auction` - component address
   * `bid_bond` - a bucket with the bid bond

In return you will receive a bidder badge.

5. Place some bids:
```
resim call-method <auction> bid <amount> <bidder_badge>
```
Where:
   * `auction` - component address
   * `amount` - bid value
   * `bidder_badge` - a bucket with the bidder badge

Note that a bid must be higher than both the reserve price and the highest bid currently placed

6. Advance some epochs until the auction is closed

7. Now the winning bidder can collect the offering:
```
resim call-method <auction> claim_offering <payment> <bidder_badge>
```
Where:
   * `payment` - a bucket with the payment according to the winning bid **minus the bid bond**

8. And the other bidders can reclaim their bid bonds:
```
resim call-method <auction> reclaim_bid_bond <bidder_badge>
```

9. The auctioneer can now collect the payment:
```
resim call-method <auction> claim_payment <auctioneer_badge>
```

10. Alternatively, if the winning bidder did not submit the payment, advance some more epochs (currently 100 after an auction end) and call the method again. As an auctioneer you will now receive your offering plus the winner's bid bond.
