extern crate stark_evm_adapter;

use stark_evm_adapter::annotation_parser::split_fri_merkle_statements;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{self, Value};
    use stark_evm_adapter::annotation_parser::AnnotatedProof;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_split_fri_merkle_statements() {
        let file = std::fs::File::open("tests/fixtures/annotated_proof.json").unwrap();
        let reader = std::io::BufReader::new(file);
        let annotated_proof: AnnotatedProof = serde_json::from_reader(reader).unwrap();

        let split_proofs = split_fri_merkle_statements(annotated_proof).unwrap();

        let expected_split_fri_proofs = get_expected_split_proofs();
        let serialized_split_proofs = serde_json::to_value(&split_proofs).unwrap();

        
        // assert merkle_statements 
        for (trace_name, trace_merkle) in expected_split_fri_proofs["merkle_statements"].as_object().unwrap() {
            for (key, value) in trace_merkle.as_object().unwrap() {
                assert_eq!(
                    *value,
                    serialized_split_proofs["merkle_statements"][trace_name][key]
                );
            }
        }
        // assert fri_merkle_statements
        for (index, fri_merkle_statement) in expected_split_fri_proofs["fri_merkle_statements"].as_array().unwrap().iter().enumerate() {
            for (key, value) in fri_merkle_statement.as_object().unwrap() {
                assert_eq!(
                    *value,
                    serialized_split_proofs["fri_merkle_statements"][index][key]
                );
            }
        }

        // assert main_proof
        assert_eq!(
            expected_split_fri_proofs["main_proof"],
            serialized_split_proofs["main_proof"]
        );
    }

    fn get_expected_split_proofs() -> Value {
        let mut file = File::open("tests/fixtures/expected_split_proofs.json")
            .expect("unable to open output file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("unable to read file");

        let expected_split_fri_proofs_value: Value =
            serde_json::from_str(&contents).expect("unable to parse output JSON");
        expected_split_fri_proofs_value
    }
}
