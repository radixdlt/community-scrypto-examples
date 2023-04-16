# Time lock xrd
The blueprint lets you lock xrd into a component and set a timer only allow another beneficiary claim the xrd once the timer expired.


---
### To test:

`resim new-account`

`export acc1=[account address]`

`resim publish .`

`export package1=[package address]`

`resim call-function $package1 Lock instantiate_lock`

`export component1=[component address]`

`resim show $acc1`

`export admin=[Admin Badge address]`

`export xrd=resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqz8qety`

`resim call-method $component1 deposit_xrd 5,$xrd`

`resim show component1`

You can only withdraw as admin:

`resim call-method $component1 withdraw_xrd_by_amount 1 1,$admin`

You set the timer to lock the xrd for a number of epochs (ex 100):

`resim call-method $component1 update_timer 100 1,$admin`

Set the ledger to epoch 99 (to test the that the lock works):

`resim set-current-epoch 99`

Anyone can see the value of the timer:
`resim call-method $component1 get_timer`


The beneficiary badge is needed to claim the locked xrd:

`resim call-method $component1 mint_badge 1,$admin`

`resim show $acc1`

`export badge1=[beneficiary badge address]`

`resim call-method $component1 claim_xrd 1,$badge1`

Set the ledger to a epoch past the timer (ex 102):

`resim set-current-epoch 102`


Anyone with the beneficiary badge can now claim the xrd which was locked:

`resim call-method $component1 claim_xrd 1,$badge1`

Everyones happy.
