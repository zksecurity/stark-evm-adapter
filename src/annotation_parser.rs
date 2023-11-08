use ethers::abi::Token;
use ethers::utils::keccak256;
use ethers::{types::U256, utils::hex};
use num_bigint::BigUint;
use num_traits::{Num, One};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::{HashMap, HashSet};

use crate::errors::ParseError;
use crate::fri_merkle_statement::FRIMerkleStatement;
use crate::merkle_statement::MerkleStatement;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnnotatedProof {
    // todo: update deserializer to rename this as main_proof while mapping to proof_hex from source proof file
    proof_hex: String,
    annotations: Vec<String>,
    extra_annotations: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MerkleLine {
    pub name: String,
    pub node: U256,
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
    pub point: String,
    pub annotation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FriExtras {
    pub values: Vec<FriLine>,
    pub inverses: Vec<FriXInvLine>,
}

pub type MerkleExtrasDict = HashMap<String, Vec<MerkleLine>>;

pub struct FriMerklesOriginal {
    pub merkle_originals: MerkleExtrasDict,
    pub merkle_commitments: HashMap<String, CommitmentLine>,
    pub fri_originals: HashMap<String, Vec<FriLine>>,
    pub eval_points: Vec<EvalPointLine>,
    pub fri_names: Vec<String>,
    pub original_proof: Vec<u8>,
    pub main_annotation: String,
    pub merkle_patches: HashSet<String>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct SplitProofs {
    #[serde_as(as = "serde_with::hex::Hex")]
    pub main_proof: Vec<u8>,
    pub merkle_statements: HashMap<String, MerkleStatement>,
    pub fri_merkle_statements: Vec<FRIMerkleStatement>,
}

// Parses hex strings and pads with zeros to make it 64 characters long
fn extract_hex(line: &str) -> Result<String, ParseError> {
    let re = Regex::new(r"\(0x([0-9a-f]+)\)")?;
    Ok(re
        .captures(line)
        .and_then(|cap| cap.get(1))
        .map_or_else(String::new, |m| format!("{:0>64}", m.as_str())))
}

fn is_merkle_line(line: &str) -> bool {
    line.contains("Decommitment") && line.contains("node") && line.contains("Hash")
}

fn parse_merkle_line(line: &str) -> Result<MerkleLine, ParseError> {
    let name = line
        .split('/')
        .last()
        .ok_or(ParseError::InvalidLineFormat)?
        .split(':')
        .next()
        .ok_or(ParseError::InvalidLineFormat)?
        .to_string();

    let node_str = line
        .split("node ")
        .nth(1)
        .ok_or(ParseError::InvalidLineFormat)?
        .split(':')
        .next()
        .ok_or(ParseError::InvalidLineFormat)?;

    let node: U256 = U256::from_dec_str(node_str)?;

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

fn parse_merkle_data_line(line: &str) -> Result<MerkleLine, ParseError> {
    let name = line
        .split('/')
        .last()
        .ok_or(ParseError::InvalidLineFormat)?
        .split(':')
        .next()
        .ok_or(ParseError::InvalidLineFormat)?
        .to_string();

    let node_str = line
        .split("element #")
        .nth(1)
        .ok_or(ParseError::InvalidLineFormat)?
        .split(':')
        .next()
        .ok_or(ParseError::InvalidLineFormat)?;

    let node: U256 = U256::from_dec_str(node_str)?;

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

/// Parses a FRI decommitment line
fn parse_fri_line(line: &str) -> Result<FriLine, ParseError> {
    let parts: Vec<&str> = line.split('/').collect();
    let name = parts
        .last()
        .ok_or(ParseError::InvalidLineFormat)?
        .split(':')
        .next()
        .ok_or(ParseError::InvalidLineFormat)?
        .to_string();

    let row_col_part = line
        .split(':')
        .nth_back(1)
        .ok_or(ParseError::InvalidLineFormat)?;
    let row_col: Vec<&str> = row_col_part.split(',').collect();
    if row_col.len() != 2 {
        return Err(ParseError::InvalidLineFormat);
    }
    let row = row_col[0]
        .split_whitespace()
        .nth(1)
        .ok_or(ParseError::InvalidLineFormat)?
        .parse::<usize>()?;
    let col = row_col[1]
        .split_whitespace()
        .nth(1)
        .ok_or(ParseError::InvalidLineFormat)?
        .parse::<usize>()?;

    let element = extract_hex(line)?;

    Ok(FriLine {
        name,
        row,
        col,
        element,
        annotation: line.to_string(),
    })
}

/// Checks if a line is a FRI xInv line
fn is_fri_xinv_line(line: &str) -> bool {
    line.contains("Decommitment") && line.contains("xInv") && line.contains("Field Element")
}

/// Parses a FRI xInv line
fn parse_fri_xinv_line(line: &str) -> Result<FriXInvLine, ParseError> {
    let parts: Vec<&str> = line.split('/').collect();
    let name = parts
        .last()
        .ok_or(ParseError::InvalidLineFormat)?
        .split(':')
        .next()
        .ok_or(ParseError::InvalidLineFormat)?
        .to_string();

    let index_str = line
        .split("index ")
        .nth(1)
        .ok_or(ParseError::InvalidLineFormat)?
        .split(':')
        .next()
        .ok_or(ParseError::InvalidLineFormat)?;

    let index = index_str
        .parse::<usize>()
        .map_err(|_| ParseError::InvalidLineFormat)?;

    let inv = extract_hex(line)?;

    Ok(FriXInvLine {
        name,
        index,
        inv,
        annotation: line.to_string(),
    })
}

/// Checks if a line is a commitment line
fn is_commitment_line(line: &str) -> bool {
    line.contains("Commitment") && line.contains("Hash")
}

/// Parses a commitment line
fn parse_commitment_line(
    line: &str,
    trace_commitment_counter: &mut usize,
) -> Result<(CommitmentLine, usize), ParseError> {
    let parts: Vec<&str> = line.split(':').collect();
    let path_parts: Vec<&str> = parts
        .get(2)
        .ok_or(ParseError::InvalidLineFormat)?
        .trim()
        .split('/')
        .collect();
    let name;
    let new_trace_commitment_counter;

    if path_parts.last() == Some(&"Commit on Trace") {
        name = format!("Trace {}", trace_commitment_counter);
        *trace_commitment_counter += 1;
        new_trace_commitment_counter = *trace_commitment_counter;
    } else if path_parts.get(path_parts.len().saturating_sub(2)) == Some(&"Commitment") {
        name = path_parts
            .last()
            .map(|s| s.to_string())
            .ok_or(ParseError::InvalidLineFormat)?;
        new_trace_commitment_counter = *trace_commitment_counter;
    } else {
        return Err(ParseError::InvalidLineFormat);
    }

    let digest = extract_hex(line)?;
    Ok((
        CommitmentLine {
            name,
            digest,
            annotation: line.to_string(),
        },
        new_trace_commitment_counter,
    ))
}

/// Check if a line is an evaluation point line
fn is_eval_point_line(line: &str) -> bool {
    line.contains("Evaluation point") && line.contains("Layer")
}

/// Parses an evaluation point line
fn parse_eval_point_line(line: &str) -> Result<EvalPointLine, ParseError> {
    let parts: Vec<&str> = line.split('/').collect();
    let name = parts
        .last()
        .ok_or(ParseError::InvalidLineFormat)?
        .split(':')
        .next()
        .ok_or(ParseError::InvalidLineFormat)?
        .to_string();

    let point = extract_hex(line)?;

    Ok(EvalPointLine {
        name,
        point,
        annotation: line.to_string(),
    })
}

/// Parses a proof annotation line to indices
fn line_to_indices(line: &str) -> Result<(usize, usize), ParseError> {
    if !line.starts_with("P->V[") {
        Ok((0, 0))
    } else {
        let indices_part = &line[5..line.find(']').ok_or(ParseError::InvalidLineFormat)?];
        let indices: Vec<&str> = indices_part.split(':').collect();
        if indices.len() != 2 {
            Err(ParseError::InvalidLineFormat)
        } else {
            let start = indices[0].parse::<usize>()?;
            let end = indices[1].parse::<usize>()?;
            Ok((start, end))
        }
    }
}

/// Function to generate a Merkle statement call
pub fn gen_merkle_statement_call(
    merkle_extras: Vec<MerkleLine>,
    merkle_original: Vec<MerkleLine>,
    merkle_commit: CommitmentLine,
) -> Result<MerkleStatement, ParseError> {
    let qs: Vec<&str> = merkle_extras.iter().map(|n| &n.name[..]).collect();
    let heights: Vec<usize> = merkle_extras
        .iter()
        .map(|mline| mline.node.bits() - 1)
        .collect();

    if !heights.iter().all(|&h| h == heights[0]) {
        return Err(ParseError::InvalidLineFormat);
    }

    let root = U256::from_str_radix(&merkle_commit.digest, 16)?;
    let merkle_queue_values: Vec<U256> = merkle_extras
        .iter()
        .map(|mline| Ok(U256::from_str_radix(&mline.digest, 16)?))
        .collect::<Result<Vec<U256>, ParseError>>()?;
    let proof: Vec<U256> = merkle_original
        .iter()
        .map(|mline| Ok(U256::from_str_radix(&mline.digest, 16)?))
        .collect::<Result<Vec<U256>, ParseError>>()?;
    let merkle_queue_indices: Vec<U256> = merkle_extras.iter().map(|mline| mline.node).collect();

    Ok(MerkleStatement::new(
        root,
        qs.len(),
        heights[0],
        merkle_queue_indices,
        merkle_queue_values,
        proof,
    ))
}

fn montgomery_encode(element: &str) -> Result<U256, ParseError> {
    let prime = BigUint::from_str_radix(
        "800000000000011000000000000000000000000000000000000000000000001",
        16,
    )?;
    let num = BigUint::from_str_radix(element, 16)?;

    let r = BigUint::one() << 256; // Use 2^256 as R
    let encoded: BigUint = (num * r) % prime; // this seems to lost the purpose of montgomery encoding which aims to avoid division

    Ok(U256::from_str_radix(&encoded.to_str_radix(10), 10)?)
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
) -> Result<FRIMerkleStatement, ParseError> {
    let root = U256::from_str_radix(&merkle_commitment.digest, 16)?;
    let eval_point = U256::from_str_radix(&evaluation_point.point, 16)?;

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
    let row_lens: Vec<usize> = rows_to_cols
        .values()
        .map(|v| v.iter().cloned().collect::<HashSet<_>>().len())
        .collect();
    assert_eq!(row_lens.iter().cloned().collect::<HashSet<_>>().len(), 1);

    let step_size = (row_lens[0] as f64).log2() as usize;
    let input_height = output_height + step_size;

    let input_layer_queries: Vec<U256> = fri_extras
        .inverses
        .iter()
        .map(|fline| U256::from(fline.index + (1 << input_height)))
        .collect();

    let output_layer_queries: Vec<U256> = merkle_extras.iter().map(|mline| mline.node).collect();

    let input_layer_values: Vec<U256> = fri_extras
        .values
        .iter()
        .map(|fline| montgomery_encode(&fline.element))
        .collect::<Result<Vec<U256>, ParseError>>()?;

    let output_layer_values: Vec<U256> = fri_extras_next
        .values
        .iter()
        .map(|fline| montgomery_encode(&fline.element))
        .collect::<Result<Vec<U256>, ParseError>>()?;

    let input_layer_inverses: Vec<U256> = fri_extras
        .inverses
        .iter()
        .map(|fline| Ok(U256::from_str_radix(&fline.inv, 16)?))
        .collect::<Result<Vec<U256>, ParseError>>()?;

    let output_layer_inverses: Vec<U256> = fri_extras_next
        .inverses
        .iter()
        .map(|fline| Ok(U256::from_str_radix(&fline.inv, 16)?))
        .collect::<Result<Vec<U256>, ParseError>>()?;

    let proof: Vec<U256> = fri_original
        .iter()
        .map(|fline| montgomery_encode(&fline.element))
        .chain(
            merkle_original
                .iter()
                .map(|mline| Ok(U256::from_str_radix(&mline.digest, 16)?)),
        )
        .collect::<Result<Vec<U256>, ParseError>>()?;

    let input_interleaved = interleave(
        input_layer_queries.clone(),
        input_layer_values.clone(),
        input_layer_inverses.clone(),
    );
    let output_interleaved = interleave(
        output_layer_queries.clone(),
        output_layer_values.clone(),
        output_layer_inverses.clone(),
    );

    Ok(FRIMerkleStatement {
        expected_root: root,
        evaluation_point: eval_point,
        fri_step_size: step_size,
        input_layer_queries,
        output_layer_queries,
        input_layer_values,
        output_layer_values,
        input_layer_inverses,
        output_layer_inverses,
        // todo: refactor these interleaved into this struct
        input_interleaved,
        output_interleaved,
        proof,
    })
}

fn parse_fri_merkles_extra(
    extra_annot_lines: Vec<&str>,
) -> Result<(MerkleExtrasDict, Vec<FriExtras>), ParseError> {
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
) -> Result<FriMerklesOriginal, ParseError> {
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
            let (cline, new_trace_commitment_counter) =
                parse_commitment_line(&line, &mut trace_commitment_counter)?;
            merkle_commits_dict.insert(cline.name.clone(), cline);
            trace_commitment_counter = new_trace_commitment_counter;
        } else if is_eval_point_line(&line) {
            let epline = parse_eval_point_line(&line)?;
            eval_points_list.push(epline);
        }

        if is_merkle_line(&line) {
            let mline = parse_merkle_line(&line)?;
            merkle_original_dict
                .entry(mline.name.clone())
                .or_default()
                .push(mline);
        } else if is_merkle_data_line(&line) {
            let mline = parse_merkle_data_line(&line)?;
            let cloned_mline_name = mline.name.clone();
            merkle_original_dict
                .entry(mline.name.clone())
                .or_default()
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
            let (start, end) = line_to_indices(&line)?;
            main_proof.extend_from_slice(&orig_proof[start..end]);
        }
    }

    Ok(FriMerklesOriginal {
        merkle_originals: merkle_original_dict,
        merkle_commitments: merkle_commits_dict,
        fri_originals: fri_original_dict,
        eval_points: eval_points_list,
        fri_names,
        original_proof: main_proof,
        main_annotation: main_annot,
        merkle_patches,
    })
}

fn single_column_merkle_patch(
    merkle_patches: &HashSet<String>,
    merkle_extras_dict: &mut HashMap<String, Vec<MerkleLine>>,
    annot_lines: &[String],
) -> Result<(), ParseError> {
    for name in merkle_patches {
        let merkle_extras = merkle_extras_dict
            .get(name)
            .ok_or(ParseError::InvalidLineFormat)?
            .clone();
        let heights: Vec<usize> = merkle_extras
            .iter()
            .map(|mline| mline.node.leading_zeros() as usize - 1)
            .collect();
        // Ensure all heights are the same
        let height = *heights.first().ok_or(ParseError::InvalidLineFormat)?;
        if !heights.iter().all(|&h| h == height) {
            return Err(ParseError::InvalidLineFormat);
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
                let element = montgomery_encode(&parsed_fri_line.element)?;
                let element_hex = format!("{:0>64x}", element);
                let merkle_line = MerkleLine {
                    name: name.clone(),
                    node,
                    digest: element_hex,
                    annotation: line.clone(),
                };
                merkle_extras_dict
                    .get_mut(name)
                    .ok_or(ParseError::InvalidLineFormat)?
                    .push(merkle_line);
            }
        }
    }
    Ok(())
}

/// This is the main function to use to split an [AnnotatedProof] file into a [SplitProofs] file.
pub fn split_fri_merkle_statements(proof_json: AnnotatedProof) -> Result<SplitProofs, ParseError> {
    // Decode the hexadecimal string
    let orig_proof = hex::decode(&proof_json.proof_hex)?;

    let annot_lines = proof_json.annotations;
    let extra_annot_lines = proof_json.extra_annotations;

    let (mut merkle_extras_dict, fri_extras_list) =
        parse_fri_merkles_extra(extra_annot_lines.iter().map(|s| s.as_str()).collect())?;
    let fri_merkles_original = parse_fri_merkles_original(orig_proof, annot_lines.clone())?;
    let merkle_names: HashSet<_> = HashSet::from_iter(merkle_extras_dict.keys().cloned());
    assert_eq!(
        merkle_names,
        HashSet::from_iter(fri_merkles_original.merkle_originals.keys().cloned())
    );

    if !fri_merkles_original.merkle_patches.is_empty() {
        single_column_merkle_patch(
            &fri_merkles_original.merkle_patches,
            &mut merkle_extras_dict,
            &annot_lines,
        )?;
    }

    let merkle_statements = merkle_names
        .into_iter()
        .filter(|name| !fri_merkles_original.fri_originals.contains_key(name))
        .map(|name| {
            let statement = gen_merkle_statement_call(
                merkle_extras_dict[&name].clone(),
                fri_merkles_original.merkle_originals[&name].clone(),
                fri_merkles_original.merkle_commitments[&name].clone(),
            )
            .unwrap();
            (name, statement)
        })
        .collect::<HashMap<_, _>>();

    let fri_merkle_statements: Vec<FRIMerkleStatement> = fri_merkles_original
        .fri_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            gen_fri_merkle_statement_call(
                fri_extras_list[i].clone(),
                fri_extras_list[i + 1].clone(),
                fri_merkles_original.fri_originals[&name].clone(),
                fri_merkles_original.merkle_originals[&name].clone(),
                merkle_extras_dict[&name].clone(),
                fri_merkles_original.merkle_commitments[&name].clone(),
                fri_merkles_original.eval_points[i].clone(),
            )
        })
        .collect::<Result<Vec<FRIMerkleStatement>, ParseError>>()?;

    let mut main_proof = fri_merkles_original.original_proof;

    for fri in &fri_merkle_statements[..fri_merkle_statements.len() - 1] {
        let fri_output_interleaved = fri
            .output_interleaved
            .iter()
            .map(|val| Token::Uint(*val))
            .collect();

        let encoded = ethers::abi::encode_packed(&[Token::Array(fri_output_interleaved)])?;
        let hash = keccak256(encoded);
        main_proof.extend_from_slice(&hash);
    }

    Ok(SplitProofs {
        main_proof,
        merkle_statements,
        fri_merkle_statements,
    })
}
