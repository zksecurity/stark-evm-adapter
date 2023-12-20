#![doc = include_str!("../README.md")]

use std::sync::Arc;

use ethers::{
    contract::FunctionCall,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::U256,
};

pub mod annotated_proof;
pub mod annotation_parser;
pub mod errors;
pub mod fri_merkle_statement;
pub mod merkle_statement;
pub mod oods_statement;
pub mod serialization;

/// Default prime field for cairo. This prime will be used when modular operations are needed.
pub fn default_prime() -> U256 {
    U256::from(2).pow(U256::from(251))
        + U256::from(17) * U256::from(2).pow(U256::from(192))
        + U256::from(1)
}

/// A type alias for ethers contract function call.
pub type ContractFunctionCall = FunctionCall<
    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    (),
>;
