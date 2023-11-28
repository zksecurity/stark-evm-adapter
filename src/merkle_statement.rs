use ethers::types::U256;
use serde::{Deserialize, Serialize};

use crate::serialization::{
    deserialize_u256_as_number, deserialize_vec_u256_as_number, serialize_u256_as_number,
    serialize_vec_u256_as_number,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct MerkleStatement {
    expected_root: U256,
    n_unique_queries: usize,
    merkle_height: usize,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    merkle_queue_indices: Vec<U256>,
    merkle_queue_values: Vec<U256>,
    proof: Vec<U256>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MerkleStatementContractArgs {
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub proof: Vec<U256>,
    #[serde(
        serialize_with = "serialize_vec_u256_as_number",
        deserialize_with = "deserialize_vec_u256_as_number"
    )]
    pub merkle_queue: Vec<U256>,
    #[serde(
        serialize_with = "serialize_u256_as_number",
        deserialize_with = "deserialize_u256_as_number"
    )]
    pub merkle_height: U256,
    #[serde(
        serialize_with = "serialize_u256_as_number",
        deserialize_with = "deserialize_u256_as_number"
    )]
    pub expected_root: U256,
}

// serialize contract args
impl From<MerkleStatement> for MerkleStatementContractArgs {
    fn from(statement: MerkleStatement) -> Self {
        let merkle_queue = statement
            .merkle_queue_indices
            .iter()
            .zip(statement.merkle_queue_values.iter())
            .flat_map(|(&index, &value)| vec![index, value])
            .collect();

        MerkleStatementContractArgs {
            proof: statement.proof,
            merkle_queue,
            merkle_height: U256::from(statement.merkle_height),
            expected_root: statement.expected_root,
        }
    }
}

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
    pub fn merkle_queue(&self) -> Vec<U256> {
        self.merkle_queue_indices
            .iter()
            .zip(self.merkle_queue_values.iter())
            .flat_map(|(&index, &value)| vec![index, value])
            .collect()
    }
}
