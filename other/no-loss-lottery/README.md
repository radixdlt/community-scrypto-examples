# No-loss Lottery example

How to test:

## Prepare
### Basics
1. `resim reset`
1. `export xrd=030000000000000000000000000000000000000000000000000004`

### Main account
1. `resim new-account` -> save address and key into $admin and $admin_priv 


### Players
1. `resim new-account` -> save component address into $player1 and private key into $p_priv1.
1. `resim new-account` -> save component address into $player2 and private key into $p_priv2.
1. `resim new-account` -> save component address into $player3 and private key into $p_priv3.

## Build Staking component
Use specific package
1. `export staking_pkg=01c7adee40dd9a16ae290272d0e99835ad5c5e679941d3fb28e608`
1. `cd staking`
1. `resim privlish . --address $staking_pkg`
1. `resim call-function $staking_pkg Staking new 1000,$xrd` -> save into $staking, save staking token def into $staking_token 


## Build Lottery component
1. `cd ../lottery`
1. `resim privlish .` -> save into $lottery_pkg
1. `resim call-function $lottery_pkg Lottery new $staking` -> save Lottery component into $lottery, save admin def into $admin_badge, lottery ticket token def into $lottery_ticket

## Start new lottery
Specify lottery name, ending epoch and price. Also use the admin badge for the auth
1. `resim call-method $lottery start_lottery "Lottery#1" 1000 100 1,$admin_badge` -> save lottery ID (will be zero for the first generated lottery) into the $lottery_id

## Buy tickets for each user
For each user setup default account, purchase one ticket using the $lottery_id and required xrd
`resim set-default-account $player1 $p_priv1`
`resim call-method $lottery buy_ticket $lottery_id 100,$xrd` 

`resim set-default-account $player2 $p_priv2`
`resim call-method $lottery buy_ticket $lottery_id 100,$xrd` 

`resim set-default-account $player3 $p_priv3`
`resim call-method $lottery buy_ticket $lottery_id 100,$xrd`


## End current lottery
1. `resim set-default-account $admin $admin_priv`
1. `resim set-current-epoch 1001`
1. `resim call-method $lottery end_lottery $lottery_id 1,$admin_badge`

## Withdraw staking and check rewards
For each user setup default account again, withdraw staking + reward using $lottery ID and the ticket resource def as auth.

`resim set-default-account $player1 $p_priv1`
`resim call-method $lottery withdraw $lottery_id 1,$lottery_ticket`

`resim set-default-account $player2 $p_priv2`
`resim call-method $lottery withdraw $lottery_id 1,$lottery_ticket`

`resim set-default-account $player3 $p_priv3`
`resim call-method $lottery withdraw $lottery_id 1,$lottery_ticket`

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