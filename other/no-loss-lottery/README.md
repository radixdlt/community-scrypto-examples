# No-loss Lottery example

How to test:

## Prepare
### Basics
resim reset
export xrd=030000000000000000000000000000000000000000000000000004

### Main account
1. `resim new-account` -> save address and key into $admin and $admin_pub 


### Players
1. `resim new-account `-> save into $player1 and $p_pub1
1. `resim new-account` -> save into $player2 and $p_pub2
1. `resim new-account` -> save into $player3 and $p_pub3

## Build Staking component
Use specific package
1. `export staking_pkg=01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c`
1. `cd staking`
1. `resim publish . --address $staking_pkg`
1. `resim call-function $staking_pkg Staking new 1000,$xrd` -> save into $lottery


## Build Lottery component
1. `cd ../caller`
1. `resim publish .` -> save into $lottery_pkg
1. `resim call-function $lottery_pkg Lottery new $staking 100` -> save Lottery component into $lottery, save admin def into $admin_badge, lottery token def into $lottery_token

## Start new lottery
Specify lottery name and ending epoch
1. `resim call-method $lottery start_lottery "Lottery#1" 100 1,$admin_badge`

## Buy tickets for each user
1. `resim set-default-account $player1 $pub1`
1. `resim call-method $lottery buy_ticket $player1 100,$xrd` 

1. `resim set-default-account $player2 $pub2`
1. `resim call-method $lottery buy_ticket $player2 100,$xrd  ` 

1. `resim set-default-account $player3 $pub3`
1. `resim call-method $lottery buy_ticket $player3 100,$xrd  `


## End current lottery
1. `resim set-current-epoch 101`
1. `resim set-default-account $admin $admin_pub `
1. `resim call-method $lottery end_lottery 1,$admin_badge`

## Withdraw staking and check rewards
1. `resim set-default-account $player1 $pub1`
1. `resim call-method $lottery withdraw "Lottery#1" $player1`
1. `resim call-method $lottery request_reward "Lottery#1" $player1 1,$lottery_token`

1. `resim set-default-account $player2 $pub2`
1. `resim call-method $lottery withdraw "Lottery#1" $player2`
1. `resim call-method $lottery request_reward "Lottery#1" $player2 1,$lottery_token`

1. `resim set-default-account $player3 $pub3`
1. `resim call-method $lottery withdraw "Lottery#1" $player3`
1. `resim call-method $lottery request_reward "Lottery#1" $player3 1,$lottery_token`
