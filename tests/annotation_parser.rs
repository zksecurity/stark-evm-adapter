extern crate stark_evm_adapter;

use stark_evm_adapter::annotation_parser::split_fri_merkle_statements;

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::U256;
    use stark_evm_adapter::annotated_proof::AnnotatedProof;
    use stark_evm_adapter::annotation_parser::SplitProofs;
    use stark_evm_adapter::fri_merkle_statement::VerifyFRICall;
    use stark_evm_adapter::merkle_statement::VerifyMerkleCall;
    use stark_evm_adapter::oods_statement::VerifyProofAndRegisterCall;

    fn get_split_proofs() -> SplitProofs {
        let proof_file = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/annotated_proof.json"
        ));
        let annotated_proof: AnnotatedProof = serde_json::from_str(proof_file).unwrap();

        // create the split proof
        let split_proofs: SplitProofs = split_fri_merkle_statements(annotated_proof).unwrap();
        split_proofs
    }

    #[test]
    fn test_trace_merkle_contract_args() {
        let split_proofs = get_split_proofs();
        let trace_merkle_contract_args = split_proofs
            .merkle_statements
            .get("Trace 0")
            .unwrap()
            .contract_function_call();
        assert_eq!(split_proofs.merkle_statements.len(), 3);

        let trace_merkle_contract_args_file = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/trace_0_contract_args.json"
        ));
        let deser_trace_merkle_contract_args: VerifyMerkleCall =
            serde_json::from_str(trace_merkle_contract_args_file).unwrap();
        assert_json_diff::assert_json_eq!(
            trace_merkle_contract_args,
            deser_trace_merkle_contract_args
        );
    }

    #[test]
    fn test_fri_merkle_contract_args() {
        let split_proofs = get_split_proofs();

        let fri_merkle_contract_args = split_proofs
            .fri_merkle_statements
            .get(0)
            .unwrap()
            .contract_function_call();
        assert_eq!(split_proofs.fri_merkle_statements.len(), 6);

        let fri_merkle_contract_args_file = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/fri_0_contract_args.json"
        ));
        let deser_fri_merkle_contract_args: VerifyFRICall =
            serde_json::from_str(fri_merkle_contract_args_file).unwrap();
        assert_json_diff::assert_json_eq!(fri_merkle_contract_args, deser_fri_merkle_contract_args);
    }

    #[test]
    fn test_main_proof_contract_args() {
        let split_proofs = get_split_proofs();
        let main_proof_contract_args = split_proofs
            .main_proof
            .contract_function_call(vec![U256::zero()]);

        let main_proof_contract_args_file = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/main_proof_contract_args.json"
        ));
        let deser_main_proof_contract_args: VerifyProofAndRegisterCall =
            serde_json::from_str(main_proof_contract_args_file).unwrap();
        assert_json_diff::assert_json_eq!(main_proof_contract_args, deser_main_proof_contract_args);
    }
}
