# Fisherman game example

How to test:

## build component
1. `resim reset`
1. `export xrd=030000000000000000000000000000000000000000000000000004`
1. `resim publish .` -> save package id into $package
1. `resim new-account`
### Build component with specific fee
1. `resim call-function $package Fisherman new 2 -> save into $component and save admin badge into $admin 

## Create players 
resim new-account -> save into $player1
resim new-account -> save into $player2
resim new-account -> save into $player3

## Initiate new game
Add some price per game
1. `resim call-method $component new_game 5 1,$admin`

## Play
### Player 1
1. `resim call-method $component capture $player1 5 2,$xrd`
### Player 2
1. `resim call-method $component capture $player2 1 2,$xrd`
### Player 3
1. r`esim call-method $component capture $player3 3 2,$xrd`

## Finish the game, calculate results and define winners
1. `resim call-method $component finish 1,$admin`       

 ## Admin Supporting Methods
 Withdraw some free assets 
 1. `resim call-method $component withdraw 4 1,$admin`