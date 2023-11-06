extern crate stark_evm_adapter;

use stark_evm_adapter::annotation_parser::split_fri_merkle_statements;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{self, Value};
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_split_fri_merkle_statements() {
        let mut file = File::open("tests/fixtures/annotated_proof.json").expect("unable to open input file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("unable to read file");

        let input_json: Value =
            serde_json::from_str(&contents).expect("unable to parse input JSON");

        let (main_proof, merkle_statements, fri_merkle_statements) =
            split_fri_merkle_statements(input_json).unwrap();

        let mut file =
            File::open("tests/fixtures/expected_split_proofs.json").expect("unable to open output file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("unable to read file");

        let expected_split_fri_proofs_value: Value =
            serde_json::from_str(&contents).expect("unable to parse output JSON");

        let keys: Vec<_> = vec![
            "expected_root",
            "evaluation_point",
            "fri_step_size",
            "input_layer_queries",
            "output_layer_queries",
            "input_layer_values",
            "output_layer_values",
            "input_layer_inverses",
            "output_layer_inverses",
            "input_interleaved",
            "output_interleaved",
            "proof",
        ];

        for (index, obj) in fri_merkle_statements.iter().enumerate() {
            for key in &keys {
                assert_eq!(&obj[*key], &expected_split_fri_proofs_value["fri_merkle_statements"][index][key]);
            }
        }
    }
}
