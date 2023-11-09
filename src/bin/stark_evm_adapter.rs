use clap::{Arg, Command};
use stark_evm_adapter::annotation_parser::{split_fri_merkle_statements, AnnotatedProof};

fn main() {
    let matches = Command::new("stark_evm_adapter")
        .version("0.1.0")
        .author("zksecurity <hi@zksecurity.xyz>")
        .about("EVM adapter for the STARK stone-prover")
        .subcommand(
            Command::new("split-proof")
                .about("Split an annotated proof into multiple FRI proofs")
                .arg(
                    Arg::new("annotated-proof-file")
                        .help("File path for annotated proof json file")
                        .long("annotated-proof-file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .help("File path for generated split proofs json file")
                        .long("output")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("split-proof", sub_matches)) => {
            let annotated_proof_filepath = sub_matches.value_of("annotated-proof-file").unwrap();
            let output_filepath = sub_matches.value_of("output").unwrap();

            // load annotated proof from file
            let reader = std::fs::File::open(annotated_proof_filepath).unwrap();

            // parse as annotated proof
            let annotated_proof: AnnotatedProof = serde_json::from_reader(reader).unwrap();

            // create the split proofs
            let split_proofs = split_fri_merkle_statements(annotated_proof).unwrap();

            // format json and write to file
            let split_proof_json = serde_json::to_string_pretty(&split_proofs).unwrap();
            std::fs::write(output_filepath, split_proof_json).unwrap();

            println!("split proof wrote to {}", output_filepath);
        }
        _ => unreachable!("Unhandled subcommand"),
    }
}
