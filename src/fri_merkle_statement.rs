use ethers::types::U256;
use serde::{Serialize, Serializer};

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

fn serialize_u256_as_number<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value_str = value.to_string();
    let json_value = Value::Number(value_str.parse::<serde_json::Number>().unwrap());

    json_value.serialize(serializer)
}

use serde::ser::SerializeSeq;
use serde_json::Value;

fn serialize_vec_u256_as_number<S>(vec: &[U256], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(vec.len()))?;
    for element in vec {
        // Use the previously defined serialize_u256_as_number for each element
        seq.serialize_element(&SerializeU256AsNumber(element))?;
    }
    seq.end()
}

// Wrapper type to use the serialize_u256_as_number function
struct SerializeU256AsNumber<'a>(&'a U256);

impl<'a> Serialize for SerializeU256AsNumber<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_u256_as_number(self.0, serializer)
    }
}