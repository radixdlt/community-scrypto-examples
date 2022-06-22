#!/usr/bin/env bash

#set -x
set -e

#Use log
source ./log.sh

logc "New fresh start"
resim reset

export XRD=030000000000000000000000000000000000000000000000000004

logc "Creating Authority account."
output=`resim new-account | awk '/Account component address: |Private key: / {print $NF}'`
export ADMIN_ACC=`echo $output | cut -d " " -f1`
export ADMIN_PIV=`echo $output | cut -d " " -f2`

logc "Creating Market Host account."
output=`resim new-account | awk '/Account component address: |Private key: / {print $NF}'`
export MHOST_ACC=`echo $output | cut -d " " -f1`
export MHOST_PIV=`echo $output | cut -d " " -f2`

logc "Creating Construction Authority account."
output=`resim new-account | awk '/Account component address: |Private key: / {print $NF}'`
export CAUTH_ACC=`echo $output | cut -d " " -f1`
export CAUTH_PIV=`echo $output | cut -d " " -f2`

logc "Creating 5 citizens account."
output=`resim new-account | awk '/Account component address: |Private key: / {print $NF}'`
export USER1_ACC=`echo $output | cut -d " " -f1`
export USER1_PIV=`echo $output | cut -d " " -f2`

output=`resim new-account | awk '/Account component address: |Private key: / {print $NF}'`
export USER2_ACC=`echo $output | cut -d " " -f1`
export USER2_PIV=`echo $output | cut -d " " -f2`

output=`resim new-account | awk '/Account component address: |Private key: / {print $NF}'`
export USER3_ACC=`echo $output | cut -d " " -f1`
export USER3_PIV=`echo $output | cut -d " " -f2`

output=`resim new-account | awk '/Account component address: |Private key: / {print $NF}'`
export USER4_ACC=`echo $output | cut -d " " -f1`
export USER4_PIV=`echo $output | cut -d " " -f2`

output=`resim new-account | awk '/Account component address: |Private key: / {print $NF}'`
export USER5_ACC=`echo $output | cut -d " " -f1`
export USER5_PIV=`echo $output | cut -d " " -f2`

logc "Publish Real Estate Manager, this will take a bit at first"
export package=`resim publish ../. | awk '/Package:/ {print $NF}'`

logy "Since this prototype will highly depend on epoch progress, to elaborate the test, let's choose a random epoch from 0-500"
RANGE=500

epoch=$RANDOM
let "epoch %= $RANGE"

logc "Set random current epoch in range 500:" $epoch
resim set-current-epoch $epoch

logy "Set input parameters to Real Estate Service component as:"

logp "name: NeverLand"
export name="NeverLand"
logp "tax: 5"
export tax=5
logp "rate: 100"
export rate=100
logp "medium token: XRD (just for testing)"

logy "You can change these parameters on ./transaction_manifest/instantiate"
logy "Check the document to study about these parameters"

output=`resim run ./transaction_manifest/instantiate | awk '/Component: |Resource: / {print $NF}'`
export COMP=`echo $output | cut -d " " -f1`
export ID_BADGE=`echo $output | cut -d " " -f3`
export LAND=`echo $output | cut -d " " -f7`
export BUILDING=`echo $output | cut -d " " -f8`
export AUTHORITY_BADGE=`echo $output | cut -d " " -f9`
export MODIFY_BADGE=`echo $output | cut -d " " -f10`
export MREQUEST_BADGE=`echo $output | cut -d " " -f11`
export CBADGE=`echo $output | cut -d " " -f13`
export MBADGE=`echo $output | cut -d " " -f12`

completed
