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
