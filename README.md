# STARK-EVM adapter

[<img alt="github" src="https://img.shields.io/badge/github-zksecurity/stark_evm_adapter-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/zksecurity/stark-evm-adapter)
[<img alt="crates.io" src="https://img.shields.io/crates/v/stark-evm-adapter.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/stark-evm-adapter)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-stark_evm_adapter-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/stark-evm-adapter)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/zksecurity/stark-evm-adapter/rust.yml?branch=main&style=for-the-badge" height="20">](https://github.com/zksecurity/stark-evm-adapter/actions?query=branch%main)

stark-evm-adapter is a library that provides a set of utilities to parse and manipulate the output of the STARK [stone proof](https://github.com/starkware-libs/stone-prover).
Specifically, the library can be used to generate a "split proof", which is necessary for proofs to be verified on Ethereum.

```toml
[dependencies]
stark-evm-adapter = "0.1.3"
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
println!("{}", serde_json::to_string_pretty(&split_proofs).unwrap());
```

## CLI demo
### Installation
```bash
cargo install stark_evm_adapter
```

### Usage
```bash
stark-evm-adapter --help
```

To generate an annotated proof based on the outputs of the stone-prover:
```bash
stark-evm-adapter gen-annotated-proof \
    --stone-proof-file tests/fixtures/stone_proof.json \
    --stone-annotation-file tests/fixtures/stone_proof_annotation.txt \
    --stone-extra-annotation-file tests/fixtures/stone_proof_annotation_extra.txt \
    --output annotated_proof.json
```

The annotated proof originates from 3 file outputs of the [stone-prover](https://github.com/starkware-libs/stone-prover/tree/00b274b55c82077184be4c0758f7bed18950eaba#creating-and-verifying-a-proof-of-a-cairozero-program). 


_stone_proof.json_ comes from the _cpu_air_prover_ command, while the annotation files come from the _cpu_air_verifier_ command with arguments _annotation_file_ and _extra_output_file_.

Once the _annotated_proof.json_ is generated, this command will transform it to split proofs which then can be used to submit to L1 EVM verifiers:
```bash
stark-evm-adapter split-proof \
    --annotated-proof-file ./annotated_proof.json \
    --output split_proofs.json
```