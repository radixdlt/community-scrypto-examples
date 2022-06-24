#!/usr/bin/env bash

#set -x
set -e

# Use init

logy "Citizen no.${ID1} request demolish a building in barren land."

resim set-default-account $USER1_ACC $USER1_PIV
export R_ACC=$USER1_ACC

logr "This should panic!"
resim run ./transaction_manifest/request_demolish_fail || true

logc "Every citizen request construct a building."

export building_size="100"
export building_floor="2"
logy "Citizen no.${ID1} request ${building_size}m2, ${building_floor} floor building."
resim run ./transaction_manifest/request_construct 

resim set-default-account $USER2_ACC $USER2_PIV

export building_size="150"
export building_floor="3"
export R_ACC=$USER2_ACC
logy "Citizen no.${ID2} request ${building_size}m2, ${building_floor} floor building."
resim run ./transaction_manifest/request_construct 

resim set-default-account $USER3_ACC $USER3_PIV

export building_size="210"
export building_floor="4"
export R_ACC=$USER3_ACC
logy "Citizen no.${ID3} request ${building_size}m2, ${building_floor} floor building."
resim run ./transaction_manifest/request_construct 

resim set-default-account $USER4_ACC $USER4_PIV

export building_size="500"
export building_floor="10"
export R_ACC=$USER4_ACC
logy "Citizen no.${ID4} request ${building_size}m2, ${building_floor} floor building."
resim run ./transaction_manifest/request_construct 

resim set-default-account $USER5_ACC $USER5_PIV

export building_size="1000"
export building_floor="70"
export R_ACC=$USER5_ACC
logy "Citizen no.${ID5} request ${building_size}m2, ${building_floor} floor building."
resim run ./transaction_manifest/request_construct 

logc "Construction institute review citizen's request."
logy "Construction institute allow all citizen's request except citizen no.${ID3}"
logy "Citizen no.${ID3}'s request is rejected because the construction will cause harm to society's environment."
resim set-default-account $CAUTH_ACC $CAUTH_PIV
resim run ./transaction_manifest/review_construction 

logc "Let citizens get their construction rights and build their building."
resim set-default-account $USER1_ACC $USER1_PIV
export R_ACC=$USER1_ACC
resim run ./transaction_manifest/construct

resim set-default-account $USER2_ACC $USER2_PIV
export R_ACC=$USER2_ACC
resim run ./transaction_manifest/construct 

resim set-default-account $USER3_ACC $USER3_PIV
export R_ACC=$USER3_ACC
logr "This should panic since citizen no.${ID3}'s request already rejected!"
resim run ./transaction_manifest/construct || true

resim set-default-account $USER4_ACC $USER4_PIV
export R_ACC=$USER4_ACC
resim run ./transaction_manifest/construct 

resim set-default-account $USER5_ACC $USER5_PIV
export R_ACC=$USER5_ACC
resim run ./transaction_manifest/construct 

logc "Let citizen no.${ID1} demolish his building"

resim set-default-account $USER1_ACC $USER1_PIV
export R_ACC=$USER1_ACC
resim run ./transaction_manifest/request_demolish 

resim set-default-account $CAUTH_ACC $CAUTH_PIV
resim run ./transaction_manifest/review_demolish

resim set-default-account $USER1_ACC $USER1_PIV
export R_ACC=$USER1_ACC
resim run ./transaction_manifest/demolish 

resim set-default-account $USER2_ACC $USER2_PIV
export R_ACC=$USER2_ACC
logc "Citizen no.${ID2} request construct a building on a land that already has building"
logr "This should panic!"
resim run ./transaction_manifest/request_construct_fail || true

completed