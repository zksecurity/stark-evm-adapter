extern crate stark_evm_adapter;

use stark_evm_adapter::annotation_parser::split_fri_merkle_statements;

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::utils::hex;
    use serde_json::{self, Value};
    use stark_evm_adapter::annotation_parser::AnnotatedProof;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_split_fri_merkle_statements() {
        let file = std::fs::File::open("tests/fixtures/annotated_proof.json").unwrap();
        let reader = std::io::BufReader::new(file);
        let annotated_proof: AnnotatedProof = serde_json::from_reader(reader).unwrap();

        let (main_proof, merkle_statements, fri_merkle_statements) =
            split_fri_merkle_statements(annotated_proof).unwrap();

        let mut file = File::open("tests/fixtures/expected_split_proofs.json")
            .expect("unable to open output file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("unable to read file");

        let expected_split_fri_proofs_value: Value =
            serde_json::from_str(&contents).expect("unable to parse output JSON");

        for (_, obj) in merkle_statements.iter().enumerate() {
            let obj_json = obj.1.to_json();
            for (key, value) in obj_json.as_object().unwrap() {
                assert_eq!(
                    value,
                    &expected_split_fri_proofs_value["merkle_statements"][obj.0][key]
                );
            }
        }

        for (index, obj) in fri_merkle_statements.iter().enumerate() {
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
            for key in &keys {
                assert_eq!(
                    &obj[*key],
                    &expected_split_fri_proofs_value["fri_merkle_statements"][index][key]
                );
            }
        }

        assert_eq!(
            hex::encode(main_proof),
            expected_split_fri_proofs_value["main_proof"]
                .to_string()
                .replace('\"', "")
        );
    }
}
