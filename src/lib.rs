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

pub fn default_prime() -> U256 {
    U256::from(2).pow(U256::from(251))
        + U256::from(17) * U256::from(2).pow(U256::from(192))
        + U256::from(1)
}

pub type SignerMiddlewareType = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;
pub type ArcSignerMiddleware = Arc<SignerMiddlewareType>;
pub type ContractFunctionCall = FunctionCall<ArcSignerMiddleware, SignerMiddlewareType, ()>;
