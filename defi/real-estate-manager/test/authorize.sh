#!/usr/bin/env bash

#set -x
set -e

# Use init
source ./init.sh

logc "Authorize 4 users to become citizens and distribute ID badges."
logy "User 5 are not citizen."

export ID1=`0536u64`
logy "User 1 is citizen no.${ID1}"

export ID2=`4571u64`
logy "User 2 is citizen no.${ID2}"

export ID3=`2245u64`
logy "User 3 is citizen no.${ID3}"

export ID4=`5578u64`
logy "User 4 is citizen no.${ID4}"

logc "Authorize a construction institute and a market place."

export Cname="Peter"
export Cfee=2
logy "New construction institute has named ${Cname} and has ${Cfee}% fee"

export Mname="Bob"
export Mfee=3
logy "New market place has named ${Mname} and has ${Mfee}% fee"

resim run ./transaction_manifest/authorize || false

output=`resim run ./transaction_manifest/authorize | awk '/Component: |Resource: / {print $NF}'`

completed

