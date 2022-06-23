#!/usr/bin/env bash

#set -x
set -e

# Use init
source ./init.sh

logc "Authorize 5 users to become citizens and distribute ID badges."

export ID1=3536
logy "User 1 is citizen no.${ID1}"

export ID2=4571
logy "User 2 is citizen no.${ID2}"

export ID3=2245
logy "User 3 is citizen no.${ID3}"

export ID4=5578
logy "User 4 is citizen no.${ID4}"

export ID5=9954
logy "User 5 is citizen no.${ID5}"

logc "Authorize a construction institute and a market place."

export Cname="Peter"
export Cfee=100
logy "New ${Cname}'s construction institute has ${Cfee} XRD fee rate"

export Mname="Bob"
export Mfee=3
logy "New ${Mname}'s market place has ${Mfee}% fee"

output=`resim run ./transaction_manifest/authorize | awk '/Component: |Resource: / {print $NF}'`
export CCOMP=`echo $output | cut -d " " -f1`
export MCOMP=`echo $output | cut -d " " -f2`
export CONSTRUCTION_BADGE=`echo $output | cut -d " " -f3`
export CREQUEST_BADGE=`echo $output | cut -d " " -f4`
export ORDER_BADGE=`echo $output | cut -d " " -f5`

completed

