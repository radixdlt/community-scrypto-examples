# SharedCompany 

This example is build for the demonstration of two things:

How a "SharedCompany" aka an infant DAO / a company governed by its shareholders could work. 
Show how a "Proposal" aka the decision mechanism for such a company could work.

It is only a prototype so there a features missing which would be highly relevant for a working SharedCompany.
Additionaly, security was not the focus so there are obvious attack vectors that are not covered. 

I hope this prototypes helps others to learn Scrypto and gives a rough idea how these systems could be designed. 

Cheers, Miro 

# How to test 
1. `export xrd=030000000000000000000000000000000000000000000000000004`
2. `resim publish .`
3. `resim call-function "your-package-address-from-2." SharedCompany new 2`
4. `export component="your-component-address"`
5. `resim call-method $component "method-name" methodArguments`
6. `--> If you create a proposal remember that you have to call that components address to interact with the proposal!`
7. `Testing can be  easier if the revup tool is used. Ask in the radix-discord for a link`


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