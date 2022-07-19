#!/usr/bin/env bash

#set -x
set -e

# Use init

logc "Let citizen no.${ID1} and citizen no ${ID5} divide their lands"

resim set-default-account $USER1_ACC $USER1_PIV
export R_ACC=$USER1_ACC
export land1_size=100
export location1="Ba Dinh, Hanoi, VietNam"
export land2_size=$(expr $land_size1 - $land1_size)
export location2="Cau Giay, Hanoi, VietNam"
resim run ./transaction_manifest/request_land_divide

resim set-default-account $USER5_ACC $USER5_PIV
export R_ACC=$USER5_ACC
export land1_size=3000
export location1="Linear Scalability, Radix"
export land2_size=$(expr $land_size5 - $land1_size)
export location2="Atomic Composability, Radix"
resim run ./transaction_manifest/request_land_divide_building

resim set-default-account $ADMIN_ACC $ADMIN_PIV
resim run ./transaction_manifest/review_land

resim set-default-account $USER1_ACC $USER1_PIV
export R_ACC=$USER1_ACC
resim run ./transaction_manifest/divide_land 

resim set-default-account $USER5_ACC $USER5_PIV
export R_ACC=$USER5_ACC
resim run ./transaction_manifest/divide_land_building

logc "Let citizen no.${ID5} merge his lands"
resim run ./transaction_manifest/request_merge

resim set-default-account $ADMIN_ACC $ADMIN_PIV
resim run ./transaction_manifest/review_merge

resim set-default-account $USER5_ACC $USER5_PIV
export R_ACC=$USER5_ACC
resim run ./transaction_manifest/merge_land

completed