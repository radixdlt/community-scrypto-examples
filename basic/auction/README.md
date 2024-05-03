# How to use
1. You may start by creating a new resource which will be offered on an auction.

2. Start a new auction:
```sh
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
```sh
resim call-method <auction> register <bid_bond>
```
Where:
   * `auction` - component address
   * `bid_bond` - a bucket with the bid bond

In return you will receive a bidder badge.

5. Place some bids:
```sh
resim call-method <auction> bid <amount> <bidder_badge>
```
Where:
   * `auction` - component address
   * `amount` - bid value
   * `bidder_badge` - a bucket with the bidder badge

Note that a bid must be higher than both the reserve price and the highest bid currently placed

6. Advance some epochs until the auction is closed

7. Now the winning bidder can collect the offering:
```sh
resim call-method <auction> claim_offering <payment> <bidder_badge>
```
Where:
   * `payment` - a bucket with the payment according to the winning bid **minus the bid bond**

8. And the other bidders can reclaim their bid bonds:
```sh
resim call-method <auction> reclaim_bid_bond <bidder_badge>
```

9. The auctioneer can now collect the payment:

The `claim_payment` method is an authenticated method which requires that the `auctioneer_badge` is present in the auth-zone for the call to be authorized. To be able to pass this badge to the auth-zone, you need to build a transaction manifest for that. The following is a simple transaction manifest for that. Before running the following commands, be sure to replace the `<placeholders>` with their appropriate values.

```sh
resim call-method <auction> claim_payment <auctioneer_badge>

echo "CALL_METHOD ComponentAddress(\"<account_address>\") \"create_proof\" ResourceAddress(\"<auctioneer_badge>\");" > ./claim_payment.rtm
echo "CALL_METHOD ComponentAddress(\"<auction>\") \"claim_payment\";" >> ./claim_payment.rtm
echo "CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"<account_address>\") \"deposit_batch\";" >> ./claim_payment.rtm
resim run ./claim_payment.rtm
```

10. Alternatively, if the winning bidder did not submit the payment, advance some more epochs (currently 100 after an auction end) and call the method again. As an auctioneer you will now receive your offering plus the winner's bid bond.


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