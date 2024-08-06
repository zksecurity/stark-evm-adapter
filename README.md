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
use stark_evm_adapter::annotated_proof::AnnotatedProof;
use stark_evm_adapter::annotation_parser::split_fri_merkle_statements;

// read an annotated proof
let file = std::fs::File::open("tests/fixtures/annotated_proof.json").unwrap();
let reader = std::io::BufReader::new(file);
let annotated_proof: AnnotatedProof = serde_json::from_reader(reader).unwrap();

// split the proof
let split_proofs = split_fri_merkle_statements(annotated_proof).unwrap();
println!("{}", serde_json::to_string_pretty(&split_proofs).unwrap());

// For how to submit the split proofs to the L1 EVM verifier, please refer to the demo: https://github.com/zksecurity/stark-evm-adapter/blob/8af44a0aa61c89e36a08261320f234709e99ed71/examples/verify_stone_proof.rs#L18
```

Note that the annotated proof file, `annotated_proof.json`, can be generated using this CLI tool.

## CLI

### Installation

```bash
cargo install stark_evm_adapter
```

### Usage

```bash
stark_evm_adapter --help
```

To generate an annotated proof based on the outputs of the stone-prover:

```bash
stark_evm_adapter gen-annotated-proof \
    --stone-proof-file tests/fixtures/stone_proof.json \
    --stone-annotation-file tests/fixtures/stone_proof_annotation.txt \
    --stone-extra-annotation-file tests/fixtures/stone_proof_annotation_extra.txt \
    --output annotated_proof.json
```

The annotated proof originates from 3 file outputs of the [stone-prover](https://github.com/starkware-libs/stone-prover/tree/00b274b55c82077184be4c0758f7bed18950eaba#creating-and-verifying-a-proof-of-a-cairozero-program).

Once you have this annotated proof, you can use it to generate the split proofs and submit them to the L1 EVM verifier. Please refer to the [example demo](https://github.com/zksecurity/stark-evm-adapter/blob/8af44a0aa61c89e36a08261320f234709e99ed71/examples/verify_stone_proof.rs#L18)

_stone_proof.json_ comes from the _cpu_air_prover_ command, while the annotation files come from the _cpu_air_verifier_ command with arguments _annotation_file_ and _extra_output_file_.

## Demo

You can run the demo to split the proof and submit it to the Ethereum mainnet verifier. The [existing proof](./examples/bootloader/fib_annotated_proof.json) contains an internal proof that the 10th Fibonacci number is 144.

### Using existing proof

First, install Anvil using [Foundry](https://book.getfoundry.sh/getting-started/installation)

Then, run the following command:

```bash
FORK_URL=<ETHEREUM-MAINNET-RPC> \
    ANNOTATED_PROOF=./examples/bootloader/fib_annotated_proof.json \
    cargo run --example verify_stone_proof
```

### Generate new proof

You can create a new proof using Docker

#### Prerequisites

- Copy `objects.py` and `utils.py` for bootloader as `hidden/bootloader-objects.py` and `hidden/bootloader-utils.py`
- Copy `objects.py` and `utils.py` for simple bootloader as `hidden/simple-bootloader-objects.py` and `hidden/simple-bootloader-utils.py`
- Copy `cpu_air_prover` and `cpu_air_verifier` binaries generated from [stone-prover](https://github.com/starkware-libs/stone-prover) into the `./examples/bootloader/stone-prover` directory (Can also use the binaries from this [release](https://github.com/zksecurity/stark-evm-adapter/releases/tag/v0.1.0-alpha))

#### Customize program that is being proven

- Replace `TASK_PROGRAM_INPUT_PATH` and `TASK_PROGRAM_COMPILED_PATH` variables in `test_bootloader_fib.py` with your own program.

#### Run

First, build the docker image:

```bash
docker build -t stark-evm-adapter .
```

Then, copy over the annotated proof from the docker container:

```bash
container_id=$(docker create stark-evm-adapter)

docker cp -L ${container_id}:/opt/app/examples/bootloader/gen/annotated_proof.json ./annotated_proof.json
```

Finally, run the demo script:

```bash
docker run -it -e FORK_URL=<ETHEREUM-MAINNET-RPC> -e ANNOTATED_PROOF=./examples/bootloader/gen/annotated_proof.json stark-evm-adapter
```

### Note

- Alternatively, you can use `URL` instead of `FORK_URL` env to submit transactions on-chain instead of running them on a fork.
- This example verifies proofs on [`0xd51a3d50d4d2f99a345a66971e650eea064dd8df`](https://etherscan.io/address/0xd51a3d50d4d2f99a345a66971e650eea064dd8df), which is the previous version of the verifier on Ethereum. The most recent version is [`0x9fb7F48dCB26b7bFA4e580b2dEFf637B13751942`](https://etherscan.io/address/0x9fb7F48dCB26b7bFA4e580b2dEFf637B13751942), and we are working to update this example to use the most recent version.
