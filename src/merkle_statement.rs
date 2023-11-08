use std::str::FromStr;

use ethers::types::U256;
use serde::{Deserialize, Serialize};

use crate::serialize_vec_u256_as_number;

#[derive(Serialize, Deserialize, Debug)]
pub struct MerkleStatement {
    expected_root: U256,
    n_unique_queries: usize,
    merkle_height: usize,
    #[serde(serialize_with = "serialize_vec_u256_as_number")]
    merkle_queue_indices: Vec<U256>,
    merkle_queue_values: Vec<U256>,
    proof: Vec<U256>,
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

    // Constructs the merkle_queue by interleaving indices and values.
    pub fn merkle_queue(&self) -> Vec<U256> {
        self.merkle_queue_indices
            .iter()
            .zip(self.merkle_queue_values.iter())
            .flat_map(|(&index, &value)| vec![index, value])
            .collect()
    }

    // Convert struct to serde json
    pub fn to_json(&self) -> serde_json::Value {
        let mut json = serde_json::to_value(self).unwrap();
        // map merkle_queue_indices to serde_json value
        self.merkle_queue_indices
            .iter()
            .enumerate()
            .for_each(|(i, v)| {
                json["merkle_queue_indices"][i] = serde_json::Value::Number(
                    serde_json::Number::from_str(&v.to_string()).unwrap(),
                );
            });

        json
    }
}
