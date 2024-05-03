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
  - `vxrd=$(resim show "$account" | grep VXRD | cut -d " " -f7 | cut -d "," -f1)`
- Swap VXRD for XRD
  - `resim call-method "$component" swap_vxrd_for_xrd 10,"$vxrd"`
- See account balances
  - `resim show $account`
  - Your VXRD should have been converted back to XRD


# License

The Radix Community Scrypto Examples code is released under Radix Modified MIT License.

    Copyright 2024 Radix Publishing Ltd

    Permission is hereby granted, free of charge, to any person obtaining a copy of
    this software and associated documentation files (the "Software"), to deal in
    the Software for non-production informational and educational purposes without
    restriction, including without limitation the rights to use, copy, modify,
    merge, publish, distribute, sublicense, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    This notice shall be included in all copies or substantial portions of the
    Software.

    THE SOFTWARE HAS BEEN CREATED AND IS PROVIDED FOR NON-PRODUCTION, INFORMATIONAL
    AND EDUCATIONAL PURPOSES ONLY.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
    FOR A PARTICULAR PURPOSE, ERROR-FREE PERFORMANCE AND NONINFRINGEMENT. IN NO
    EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES,
    COSTS OR OTHER LIABILITY OF ANY NATURE WHATSOEVER, WHETHER IN AN ACTION OF
    CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
    SOFTWARE OR THE USE, MISUSE OR OTHER DEALINGS IN THE SOFTWARE. THE AUTHORS SHALL
    OWE NO DUTY OF CARE OR FIDUCIARY DUTIES TO USERS OF THE SOFTWARE.