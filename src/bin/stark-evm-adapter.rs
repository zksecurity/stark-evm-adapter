use std::io::BufRead;

use clap::{Arg, Command};
use stark_evm_adapter::annotation_parser::{split_fri_merkle_statements, AnnotatedProof};

fn main() {
    let matches = Command::new("stark-evm-adapter")
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
        .subcommand(
            Command::new("gen-annotated-proof")
                .about("Merge stone proof and annotations into a single annotated proof json file")
                .arg(
                    Arg::new("proof-file")
                        .help("File path for proof file generated by the STARK stone-prover")
                        .long("stone-proof-file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::new("annotation-file")
                        .help("File path for annotation file generated by the STARK stone-prover")
                        .long("stone-annotation-file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::new("extra-annotation-file")
                        .help("File path for extra-annotation file generated by the STARK stone-prover")
                        .long("stone-extra-annotation-file")
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
        Some(("gen-annotated-proof", sub_matches)) => {
            let proof_filepath = sub_matches.value_of("proof-file").unwrap();
            let annotation_filepath = sub_matches.value_of("annotation-file").unwrap();
            let extra_annotation_filepath = sub_matches.value_of("extra-annotation-file").unwrap();
            let output_filepath = sub_matches.value_of("output").unwrap();

            // load proof file as json
            let proof_reader = std::fs::File::open(proof_filepath).unwrap();
            let proof: serde_json::Value = serde_json::from_reader(proof_reader).unwrap();

            // load annotation file and save the lines as an array property (annotations) in the proof json file
            let annotation_reader = std::fs::File::open(annotation_filepath).unwrap();
            let annotation_lines: Vec<String> = std::io::BufReader::new(annotation_reader)
                .lines()
                .map(|line| line.unwrap())
                .collect();
            let mut proof_with_annotations = proof.clone();
            proof_with_annotations["annotations"] = serde_json::json!(annotation_lines);

            // load extra annotation file and save the lines as an array property (extra_annotations) in the proof json file
            let extra_annotation_reader = std::fs::File::open(extra_annotation_filepath).unwrap();
            let extra_annotation_lines: Vec<String> =
                std::io::BufReader::new(extra_annotation_reader)
                    .lines()
                    .map(|line| line.unwrap())
                    .collect();
            proof_with_annotations["extra_annotations"] = serde_json::json!(extra_annotation_lines);

            // format json and write to file
            let proof_with_annotations_json =
                serde_json::to_string_pretty(&proof_with_annotations).unwrap();
            std::fs::write(output_filepath, proof_with_annotations_json).unwrap();

            println!("annotated proof wrote to {}", output_filepath);
        }
        _ => unreachable!("Unhandled subcommand"),
    }
}
