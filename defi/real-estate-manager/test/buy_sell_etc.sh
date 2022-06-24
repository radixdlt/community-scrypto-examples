#!/usr/bin/env bash

#set -x
set -e

# Use init

logc "Let citizen no.${ID1} put a sell order on his land, price: 50000 XRD"
resim set-default-account $USER1_ACC $USER1_PIV
export R_ACC=$USER1_ACC
export price=50000
resim run ./transaction_manifest/sell_order_test

logc "Let citizen no.${ID4} put a sell order on his land with attached building, price: 70000 XRD"
resim set-default-account $USER4_ACC $USER4_PIV
export R_ACC=$USER4_ACC
export price=70000
resim run ./transaction_manifest/sell_order

logy "Citizen no.${ID4} cancel his order and raise the price: 100000 XRD"
resim run ./transaction_manifest/cancel_order
export price=100000
resim run ./transaction_manifest/sell_order

logy "Citizen no.${ID1} fill the order and take the real estate of the Citizen no.${ID4}"
resim set-default-account $USER1_ACC $USER1_PIV
export R_ACC=$USER1_ACC
export order_no=2
export payment=110000
resim run ./transaction_manifest/fill_order

logy "Citizen no.${ID5} fill the order and take the real estate of the Citizen no.${ID1}"
resim set-default-account $USER1_ACC $USER5_PIV
export R_ACC=$USER5_ACC
export order_no=0
export payment=55000
resim run ./transaction_manifest/fill_order

logy "Citizen no.${ID1} and citizen no.${ID4} take the payment"
resim set-default-account $USER1_ACC $USER1_PIV
export R_ACC=$USER1_ACC
resim run ./transaction_manifest/take_payment

resim set-default-account $USER4_ACC $USER4_PIV
export R_ACC=$USER4_ACC
resim run ./transaction_manifest/take_payment

logc "Govt authority and institutes collect their tax, fee"
resim set-default-account $ADMIN_ACC $ADMIN_PIV
resim run ./transaction_manifest/collect_tax

resim set-default-account $MHOST_ACC $MHOST_PIV
resim run ./transaction_manifest/collect_mfee

resim set-default-account $CAUTH_ACC $CAUTH_PIV
resim run ./transaction_manifest/collect_cfee

logc "Govt authority and institutes edit their tax, fee"
resim set-default-account $ADMIN_ACC $ADMIN_PIV
export edit=3
resim run ./transaction_manifest/edit_tax

export edit=350
resim run ./transaction_manifest/edit_rate

resim set-default-account $MHOST_ACC $MHOST_PIV
export edit=1
resim run ./transaction_manifest/edit_mfee

resim set-default-account $CAUTH_ACC $CAUTH_PIV
export edit=120
resim run ./transaction_manifest/edit_cfee

completed