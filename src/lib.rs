//! stark-evm-adapter is a library that provides a set of utilities to parse and manipulate the output of the STARK [stone proof](https://github.com/starkware-libs/stone-prover).
//! Specifically, the library can be used to generate a "split proof", which is necessary for proofs to be verified on Ethereum.

pub mod annotation_parser;
pub mod errors;
pub mod fri_merkle_statement;
pub mod merkle_statement;
pub mod serialization;
