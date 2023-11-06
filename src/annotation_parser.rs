use ethers::abi::Token;
use ethers::utils::hex::FromHexError;
use ethers::utils::keccak256;
use ethers::{types::U256, utils::hex};
use num_bigint::BigUint;
use num_traits::{Num, One};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    num::ParseIntError,
};

use crate::fri_merkle_statement::FriMerkleStatement;
use crate::merkle_statement::MerkleStatement;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MerkleLine {
    pub name: String,
    // todo: check if better to change to usize
    pub node: U256, // Assuming the Python `int` is equivalent to a U256
    pub digest: String,
    pub annotation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FriLine {
    pub name: String,
    pub row: usize,
    pub col: usize,
    pub element: String,
    pub annotation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FriXInvLine {
    pub name: String,
    pub index: usize,
    pub inv: String,
    pub annotation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitmentLine {
    pub name: String,
    pub digest: String,
    pub annotation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EvalPointLine {
    pub name: String,
    pub point: String, // In Python, it could be more than just a String. Adjust accordingly.
    pub annotation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FriExtras {
    pub values: Vec<FriLine>,
    pub inverses: Vec<FriXInvLine>,
}

// For dictionaries in Python, we use HashMaps in Rust.
pub type MerkleFactRegistryInput = HashMap<String, serde_json::Value>;
pub type MerkleExtrasDict = HashMap<String, Vec<MerkleLine>>;
pub type FRIMerkleFactRegistryInput = HashMap<String, serde_json::Value>; // Adjust according to the structure

// Parses hex strings and pads with zeros to make it 64 characters long
fn extract_hex(line: &str) -> Result<String, regex::Error> {
    let re = Regex::new(r"\(0x([0-9a-f]+)\)")?;
    Ok(re
        .captures(line)
        .and_then(|cap| cap.get(1))
        .map_or_else(|| String::new(), |m| format!("{:0>64}", m.as_str())))
}

fn is_merkle_line(line: &str) -> bool {
    line.contains("Decommitment") && line.contains("node") && line.contains("Hash")
}

fn parse_merkle_line(line: &str) -> Result<MerkleLine, regex::Error> {
    let name = line
        .split('/')
        .last()
        .unwrap()
        .split(':')
        .next()
        .unwrap()
        .to_string();
    let node: U256 = U256::from_dec_str(
        line.split("node ")
            .nth(1)
            .unwrap()
            .split(':')
            .next()
            .unwrap(),
    )
    .unwrap();

    let digest = extract_hex(line)?;

    Ok(MerkleLine {
        name,
        node,
        digest,
        annotation: line.to_string(),
    })
}

fn is_merkle_data_line(line: &str) -> bool {
    line.contains("Decommitment") && line.contains("element #") && line.contains("Data")
}

fn parse_merkle_data_line(line: &str) -> Result<MerkleLine, regex::Error> {
    let name = line
        .split('/')
        .last()
        .unwrap()
        .split(':')
        .next()
        .unwrap()
        .to_string();
    let node = line
        .split("element #")
        .nth(1)
        .unwrap()
        .split(':')
        .next()
        .unwrap()
        .parse::<U256>()
        .unwrap();
    let digest = extract_hex(line)?;

    Ok(MerkleLine {
        name,
        node,
        digest,
        annotation: line.to_string(),
    })
}

fn is_fri_line(line: &str) -> bool {
    line.contains("Decommitment")
        && line.contains("Row")
        && line.contains("Field Element")
        && !line.contains("Virtual Oracle")
}

// Parses a FRI decommitment line
fn parse_fri_line(line: &str) -> Result<FriLine, Box<dyn std::error::Error>> {
    let re = Regex::new(r"/Decommitment/(Last Layer|Layer \d+): Row (\d+), Column (\d+)").unwrap();

    match re.captures(line) {
        Some(captures) => {
            let name = captures[1].to_string();
            let row: usize = captures[2].parse().unwrap();
            let col: usize = captures[3].parse().unwrap();
            let element = extract_hex(line)?;

            Ok(FriLine {
                name,
                row,
                col,
                element,
                annotation: line.to_string(),
            })
        }
        None => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to parse FRI line",
        ))),
    }
}

// Checks if a line is a FRI xInv line
fn is_fri_xinv_line(line: &str) -> bool {
    line.contains("Decommitment") && line.contains("xInv") && line.contains("Field Element")
}

// Parses a FRI xInv line
fn parse_fri_xinv_line(line: &str) -> Result<FriXInvLine, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split('/').collect();
    let name = parts.last().unwrap().split(':').next().unwrap().to_string();
    let index_str = line
        .split("index ")
        .nth(1)
        .unwrap()
        .split(':')
        .next()
        .unwrap();
    let index = index_str.parse::<usize>()?;
    let inv = extract_hex(line)?;

    Ok(FriXInvLine {
        name,
        index,
        inv,
        annotation: line.to_string(),
    })
}

// Checks if a line is a commitment line
fn is_commitment_line(line: &str) -> bool {
    line.contains("Commitment") && line.contains("Hash")
}

// Parses a commitment line
fn parse_commitment_line(
    line: &str,
    trace_commitment_counter: &mut usize,
) -> Result<(CommitmentLine, usize), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split(':').collect();
    let path_parts: Vec<&str> = parts[2].trim().split('/').collect();
    let mut name = None;
    let new_trace_commitment_counter;

    if path_parts.last() == Some(&"Commit on Trace") {
        name = Some(format!("Trace {}", trace_commitment_counter));
        *trace_commitment_counter += 1;
        new_trace_commitment_counter = *trace_commitment_counter;
    } else if path_parts[path_parts.len() - 2] == "Commitment" {
        name = Some(path_parts.last().unwrap().to_string());
        new_trace_commitment_counter = *trace_commitment_counter;
    } else {
        return Err(From::from("Unexpected commitment path format"));
    }

    let digest = extract_hex(line)?;
    Ok((
        CommitmentLine {
            name: name.unwrap(),
            digest,
            annotation: line.to_string(),
        },
        new_trace_commitment_counter,
    ))
}

// Check if a line is an evaluation point line
fn is_eval_point_line(line: &str) -> bool {
    line.contains("Evaluation point") && line.contains("Layer")
}

// Parses an evaluation point line
fn parse_eval_point_line(line: &str) -> Result<EvalPointLine, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split('/').collect();
    let name = parts.last().unwrap().split(':').next().unwrap().to_string();
    let point = extract_hex(line)?;

    Ok(EvalPointLine {
        name,
        point,
        annotation: line.to_string(),
    })
}

// Parses a proof annotation line to indices
fn line_to_indices(line: &str) -> (usize, usize) {
    if !line.starts_with("P->V[") {
        return (0, 0);
    }
    let indices_str = &line[5..line.find(']').unwrap()];
    let indices: Vec<usize> = indices_str
        .split(':')
        .map(|x| x.parse::<usize>().unwrap_or(0)) // Assuming we want to return 0 on parse failure
        .collect();

    (indices[0], indices.get(1).cloned().unwrap_or(0)) // If there's no second element, return 0
}

// Function to generate a Merkle statement call
pub fn gen_merkle_statement_call(
    merkle_extras: Vec<MerkleLine>,
    merkle_original: Vec<MerkleLine>,
    merkle_commit: CommitmentLine,
) -> MerkleStatement {
    let qs: Vec<&str> = merkle_extras.iter().map(|n| &n.name[..]).collect();
    let heights: Vec<usize> = merkle_extras
        .iter()
        .map(|mline| mline.node.bits() - 1)
        .collect();
    assert!(heights.iter().all(|&h| h == heights[0]));

    let root = U256::from_str_radix(&merkle_commit.digest, 16).expect("Invalid hex number");
    let merkle_queue_values: Vec<U256> = merkle_extras
        .iter()
        .map(|mline| U256::from_str_radix(&mline.digest, 16).expect("Invalid hex number"))
        .collect();
    let proof: Vec<U256> = merkle_original
        .iter()
        .map(|mline| U256::from_str_radix(&mline.digest, 16).expect("Invalid hex number"))
        .collect();
    let merkle_queue_indices: Vec<U256> = merkle_extras.iter().map(|mline| mline.node).collect();

    MerkleStatement::new(
        root,
        qs.len(),
        heights[0],
        merkle_queue_indices,
        merkle_queue_values,
        proof,
    )
}

fn montgomery_encode(element: &str) -> U256 {
    let prime = BigUint::from_str_radix(
        "800000000000011000000000000000000000000000000000000000000000001",
        16,
    )
    .expect("Invalid prime number");
    let num = BigUint::from_str_radix(element, 16).expect("Invalid hex number for element");

    let r = BigUint::one() << 256; // Use 2^256 as R
    let encoded: BigUint = (num * r) % prime; // this seems to lost the purpose of montgomery encoding which aims to avoid division
    // println!("encoded: {}, element: {}", encoded, element);
    U256::from_str_radix(&encoded.to_str_radix(10), 10).unwrap()
}

fn interleave<T: Clone>(a: Vec<T>, b: Vec<T>, c: Vec<T>) -> Vec<T> {
    a.into_iter()
        .zip(b)
        .zip(c)
        .flat_map(|((x, y), z)| vec![x, y, z])
        .collect()
}

pub fn gen_fri_merkle_statement_call(
    fri_extras: FriExtras,
    fri_extras_next: FriExtras,
    fri_original: Vec<FriLine>,
    merkle_original: Vec<MerkleLine>,
    merkle_extras: Vec<MerkleLine>,
    merkle_commitment: CommitmentLine,
    evaluation_point: EvalPointLine,
) -> FRIMerkleFactRegistryInput {
    let mut statement_json = FRIMerkleFactRegistryInput::new();

    let root = U256::from_str_radix(&merkle_commitment.digest, 16).expect("Invalid hex number");
    statement_json.insert(
        "expected_root".to_string(),
        serde_json::Value::Number(serde_json::Number::from_str(&root.to_string()).unwrap()),
    );

    let eval_point = U256::from_str_radix(&evaluation_point.point, 16).expect("Invalid hex number");
    statement_json.insert(
        "evaluation_point".to_string(),
        serde_json::Value::Number(serde_json::Number::from_str(&eval_point.to_string()).unwrap()),
    );

    let heights: Vec<usize> = merkle_extras
        .iter()
        .map(|mline| mline.node.bits() - 1)
        .collect();
    assert_eq!(heights.iter().cloned().collect::<HashSet<_>>().len(), 1);
    let output_height = heights[0];

    let mut rows_to_cols: HashMap<usize, Vec<usize>> = HashMap::new();
    for fline in fri_extras.values.iter().chain(fri_original.iter()) {
        rows_to_cols.entry(fline.row).or_default().push(fline.col);
    }
    println!("rows_to_cols: {:?}", rows_to_cols);
    let row_lens: Vec<usize> = rows_to_cols
        .values()
        .map(|v| v.iter().cloned().collect::<HashSet<_>>().len())
        .collect();
    println!("row_lens: {:?}", row_lens);
    assert_eq!(row_lens.iter().cloned().collect::<HashSet<_>>().len(), 1);

    let step_size = (row_lens[0] as f64).log2() as usize;
    statement_json.insert(
        "fri_step_size".to_string(),
        serde_json::Value::Number(serde_json::Number::from(step_size)),
    );
    let input_height = output_height + step_size;

    statement_json.insert(
        "input_layer_queries".to_string(),
        fri_extras
            .inverses
            .iter()
            .map(|fline| fline.index + (1 << input_height))
            .collect(),
    );

    let output_layer_queries = merkle_extras
        .iter()
        .map(|mline| mline.node)
        .collect::<Vec<_>>();
    statement_json.insert(
        "output_layer_queries".to_string(),
        serde_json::Value::Array(
            output_layer_queries
                .iter()
                .map(|&node| serde_json::Value::Number(serde_json::Number::from(node.low_u64())))
                .collect(),
        ),
    );

    let input_layer_values = fri_extras
        .values
        .iter()
        // todo check how this encoding work with other parts
        .map(|fline| montgomery_encode(&fline.element))
        .collect::<Vec<_>>();

    statement_json.insert(
        "input_layer_values".to_string(),
        serde_json::Value::Array(
            input_layer_values
                .iter()
                .map(|&val| serde_json::Value::Number(serde_json::Number::from_str(&val.to_string()).unwrap()))
                .collect(),
        ),
    );

    let output_layer_values = fri_extras_next
        .values
        .iter()
        .map(|fline| montgomery_encode(&fline.element))
        .collect::<Vec<_>>();

    statement_json.insert(
        "output_layer_values".to_string(),
        serde_json::Value::Array(
            output_layer_values
                .iter()
                .map(|&val| serde_json::Value::Number(serde_json::Number::from(val.low_u64())))
                .collect(),
        ),
    );

    let input_layer_inverses = fri_extras
        .inverses
        .iter()
        .map(|fline| U256::from_str_radix(&fline.inv, 16).expect("Invalid hex number"))
        .collect::<Vec<_>>();
    statement_json.insert(
        "input_layer_inverses".to_string(),
        serde_json::Value::Array(
            input_layer_inverses
                .iter()
                .map(|&inv| serde_json::Value::Number(serde_json::Number::from(inv.low_u64())))
                .collect(),
        ),
    );

    let output_layer_inverses = fri_extras_next
        .inverses
        .iter()
        .map(|fline| U256::from_str_radix(&fline.inv, 16).expect("Invalid hex number"))
        .collect::<Vec<_>>();
    statement_json.insert(
        "output_layer_inverses".to_string(),
        serde_json::Value::Array(
            output_layer_inverses
                .iter()
                .map(|&inv| serde_json::Value::Number(serde_json::Number::from(inv.low_u64())))
                .collect(),
        ),
    );

    statement_json.insert(
        "input_interleaved".to_string(),
        serde_json::Value::Array(interleave(
            statement_json["input_layer_queries"]
                .as_array()
                .unwrap()
                .clone(),
            statement_json["input_layer_values"]
                .as_array()
                .unwrap()
                .clone(),
            statement_json["input_layer_inverses"]
                .as_array()
                .unwrap()
                .clone(),
        )),
    );

    statement_json.insert(
        "output_interleaved".to_string(),
        serde_json::Value::Array(interleave(
            statement_json["output_layer_queries"]
                .as_array()
                .unwrap()
                .clone(),
            statement_json["output_layer_values"]
                .as_array()
                .unwrap()
                .clone(),
            statement_json["output_layer_inverses"]
                .as_array()
                .unwrap()
                .clone(),
        )),
    );

    statement_json.insert(
        "proof".to_string(),
        fri_original
            .iter()
            .map(|fline| montgomery_encode(&fline.element))
            .chain(merkle_original.iter().map(|mline| {
                U256::from_str_radix(&mline.digest, 16).expect("Invalid hex number")
                // .as_u64()
            }))
            .map(|num| {
                serde_json::Value::Number(serde_json::Number::from_str(&num.to_string()).unwrap())
            })
            .collect(),
    );

    statement_json
}

fn parse_merkles_extra(extra_annot_lines: Vec<&str>) -> Result<MerkleExtrasDict, regex::Error> {
    let mut merkle_extras_dict = MerkleExtrasDict::new();

    for line in extra_annot_lines {
        if !is_merkle_line(line) {
            continue;
        }
        let mline = parse_merkle_line(line)?;
        merkle_extras_dict
            .entry(mline.name.clone())
            .or_default()
            .push(mline);
    }

    Ok(merkle_extras_dict)
}

fn parse_merkles_original(
    orig_proof: &[u8],
    annot_lines: Vec<&str>,
) -> Result<
    (
        MerkleExtrasDict,
        HashMap<String, CommitmentLine>,
        Vec<u8>,
        String,
    ),
    Box<dyn Error>,
> {
    let mut merkle_original_dict = MerkleExtrasDict::new();
    let mut merkle_commits_dict = HashMap::new();
    let mut main_proof = Vec::new();
    let mut main_annot = String::new();
    let mut trace_commitment_counter = 0;

    for line in annot_lines {
        if is_commitment_line(line) {
            match parse_commitment_line(line, &mut trace_commitment_counter) {
                Ok((cline, new_trace_commitment_counter)) => {
                    merkle_commits_dict.insert(cline.name.clone(), cline);
                    trace_commitment_counter = new_trace_commitment_counter;
                }
                Err(e) => {
                    eprintln!("Error parsing commitment line: {}", e);
                    return Err(e);
                }
            }
        } else if !is_merkle_line(line) {
            main_annot.push_str(line);
            main_annot.push('\n');
            let (start, end) = line_to_indices(line);
            main_proof.extend_from_slice(&orig_proof[start..end]);
        } else {
            let mline = parse_merkle_line(line)?;
            merkle_original_dict
                .entry(mline.name.clone())
                .or_default()
                .push(mline);
        }
    }

    Ok((
        merkle_original_dict,
        merkle_commits_dict,
        main_proof,
        main_annot,
    ))
}

fn parse_fri_merkles_extra(
    // todo does it have to be &str instead of String?
    extra_annot_lines: Vec<&str>,
) -> Result<(MerkleExtrasDict, Vec<FriExtras>), Box<dyn Error>> {
    let mut merkle_extras_dict = MerkleExtrasDict::new();
    let mut fri_extras_dict = HashMap::new();
    let mut fri_names = Vec::new();

    for line in extra_annot_lines {
        if is_merkle_line(line) {
            let mline = parse_merkle_line(line)?;
            merkle_extras_dict
                .entry(mline.name.clone())
                .or_default()
                .push(mline);
        } else if is_fri_line(line) {
            let fline = parse_fri_line(line)?;
            if !fri_extras_dict.contains_key(&fline.name) {
                fri_extras_dict.insert(
                    fline.name.clone(),
                    FriExtras {
                        values: Vec::new(),
                        inverses: Vec::new(),
                    },
                );
                fri_names.push(fline.name.clone());
            }
            fri_extras_dict
                .get_mut(&fline.name)
                .unwrap()
                .values
                .push(fline);
        } else if is_fri_xinv_line(line) {
            let fxline = parse_fri_xinv_line(line)?;
            fri_extras_dict
                .get_mut(&fxline.name)
                .unwrap()
                .inverses
                .push(fxline);
        }
    }
    let fri_extras_list = Vec::from_iter(
        fri_names
            .into_iter()
            .map(|name| fri_extras_dict.remove(&name).unwrap()),
    );

    Ok((merkle_extras_dict, fri_extras_list))
}

pub fn parse_fri_merkles_original(
    orig_proof: Vec<u8>,
    annot_lines: Vec<String>,
) -> Result<
    (
        MerkleExtrasDict,
        HashMap<String, CommitmentLine>,
        HashMap<String, Vec<FriLine>>,
        Vec<EvalPointLine>,
        Vec<String>,
        Vec<u8>,
        String,
        HashSet<String>,
    ),
    Box<dyn Error>,
> {
    let mut merkle_original_dict = MerkleExtrasDict::new();
    let mut merkle_commits_dict = HashMap::new();
    let mut fri_original_dict = HashMap::new();
    let mut fri_names = Vec::new();
    let mut eval_points_list = Vec::new();
    let mut merkle_patches = HashSet::new();
    let mut main_proof = Vec::new();
    let mut main_annot = String::new();
    let mut trace_commitment_counter = 0;

    for line in annot_lines {
        if is_commitment_line(&line) {
            match parse_commitment_line(&line, &mut trace_commitment_counter) {
                Ok((cline, new_trace_commitment_counter)) => {
                    merkle_commits_dict.insert(cline.name.clone(), cline);
                    trace_commitment_counter = new_trace_commitment_counter;
                }
                Err(e) => return Err(e.into()),
            }
        } else if is_eval_point_line(&line) {
            let epline = parse_eval_point_line(&line)?;
            eval_points_list.push(epline);
        }

        if is_merkle_line(&line) {
            let mline = parse_merkle_line(&line)?;
            merkle_original_dict
                .entry(mline.name.clone())
                .or_insert_with(Vec::new)
                .push(mline);
        } else if is_merkle_data_line(&line) {
            let mline = parse_merkle_data_line(&line)?;
            let cloned_mline_name = mline.name.clone();
            merkle_original_dict
                .entry(mline.name.clone())
                .or_insert_with(Vec::new)
                .push(mline);
            merkle_patches.insert(cloned_mline_name);
        } else if is_fri_line(&line) {
            let fline = parse_fri_line(&line)?;
            if !fri_original_dict.contains_key(&fline.name) {
                fri_original_dict.insert(fline.name.clone(), Vec::new());
                fri_names.push(fline.name.clone());
            }
            fri_original_dict.get_mut(&fline.name).unwrap().push(fline);
        } else {
            main_annot.push_str(&line);
            main_annot.push('\n');
            let (start, end) = line_to_indices(&line);
            main_proof.extend_from_slice(&orig_proof[start..end]);
        }
    }

    Ok((
        merkle_original_dict,
        merkle_commits_dict,
        fri_original_dict,
        eval_points_list,
        fri_names,
        main_proof,
        main_annot,
        merkle_patches,
    ))
}

fn single_column_merkle_patch(
    merkle_patches: &HashSet<String>,
    merkle_extras_dict: &mut HashMap<String, Vec<MerkleLine>>,
    annot_lines: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    for name in merkle_patches {
        let merkle_extras = merkle_extras_dict
            .get(name)
            .ok_or("Name not found in merkle_extras_dict")?
            .clone();
        let heights: Vec<usize> = merkle_extras
            .iter()
            .map(|mline| mline.node.leading_zeros() as usize - 1)
            .collect();
        // Ensure all heights are the same
        let height = *heights.get(0).ok_or("No heights found")?;
        if !heights.iter().all(|&h| h == height) {
            return Err(From::from("Mismatched heights"));
        }
        // When patched, the apparent Merkle height is one lower than the original.
        let height = height + 1;
        merkle_extras_dict.insert(name.clone(), Vec::new());

        for line in annot_lines {
            if line.contains(name) && line.contains("Column 0") && line.contains("Field Element") {
                // It is not a Fri line, but the structure is similar enough for the parser.
                let parsed_fri_line = parse_fri_line(line)?;
                // let node = parsed_fri_line.row + (1 << height);
                let node = U256::from(parsed_fri_line.row) + U256::from(1 << height);
                let element = montgomery_encode(&parsed_fri_line.element);
                let element_hex = format!("{:0>64x}", element);
                let merkle_line = MerkleLine {
                    name: name.clone(),
                    node,
                    digest: element_hex,
                    annotation: line.clone(),
                };
                merkle_extras_dict
                    .get_mut(name)
                    .ok_or("Name not found in merkle_extras_dict")?
                    .push(merkle_line);
            }
        }
    }
    Ok(())
}

fn extract_proof_and_annotations(
    proof_json: Value,
) -> Result<(Vec<u8>, Vec<String>, Vec<String>), Box<dyn Error>> {
    let orig_proof_hex = proof_json["proof_hex"]
        .as_str()
        .ok_or("proof_hex field is missing or not a string")?;
    let orig_proof = hex::decode(&orig_proof_hex[2..])?;

    let annot_lines_json = proof_json["annotations"]
        .as_array()
        .ok_or("annotations field is missing or not an array")?;
    let annot_lines: Vec<String> = annot_lines_json[2..(annot_lines_json.len() - 8)]
        .iter()
        .map(|val| val.to_string())
        .collect();

    let extra_annot_lines_json = proof_json["extra_annotations"]
        .as_array()
        .ok_or("extra_annotations field is missing or not an array")?;
    let extra_annot_lines: Vec<String> = extra_annot_lines_json
        .iter()
        .map(|val| val.to_string())
        .collect();

    Ok((orig_proof, annot_lines, extra_annot_lines))
}

pub fn split_fri_merkle_statements(
    proof_json: Value,
) -> Result<
    (
        Vec<u8>,
        HashMap<String, MerkleStatement>,
        Vec<FRIMerkleFactRegistryInput>,
    ),
    Box<dyn std::error::Error>,
> {
    let (orig_proof, annot_lines, extra_annot_lines) = extract_proof_and_annotations(proof_json)?;
    let (mut merkle_extras_dict, fri_extras_list) =
        parse_fri_merkles_extra(extra_annot_lines.iter().map(|s| s.as_str()).collect())?;
    let (
        merkle_original_dict,
        merkle_commits_dict,
        fri_original_dict,
        eval_points_list,
        fri_names,
        mut main_proof,
        _,
        merkle_patches,
    ) = parse_fri_merkles_original(orig_proof, annot_lines.clone())?;
    let merkle_names: HashSet<_> = HashSet::from_iter(merkle_extras_dict.keys().cloned());
    assert_eq!(
        merkle_names,
        HashSet::from_iter(merkle_original_dict.keys().cloned())
    );

    if !merkle_patches.is_empty() {
        single_column_merkle_patch(&merkle_patches, &mut merkle_extras_dict, &annot_lines);
    }

    let merkle_statements = merkle_names
        .into_iter()
        .filter(|name| !fri_original_dict.contains_key(name))
        .map(|name| {
            let statement = gen_merkle_statement_call(
                merkle_extras_dict[&name].clone(),
                merkle_original_dict[&name].clone(),
                merkle_commits_dict[&name].clone(),
            );
            (name, statement)
        })
        .collect::<HashMap<_, _>>();

    let fri_merkle_statements: Vec<FRIMerkleFactRegistryInput> = fri_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            gen_fri_merkle_statement_call(
                fri_extras_list[i].clone(),
                fri_extras_list[i + 1].clone(),
                fri_original_dict[&name].clone(),
                merkle_original_dict[&name].clone(),
                merkle_extras_dict[&name].clone(),
                merkle_commits_dict[&name].clone(),
                eval_points_list[i].clone(),
            )
        })
        .collect();

    for fri in &fri_merkle_statements[..fri_merkle_statements.len() - 1] {
        let fri_output_interleaved = fri
            .get("output_interleaved")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|val| Token::Uint(val.as_u64().unwrap().into()))
            .collect();

        let encoded = ethers::abi::encode_packed(&[Token::Array(fri_output_interleaved)])?;
        let hash = keccak256(encoded);
        main_proof.extend_from_slice(&hash);
    }

    Ok((main_proof, merkle_statements, fri_merkle_statements))
}
