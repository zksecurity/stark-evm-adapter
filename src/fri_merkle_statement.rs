use ethers::types::U256;
use serde::{Deserialize, Serialize};

use crate::serialization::{
    deserialize_u256_as_number, deserialize_vec_u256_as_number, serialize_u256_as_number,
    serialize_vec_u256_as_number,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct FRIMerkleStatement {
    #[serde(
        serialize_with = "serialize_u256_as_number",
        deserialize_with = "deserialize_u256_as_number"
    )]
    pub expected_root: U256,
    #[serde(
        serialize_with = "serialize_u256_as_number",
        deserialize_with = "deserialize_u256_as_number"
    )]
    pub evaluation_point: U256,
    pub fri_step_size: usize,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub input_layer_queries: Vec<U256>,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub output_layer_queries: Vec<U256>,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub input_layer_values: Vec<U256>,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub output_layer_values: Vec<U256>,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub input_layer_inverses: Vec<U256>,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub output_layer_inverses: Vec<U256>,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub input_interleaved: Vec<U256>,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub output_interleaved: Vec<U256>,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub proof: Vec<U256>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FRIMerkleStatementContractArgs {
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub proof: Vec<U256>,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub fri_queue: Vec<U256>,
    #[serde(
        serialize_with = "serialize_u256_as_number",
        deserialize_with = "deserialize_u256_as_number"
    )]
    pub evaluation_point: U256,
    #[serde(
        serialize_with = "serialize_u256_as_number",
        deserialize_with = "deserialize_u256_as_number"
    )]
    pub fri_step_size: U256,
    #[serde(
        serialize_with = "serialize_u256_as_number",
        deserialize_with = "deserialize_u256_as_number"
    )]
    pub expected_root: U256,
}

impl From<FRIMerkleStatement> for FRIMerkleStatementContractArgs {
    fn from(statement: FRIMerkleStatement) -> Self {
        let mut fri_queue: Vec<U256> = statement.input_interleaved.clone();
        fri_queue.push(U256::zero());

        FRIMerkleStatementContractArgs {
            proof: statement.proof,
            fri_queue,
            evaluation_point: statement.evaluation_point,
            fri_step_size: U256::from(statement.fri_step_size),
            expected_root: statement.expected_root,
        }
    }
}
