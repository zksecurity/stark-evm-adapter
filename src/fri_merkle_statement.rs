use ethers::types::U256;
use serde::Serialize;

use crate::{serialize_u256_as_number, serialize_vec_u256_as_number};

#[derive(Serialize, Debug)]
pub struct FRIMerkleStatement {
    #[serde(serialize_with = "serialize_u256_as_number")]
    pub expected_root: U256,
    #[serde(serialize_with = "serialize_u256_as_number")]
    pub evaluation_point: U256,
    pub fri_step_size: usize,
    #[serde(serialize_with = "serialize_vec_u256_as_number")]
    pub input_layer_queries: Vec<U256>,
    #[serde(serialize_with = "serialize_vec_u256_as_number")]
    pub output_layer_queries: Vec<U256>,
    #[serde(serialize_with = "serialize_vec_u256_as_number")]
    pub input_layer_values: Vec<U256>,
    #[serde(serialize_with = "serialize_vec_u256_as_number")]
    pub output_layer_values: Vec<U256>,
    #[serde(serialize_with = "serialize_vec_u256_as_number")]
    pub input_layer_inverses: Vec<U256>,
    #[serde(serialize_with = "serialize_vec_u256_as_number")]
    pub output_layer_inverses: Vec<U256>,
    #[serde(serialize_with = "serialize_vec_u256_as_number")]
    pub input_interleaved: Vec<U256>,
    #[serde(serialize_with = "serialize_vec_u256_as_number")]
    pub output_interleaved: Vec<U256>,
    #[serde(serialize_with = "serialize_vec_u256_as_number")]
    pub proof: Vec<U256>,
}
