# DocumentIndexer - An append-only database for document versioning

DocumentIndexer offers a non-falsifiable record of changes to digital
documents. It can be used to help auditing, to establish evidence to
be used to prove ownership etc. at a later date, or just to help
people navigate the different versions of your documents / art /
source code / etc.

## How to build the blueprint
Make sure you have the necessary toolchain installed, see
[here](https://docs-babylon.radixdlt.com/main/getting-started-developers/getting-started-developers.html)
for details. You will need Scrypto 0.10.0.
- From the command line, in the `document_indexer` directory, run `scrypto build`

### How to run the test suite
- From the command line, in the `document_indexer` directory, run `scrypto test`

The test suite includes transaction manifest building for the
blueprint's public API.

### How to generate the documentation
- From the command line, in the `document_indexer` directory, run `cargo doc`

The generated web pages contain detailed documentation on how the
blueprint works.

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