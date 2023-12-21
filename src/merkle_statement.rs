use std::sync::Arc;

use ethers::{
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};

use crate::ContractFunctionCall;

/// Decommitment for a merkle statement
#[derive(Serialize, Deserialize, Debug)]
pub struct MerkleStatement {
    expected_root: U256,
    n_unique_queries: usize,
    merkle_height: usize,
    merkle_queue_indices: Vec<U256>,
    merkle_queue_values: Vec<U256>,
    proof: Vec<U256>,
}

abigen!(
    MerkleStatementContract,
    r#"[
        function verifyMerkle(uint256[] proof,uint256[] merkle_queue,uint256 merkle_height,uint256 expected_root)
    ]"#,
    derives(serde::Deserialize, serde::Serialize)
);

impl MerkleStatement {
    pub fn new(
        expected_root: U256,
        n_unique_queries: usize,
        merkle_height: usize,
        merkle_queue_indices: Vec<U256>,
        merkle_queue_values: Vec<U256>,
        proof: Vec<U256>,
    ) -> MerkleStatement {
        MerkleStatement {
            expected_root,
            n_unique_queries,
            merkle_height,
            merkle_queue_indices,
            merkle_queue_values,
            proof,
        }
    }

    /// Constructs the merkle_queue by interleaving indices and values.
    fn merkle_queue(&self) -> Vec<U256> {
        self.merkle_queue_indices
            .iter()
            .zip(self.merkle_queue_values.iter())
            .flat_map(|(&index, &value)| vec![index, value])
            .collect()
    }

    /// Constructs `verifyMerkle` contract function call.
    pub fn contract_function_call(&self) -> VerifyMerkleCall {
        VerifyMerkleCall {
            proof: self.proof.clone(),
            merkle_queue: self.merkle_queue(),
            merkle_height: U256::from(self.merkle_height),
            expected_root: self.expected_root,
        }
    }

    /// Initiates `verifyMerkle` contract call.
    pub fn verify(
        &self,
        address: Address,
        signer: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    ) -> ContractFunctionCall {
        let contract = MerkleStatementContract::new(address, signer);

        let verify_merkle_call = self.contract_function_call();
        contract.method("verifyMerkle", verify_merkle_call).unwrap()
    }
}
