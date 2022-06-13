#!/usr/bin/env bash

#set -x
set -e

# Use init
source ./init.sh

logy "Set input parameters to Real Estate Manager component as:"

logp "tax: 5"
export tax=5
logp "medium token: XRD"

logy "You can change these parameters on ./transaction_manifest/instantiate"
logy "Check the document to study about these parameters"

output=`resim run ./transaction_manifest/instantiate | awk '/Component: |Resource: / {print $NF}'`
export COMP=`echo $output | cut -d " " -f1`
export LAND=`echo $output | cut -d " " -f3`
export HOUSE=`echo $output | cut -d " " -f4`
export AUTHORITY_BADGE=`echo $output | cut -d " " -f5`
export POSITION_BADGE=`echo $output | cut -d " " -f4`

logc "Mint 5 land with different data and the associated house, and distribute those to the citizens"

export land_size1=3000
export location1="Hanoi, VietNam"
logy "Citizen no.1 has ${land_size1}m2 land on ${location1}"

export land_size2=300
export location2="Amsterdam, Netherlands"
export house_size2=200
export house_floor2=4
logy "Citizen no.2 has ${land_size2}m2 land on ${location2} with a ${house_size2}m2, ${house_floor2} floor house"

export land_size3=2045
export location3="London, UK"
logy "Citizen no.3 has ${land_size3}m2 land on ${location3}"

export land_size4=5000
export location4="HCM City, VietNam"
export house_size4=500
export house_floor4=10
logy "Citizen no.4 has ${land_size4}m2 land on ${location4} with a ${house_size4}m2, ${house_floor4} floor house"

export land_size5=202400
export location5="Cerberus, Consensus, Radix"
export house_size5=10000
export house_floor5=500
logy "Citizen no.5 has ${land_size5}m2 land on ${location5} with a ${house_size5}m2, ${house_floor5} floor house"

resim run ./transaction_manifest/distribute_land

export LAND1_ID=`resim show $USER1_ACC | grep -oP '(?<=id: ).*?(?=,)'`

output=`resim show $USER2_ACC | grep -oP '(?<=id: ).*?(?=,)'`
export HOUSE2_ID=`echo $output | cut -d " " -f1`
export LAND2_ID=`echo $output | cut -d " " -f2`

export LAND3_ID=`resim show $USER3_ACC | grep -oP '(?<=id: ).*?(?=,)'`

output=`resim show $USER4_ACC | grep -oP '(?<=id: ).*?(?=,)'`
export HOUSE4_ID=`echo $output | cut -d " " -f1`
export LAND4_ID=`echo $output | cut -d " " -f2`

output=`resim show $USER5_ACC | grep -oP '(?<=id: ).*?(?=,)'`
export HOUSE5_ID=`echo $output | cut -d " " -f1`
export LAND5_ID=`echo $output | cut -d " " -f2`

completed