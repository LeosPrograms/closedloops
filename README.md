# mtcs-rs

[![Build Status][build-image]][build-link]
[![Docs][docs-image]][docs-link]
[![Apache 2.0 Licensed][license-image]][license-link]
![Rust Stable][rustc-image]
![Rust 1.60+][rustc-version]

Rust implementations of algorithms and tools for Multilateral Trade Credit Set-off (MTCS).

This repository hosts the `mtcs` rust crate which provides -

* A library containing implementations of algorithms used for MTCS.
* A CLI tool that runs MTCS on a specified input CSV file (containing a list of obligations) and outputs the resulting set-off notices as a CSV file.

## CLI usage

```shell
$ cargo run -- --help
Tool for running Multilateral Trade Credit Set-off (MTCS) on an obligation network

Usage: mtcs-cli [OPTIONS] --input-file <INPUT_FILE> --output-file <OUTPUT_FILE>

Options:
  -i, --input-file <INPUT_FILE>    Path to input CSV file with obligations (fields - `id` (optional), `debtor`, `creditor`, `amount`)
  -o, --output-file <OUTPUT_FILE>  Path to output CSV file
  -v, --verbose...                 Log level
  -h, --help                       Print help information
  -V, --version                    Print version information
```

The input is expected to be a CSV file containing a list of obligations with the following header fields - `id` (optional), `debtor`, `creditor` & `amount`. For example -

```shell
$ cat data/micro.csv
id,debtor,creditor,amount
1,10,20,100
2,20,30,100
3,30,10,200
4,40,30,100
```

The output is a CSV file containing a list of set-offs with the following header fields - `id` & `amount`. For every obligation we state the setoff amount. For
example -

```shell
$ cargo run -- --input-file data/micro.csv --output-file micro-set-offs.csv
$ cat micro-set-offs.csv
debtor,creditor,amount,setoff,remainder
10,20,0,100,0
20,30,0,100,0
30,10,200,100,100
```

## Contributing

If you're interested in contributing, please comment on a relevant issue (if there is one) or open a new one! See [CONTRIBUTING.md](./CONTRIBUTING.md)

## Resources

* [Liquidity-Saving through Obligation-Clearing and Mutual Credit: An Effective Monetary Innovation for SMEs in Times of Crisis](https://www.mdpi.com/1911-8074/13/12/295)
* [Mathematical Foundations for Balancing the Payment System in the Trade Credit Market](https://eprints.lse.ac.uk/112151/1/jrfm_14_00452_v5_1_.pdf)
* [Prof. David Karger's lectures on max-flow algorithms at MIT 6.5210](https://6.5210.csail.mit.edu/materials.html)
* [WilliamFiset's awesome network flow playlist on YouTube](https://www.youtube.com/playlist?list=PLDV1Zeh2NRsDj3NzHbbFIC58etjZhiGcG)

## License

Copyright © 2023 Informal Systems Inc. and mtcs-rs authors.

Licensed under the Apache License, Version 2.0 (the "License"); you may not use the files in this repository except in compliance with the License. You may
obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.


[//]: # (badges)

[docs-image]: https://docs.rs/mtcs/badge.svg

[docs-link]: https://docs.rs/mtcs/

[build-image]: https://github.com/informalsystems/mtcs/workflows/Rust/badge.svg

[build-link]: https://github.com/informalsystems/mtcs/actions?query=workflow%3ARust

[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg

[license-link]: https://github.com/informalsystems/mtcs/blob/main/LICENSE

[rustc-image]: https://img.shields.io/badge/rustc-stable-blue.svg

[rustc-version]: https://img.shields.io/badge/rustc-1.60+-blue.svg
