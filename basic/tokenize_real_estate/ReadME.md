# TokenizeRealEstate

This blueprint is meant to demonstrate the use of NFTs as a means to tokenize Real Estate Assets.

Instructions:
1. `resim reset`
2. `resim new-account` -> Save the account address somewhere
3. `resim publish .` -> Make sure to be located in the "tokenize_real_estate" directory
4. `resim call-function [package_address] TokenizeRealEstate instantiate_tokenize_real_estate` -> Save the component address somewhere

RTM Commands:
`resim run component_owner_start.rtm`
`resim run component_user_nft_mint.rtm`
`resim run component_user_update_nft_number.rtm`
`resim run component_user_update_nft_owner.rtm`
`resim run component_user_update_nft_propertype.rtm`
`resim run component_user_nft_view.rtm`
`resim run component_owner_claim_xrd.rtm`
`resim run component_user_nft_delete.rtm`

*Note: Be sure to fill out the variables in the .rtm files before running*