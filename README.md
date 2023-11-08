# STARK-EVM adapter

[<img alt="github" src="https://img.shields.io/badge/github-zksecurity/stark-evm-adapter-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/zksecurity/stark-evm-adapter)
[<img alt="crates.io" src="https://img.shields.io/crates/v/stark-evm-adapter.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/stark-evm-adapter)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-stark-evm-adapter-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/stark-evm-adapter)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/zksecurity/stark-evm-adapter/rust.yml?branch=main&style=for-the-badge" height="20">](https://github.com/zksecurity/stark-evm-adapter/actions?query=branch%main)

This library provides TKTK

```toml
[dependencies]
stark-evm-adapter = "0.1.0"
```

## Example

```rust
use stark_evm_adapter::annotation_parser::AnnotatedProof;

// read an annotated proof
let file = std::fs::File::open("tests/fixtures/annotated_proof.json").unwrap();
let reader = std::io::BufReader::new(file);
let annotated_proof: AnnotatedProof = serde_json::from_reader(reader).unwrap();

// split the proof
let split_proofs = split_fri_merkle_statements(annotated_proof).unwrap();

let expected_split_fri_proofs = get_expected_split_proofs();
println!("{}", serde_json::to_string_pretty(&split_proofs).unwrap());
```
