#!/usr/bin/env bash

#set -x
set -e

# Use init
source ./authorize.sh

logc "Mint 4 land with different data and distribute those to the citizens."
logy "The location can be address ID instead for more privacy."

export land_size1="3000"
export location1="Hanoi, VietNam"
export ok=true
logy "Citizen no.1 has ${land_size1}m2 land on ${location1}"

logy "Make an overlapped address."
logr "This should panic."
export land_size2=300
export location2="Hanoi, VietNam"
export not_ok=false
resim run ./transaction_manifest/distribute_land_fail || true

logy "Change the address and try again."
export location2="Amsterdam, Netherlands"
logy "Citizen no.2 has ${land_size2}m2 land on ${location2}"

export land_size3=2045
export location3="London, UK"
logy "Citizen no.3 has ${land_size3}m2 land on ${location3}"

export land_size4=5000
export location4="HCM City, VietNam"
logy "Citizen no.4 has ${land_size4}m2 land on ${location4}"

export land_size5=10000
export location5="Cerberus, Xi'an, Radix"
logy "Citizen no.5 has ${land_size5}m2 land on ${location5}"

resim run ./transaction_manifest/distribute_land

export LAND1_ID=`resim show $USER1_ACC | grep -oP '(?<=id: ).*?(?=,)'`

export LAND2_ID=`resim show $USER2_ACC | grep -oP '(?<=id: ).*?(?=,)'`

export LAND3_ID=`resim show $USER3_ACC | grep -oP '(?<=id: ).*?(?=,)'`

export LAND4_ID=`resim show $USER4_ACC | grep -oP '(?<=id: ).*?(?=,)'`

export LAND5_ID=`resim show $USER5_ACC | grep -oP '(?<=id: ).*?(?=,)'`

completed