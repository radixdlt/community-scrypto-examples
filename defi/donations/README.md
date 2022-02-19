# Donations example

 How to test:

 ## build component
 1. `resim reset`
 1. `export xrd=030000000000000000000000000000000000000000000000000004`
 1. `resim publish .` -> save package id into $package
 1. `resim new-account` -> save address into $acc1 and public key into $pub1

 Create the Donations component with 2% fee
 1. `resim call-function $package Donations new 2` -> save component address into $component
 This will also create and return an admin badge

 ## Create policy
 Create badge with following params:
 - owner - The Address that will receive donations, 
 - identifier - any identifier that will help to track badges on other services, 
 - title, 
 - description
 - url
 - price 
 - supply - the amount of badges to be minted
 1. `resim call-method $component make_badge $acct1 "Level#1" "Level 1 donator" "Opens first level color" "Some URL" 5 100`

 ## Get available badges
 1. `resim call-method $component get_badges $acct1` -> save the badge address into $badge1

 ## Donate 
 Donator will pay for this badge according to the price
 The corresponding fee will be taken from the receiver bucket
 1. `resim call-method $component donate $acct1 $badge 5,$xrd `

 ## Admin Supporting Methods
 Withdraw some free assets 
 1. `resim call-method $component withdraw 4`
