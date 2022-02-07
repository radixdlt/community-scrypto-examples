# Virtual XRD

## About
Radix Scrypto blueprint for creating Virtual XRD (Think eXRD)

This blueprint allows you to:
- Swap XRD for newly minted VXRD tokens
- Swap VXRD for XRD (this burns the VXRD)

## Testing
- Setup and build
  - `source setup_and_build.sh`
- Swap XRD for VXRD
  - `resim call-method "$component" swap_xrd_for_vxrd 10,"$xrd"`
- See account balances
  - `resim show $account`
  - You should have some VXRD now
- Get resource_def for VXRD
  - `vxrd=$(resim show "$account" | grep VXRD | cut -d " " -f6 | cut -d "," -f1)`
- Swap VXRD for XRD
  - `resim call-method "$component" swap_vxrd_for_xrd 10,"$vxrd"`
- See account balances
  - `resim show $account`
  - Your VXRD should have been converted back to XRD
