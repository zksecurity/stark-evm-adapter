extern crate stark_evm_adapter;

use stark_evm_adapter::annotation_parser::split_fri_merkle_statements;

#[cfg(test)]
mod tests {
    use super::*;
    use stark_evm_adapter::annotation_parser::{AnnotatedProof, SplitProofs};

    #[test]
    fn test_split_fri_merkle_statements() {
        // get annotated proof
        let proof_file = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/annotated_proof.json"
        ));
        let annotated_proof: AnnotatedProof = serde_json::from_str(proof_file).unwrap();

        // create the split proof
        let split_proofs = split_fri_merkle_statements(annotated_proof).unwrap();

        // get expected split proof
        let split_proof_file = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/expected_split_proofs.json"
        ));
        let expected_split_fri_proofs: SplitProofs =
            serde_json::from_str(split_proof_file).unwrap();

        // compare
        assert_json_diff::assert_json_eq!(split_proofs, expected_split_fri_proofs);
    }
}
