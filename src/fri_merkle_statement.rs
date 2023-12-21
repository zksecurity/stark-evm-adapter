use std::sync::Arc;

use ethers::{
    abi::Address,
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::U256,
};
use serde::{Deserialize, Serialize};

use crate::ContractFunctionCall;

/// Decommitment for a FRI layer merkle statement
#[derive(Serialize, Deserialize, Debug)]
pub struct FRIMerkleStatement {
    pub expected_root: U256,
    pub evaluation_point: U256,
    pub fri_step_size: usize,
    pub input_layer_queries: Vec<U256>,
    pub output_layer_queries: Vec<U256>,
    pub input_layer_values: Vec<U256>,
    pub output_layer_values: Vec<U256>,
    pub input_layer_inverses: Vec<U256>,
    pub output_layer_inverses: Vec<U256>,
    pub input_interleaved: Vec<U256>,
    pub output_interleaved: Vec<U256>,
    pub proof: Vec<U256>,
}

abigen!(
    FriStatementContract,
    r#"[
        function verifyFRI(uint256[] proof,uint256[] fri_queue,uint256 evaluation_point,uint256 fri_step_size,uint256 expected_root)
    ]"#,
    derives(serde::Deserialize, serde::Serialize)
);

impl FRIMerkleStatement {
    /// Constructs `verifyFRI` contract function call
    pub fn contract_function_call(&self) -> VerifyFRICall {
        let mut fri_queue: Vec<U256> = self.input_interleaved.clone();
        fri_queue.push(U256::zero());

        VerifyFRICall {
            proof: self.proof.clone(),
            fri_queue,
            evaluation_point: self.evaluation_point,
            fri_step_size: U256::from(self.fri_step_size),
            expected_root: self.expected_root,
        }
    }

    /// Initiates `verifyFRI` contract function call
    pub fn verify(
        &self,
        address: Address,
        signer: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    ) -> ContractFunctionCall {
        let contract = FriStatementContract::new(address, signer);

        let call = self.contract_function_call();
        contract.method("verifyFRI", call).unwrap()
    }
}
