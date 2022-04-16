# Transit Setup

This is a basic example of a transit component.

## Setup account 1
- Make sure you are in the correct directory: `cd transit` and clear data: `resim reset`
```
resim new-account
export pubkey1=
export acct1=
resim new-token-fixed 1000 --description "The American dollar" --name "dollar" --symbol "USD"
export usd=
resim new-token-fixed 1000 --description "The European euro" --name "euro" --symbol "EUR"
export eur=
```
## Setup account 2
```
resim new-account
export pubkey2=
export acct2=
```
## Transfer funds from account 1 to account 2
```
resim transfer 500,$usd $acct2
resim transfer 500,$eur $acct2
```
- At this point both accounts should have $500 and €500 you can verify with: `resim show $acct1`
## Publish package
```
resim publish .
export package=
```
## Call-function package
- Here you can specify CLI args: `resim call-function $package Transit new [cost per ticket] [cost per ride] $usd $eur`
```
resim call-function $package Transit new 10 1 $usd $eur
export american_badge=
export european_badge=
export mint_auth=
export ticket=
export component=
```
## Call the buy_ticket method in our component
- If you modified the cost per ticket or cost per ride above, you should adjust arguments accordingly.
```
resim call-method $component buy_ticket 10,$usd
resim call-method $component buy_ticket 10,$usd
resim call-method $component buy_ticket 10,$eur
resim call-method $component buy_ticket 10,$eur
```
- Account 1 should now have 4 tickets after spending $20 and €20
## Transfer the european badge to account 2
- Each region has 1 host badge for controlling rides and withdrawing payments
```
resim transfer 1,$european_badge $acct2
```
## Switch to account 2
- account 2 should now have the european badge, $500 and €500: `resim show $acct2`
```
resim set-default-account $acct2 $pubkey2
```
## Buy more tickets but now with account 2
```
resim call-method $component buy_ticket 10,$usd
resim call-method $component buy_ticket 10,$usd
resim call-method $component buy_ticket 10,$eur
resim call-method $component buy_ticket 10,$eur
```
- Account 2 should now have 4 tickets after spending $20 and €20: `resim show $acct2`
## Time to go for a ride on the transit!
- In order to ride the transit you must provide a ticket you purchased earlier.
```
resim call-method $component ride 1,$ticket "European"
resim call-method $component ride 1,$ticket "American"
## Set current epoch
```
resim set-current-epoch 5
```
## Go for a another ride on the transit
```
resim call-method $component ride 1,$ticket "American"
```
## Time to shutdown the transit and collect payments
```
resim call-method $component withdraw_euros false 1,$european_badge
```
- The european rides are shutdown and we made an additional €20 today if you check: `resim show $acct2`
## Attempt to ride european again
```
resim call-method $component ride 1,$ticket "European"
```
- You should notice an error, indicating that the ride is shutdown.
## Switch back to account 1
```
resim set-default-account $acct1 $pubkey1
```
## Shutdown the transit and collect payments
```
resim call-method $component withdraw_dollars false 1,$american_badge
```
- The American rides are now shutdown and we should have earned $20 if you check: `resim show $acct1`
