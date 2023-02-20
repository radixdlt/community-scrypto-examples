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
       
        %-> resim call-method $component deposit 100 101,$radix
-   Buy a Ticket,

        %-> resim call-method $component buy_ticket 2,$radix
-   Obtain the resource of the ticket as $ticket and start playing by repeating this command multiple times.

        %-> resim call-method $component play_round 1,$ticket
Note: If your account contains multiple playable tickets you can specify the ticket to use for gameplay

        %-> resim call-method $component play_round $ticket:#NFT_ID#

-   To check your Ticket status

        %-> resim show $account

-   If the ticket level=0 renew your Ticket with a discount

        %-> resim call-method $component reinit_ticket 1,$ticket 10,$radix

-   If the ticket level=25 redeem your prize

        %-> resim call-method $component redeem_prize 1,$ticket


-   As Admin, get yourself a free ticket.

        %-> resim call-method $component admin_ticket --proof 1,$proof

-   As Admin, get all the cash out of the prizepool.

        %-> resim call-method $component withdrawal_all --proof 1,$proof
