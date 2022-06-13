#!/usr/bin/env bash

#set -x
set -e

# Use init
source ./distribute_land.sh

logc "Add a new house right's NFT attached to a existed land"

export house_size1=100
export house_floor1=3
logy "Citizen no.1 has constructed a ${house_size1}m2, ${house_floor1} floor house on ${location1} land"
resim run ./transaction_manifest/construct

logy "Modified an existed house right's NFT attached to a existed land"

export house_size2=250
export house_floor2=6
logy "Citizen no.2 has re-constructed a ${house_size2}m2, ${house_floor2} floor house on ${location2} land"
resim run ./transaction_manifest/re_construct

completed