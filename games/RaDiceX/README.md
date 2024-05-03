# RaDiceX

Radicex or playing dice on the Radix network.</br>

Buy a RaDiCeX Ticket and play against the House. The game controlled Ticket (NFT) keeps track of your game progress.</br>

## Gameplay:
    A New Ticket will start at level 10.
    In a play round the you and the house will roll dice.
    The difference between player die and house die is determined
    This difference is added/subtracted from the Ticket's level. 
    This play continues until the either level=0 or level=25 is reached. 
    On level=0 the ticket can be renewed with a discount. 
    On ticket level=25 the ticket can be redeemed for a price of 5 time buy-in.
    Finished with playing, the ticket can be burned.

## Getting Started
-   Source the sourceme on Linux/Bash for an easy start.

        %-> source sourceme
-   put some XRD in the prizepool so winning tickets can be redeemed.
       
        %-> resim call-method $component deposit 100 $radix:101
-   Buy a Ticket,

        %-> resim call-method $component buy_ticket $radix:2
-   Obtain the resource of the ticket as $ticket and start playing by repeating this command multiple times.

        %-> resim call-method $component play_round $ticket:1
Note: If your account contains multiple playable tickets you can specify the ticket to use for gameplay

        %-> resim call-method $component play_round $ticket:#NFT_ID#

-   To check your Ticket status

        %-> resim show $account

-   If the ticket level=0 renew your Ticket with a discount

        %-> resim call-method $component reinit_ticket $ticket:1 $radix:10

-   If the ticket level=25 redeem your prize

        %-> resim call-method $component redeem_prize $ticket:1


-   As Admin, get yourself a free ticket.

        %-> resim call-method $component admin_ticket --proofs $proof:1

-   As Admin, get all the cash out of the prizepool.

        %-> resim call-method $component withdrawal_all --proofs $proof:1


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