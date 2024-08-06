use std::{collections::HashMap, sync::Arc};

use ethers::{
    abi::Token,
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, U256},
    utils::{hex, keccak256},
};
use num_bigint::BigInt;
use num_traits::{Num, One};
use serde::{Deserialize, Serialize};

use crate::{
    annotated_proof::{MemorySegment, ProofParameters, PublicInput, PublicMemory},
    default_prime, ContractFunctionCall,
};

/// Proof for consistency check for out of domain sampling
#[derive(Serialize, Deserialize, Debug)]
pub struct MainProof {
    pub proof: Vec<U256>,
    pub proof_parameters: ProofParameters,
    pub public_input: PublicInput,
    pub interaction_z: U256,
    pub interaction_alpha: U256,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FactTopology {
    tree_structure: Vec<u8>,
    page_sizes: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FactNode {
    node_hash: U256,
    end_offset: usize,
    size: usize,
    children: Vec<FactNode>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegularMemoryPage {
    page: Vec<U256>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContinuousMemoryPage {
    start_address: U256,
    values: Vec<U256>,
}

abigen!(
    GpsStatementVerifierContract,
    r#"[
        function verifyProofAndRegister(uint256[] proof_params,uint256[] proof,uint256[] task_metadata,uint256[] cairo_aux_input,uint256 cairo_verifier_id)
    ]"#,
    derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    MemoryPageFactRegistryContract,
    r#"[
        function registerContinuousMemoryPage(uint256 startAddr,uint256[] values,uint256 z,uint256 alpha,uint256 prime)
    ]"#,
    derives(serde::Deserialize, serde::Serialize)
);

// todo use thiserror
impl MainProof {
    pub fn new(
        proof: Vec<U256>,
        proof_parameters: ProofParameters,
        public_input: PublicInput,
        interaction_z: U256,
        interaction_alpha: U256,
    ) -> MainProof {
        MainProof {
            proof,
            proof_parameters,
            public_input,
            interaction_z,
            interaction_alpha,
        }
    }

    /// Serialize proof parameters
    fn proof_params(&self) -> Vec<U256> {
        let blow_up_factor = self.proof_parameters.stark.log_n_cosets;
        let pow_bits = self.proof_parameters.stark.fri.proof_of_work_bits;
        let n_queries = self.proof_parameters.stark.fri.n_queries;

        let mut proof_params: Vec<U256> = Vec::new();
        proof_params.push(U256::from(n_queries));
        proof_params.push(U256::from(blow_up_factor));
        proof_params.push(U256::from(pow_bits));

        let last_layer_degree_bound = self.proof_parameters.stark.fri.last_layer_degree_bound;
        let ceil_log2 = (last_layer_degree_bound as f64).log2().ceil() as u32;
        proof_params.push(U256::from(ceil_log2));

        let fri_step_list_len = U256::from(self.proof_parameters.stark.fri.fri_step_list.len());
        proof_params.push(fri_step_list_len);

        let fri_step_list: Vec<U256> = self
            .proof_parameters
            .stark
            .fri
            .fri_step_list
            .iter()
            .map(|&x| U256::from(x))
            .collect();
        proof_params.extend_from_slice(&fri_step_list);

        proof_params
    }

    /// Collect and serialize cairo public input
    fn cairo_aux_input(&self) -> Vec<U256> {
        let log_n_steps = (self.public_input.n_steps as f64).log2() as u64;
        let mut cairo_aux_input = vec![
            U256::from(log_n_steps),
            U256::from(self.public_input.rc_min),
            U256::from(self.public_input.rc_max),
        ];

        // Encoding the 'layout' string to its ASCII byte representation and converting to U256
        let layout_big = U256::from_big_endian(self.public_input.layout.as_bytes());
        cairo_aux_input.push(layout_big);

        // Extend with serialized segments
        let serialized_segments = self.serialize_segments();
        cairo_aux_input.extend(serialized_segments);

        let z = self.interaction_z;
        let alpha = self.interaction_alpha;

        let memory_pages_public_input =
            self.memory_page_public_input(self.public_input.public_memory.clone(), z, alpha);

        // Extend with memory pages public input - assuming this is already a Vec<U256>
        cairo_aux_input.extend(memory_pages_public_input);

        // Append z and alpha
        cairo_aux_input.push(z);
        cairo_aux_input.push(alpha);

        cairo_aux_input
    }

    /// Serialize memory segments in order
    fn serialize_segments(&self) -> Vec<U256> {
        let segment_names = [
            "program",
            "execution",
            "output",
            "pedersen",
            "range_check",
            "ecdsa",
            "bitwise",
            "ec_op",
            "keccak",
            "poseidon",
        ];

        let segments = &self.public_input.memory_segments;
        let mut sorted_segments: Vec<MemorySegment> = Vec::new();

        for name in segment_names.iter() {
            let segment: Option<&MemorySegment> = segments.get(*name);
            if let Some(seg) = segment {
                sorted_segments.push(seg.clone());
            }
        }

        assert_eq!(sorted_segments.len(), segments.len());

        let mut result: Vec<U256> = Vec::new();
        for segment in sorted_segments {
            result.push(U256::from(segment.begin_addr));
            result.push(U256::from(segment.stop_ptr));
        }

        result
    }

    /// Calculate accumulated product for continuous memory
    fn calculate_product(
        prod: U256,
        z: U256,
        alpha: U256,
        memory_address: U256,
        memory_value: U256,
        prime: U256,
    ) -> U256 {
        let bigint_prod = BigInt::from_str_radix(&prod.to_string(), 10).unwrap();
        let bigint_alpha = BigInt::from_str_radix(&alpha.to_string(), 10).unwrap();
        let bigint_z = BigInt::from_str_radix(&z.to_string(), 10).unwrap();
        let bigint_memory_value = BigInt::from_str_radix(&memory_value.to_string(), 10).unwrap();
        let bigint_memory_address =
            BigInt::from_str_radix(&memory_address.to_string(), 10).unwrap();
        let bigint_prime = BigInt::from_str_radix(&prime.to_string(), 10).unwrap();

        let multiply =
            bigint_prod * (bigint_z - (bigint_memory_address + bigint_alpha * bigint_memory_value));
        let mod_multiply = multiply.modpow(&BigInt::one(), &bigint_prime);
        U256::from_dec_str(&mod_multiply.to_string()).unwrap()
    }

    /// Calculate accomulative product for each memory page
    fn get_pages_and_products(
        &self,
        public_memory: Vec<PublicMemory>,
        z: U256,
        alpha: U256,
    ) -> (HashMap<u32, Vec<U256>>, HashMap<u32, U256>) {
        let mut pages: HashMap<u32, Vec<U256>> = HashMap::new();
        let mut page_prods: HashMap<u32, U256> = HashMap::new();

        for cell in public_memory {
            let page = pages.entry(cell.page).or_default();
            let memory_address = U256::from(cell.address);
            let memory_value = U256::from_str_radix(&cell.value, 16).unwrap();
            page.push(memory_address);
            page.push(memory_value);

            let prod = page_prods.entry(cell.page).or_insert(U256::one());

            *prod = Self::calculate_product(
                *prod,
                z,
                alpha,
                memory_address,
                memory_value,
                default_prime(),
            );
        }

        (pages, page_prods)
    }

    /// Construct contract args for public input of memory pages
    fn memory_page_public_input(
        &self,
        public_memory: Vec<PublicMemory>,
        z: U256,
        alpha: U256,
    ) -> Vec<U256> {
        let mut result: Vec<U256> = Vec::new();

        // Get pages and page_prods
        let (pages, page_prods) = self.get_pages_and_products(public_memory.clone(), z, alpha);

        // Append padding values for public memory
        let padding_cell = &public_memory[0];
        let memory_address = U256::from(padding_cell.address);
        let memory_value = U256::from_str_radix(&padding_cell.value, 16).unwrap();
        result.push(memory_address);
        result.push(memory_value);

        result.push(U256::from(pages.len()));

        for i in 0..pages.len() {
            let page = pages.get(&(i as u32)).unwrap();
            let page_hash = if i == 0 {
                let tokens: Vec<Token> = page.iter().map(|val| Token::Uint(*val)).collect();
                let encoded = ethers::abi::encode_packed(&[Token::Array(tokens)]).unwrap();
                U256::from(keccak256(encoded.as_slice()).as_slice())
            } else {
                // Verify that the addresses of the page are indeed continuous
                let range: Vec<U256> = (0..page.len() as u64 / 2)
                    .map(|i| page[0] + U256::from(i))
                    .collect();
                assert!(page.iter().step_by(2).eq(range.iter()));
                result.push(page[0]); // First address

                let tokens: Vec<Token> = page
                    .iter()
                    .skip(1)
                    .step_by(2)
                    .map(|val| Token::Uint(*val))
                    .collect();
                let encoded = ethers::abi::encode_packed(&[Token::Array(tokens)]).unwrap();
                U256::from(keccak256(encoded.as_slice()).as_slice())
            };

            result.push(U256::from(page.len() as u64 / 2)); // Page size
            result.push(page_hash); // Page hash
        }

        // Append the products of the pages
        // Note: this assumes that the pages are ordered from 0 to n
        for index in 0..page_prods.len() {
            let page_prod = page_prods.get(&(index as u32)).unwrap();
            result.push(*page_prod);
        }

        result
    }

    pub fn memory_page_registration_args(&self) -> (RegularMemoryPage, Vec<ContinuousMemoryPage>) {
        let (pages, _) = self.get_pages_and_products(
            self.public_input.public_memory.clone(),
            self.interaction_z,
            self.interaction_alpha,
        );

        let regular_page = RegularMemoryPage {
            page: pages.get(&0).unwrap().clone(),
        };

        let continuous_pages: Vec<ContinuousMemoryPage> = (1..pages.len() as u32)
            .map(|i| ContinuousMemoryPage {
                start_address: pages.get(&i).unwrap()[0],
                values: pages
                    .get(&i)
                    .unwrap()
                    .iter()
                    .skip(1)
                    .step_by(2)
                    .cloned()
                    .collect(),
            })
            .collect();

        (regular_page, continuous_pages)
    }

    //todo use thiserror
    fn extract_public_memory(public_input: &PublicInput) -> Result<HashMap<u32, U256>, String> {
        let mut memory_map = HashMap::new();
        for entry in &public_input.public_memory {
            let addr = entry.address;
            let value = &entry.value;
            if memory_map.contains_key(&addr) {
                return Err(format!(
                    "Duplicate public memory entries found with the same address: {}",
                    addr
                ));
            }
            memory_map.insert(addr, U256::from_str_radix(value, 16).unwrap());
        }
        Ok(memory_map)
    }

    //todo use thiserror
    fn extract_program_output(
        public_input: &PublicInput,
        memory: &HashMap<u32, U256>,
    ) -> Result<Vec<U256>, String> {
        let output_segment = public_input
            .memory_segments
            .get("output")
            .ok_or("Missing output segment.")?;

        let stop_ptr = output_segment.stop_ptr;

        let mut output = Vec::new();
        for addr in output_segment.begin_addr..stop_ptr {
            let value = *memory
                .get(&addr)
                .ok_or(format!("Missing value for address: {}", addr))?;
            output.push(value);
        }
        Ok(output)
    }

    #[allow(dead_code)]
    fn get_trivial_topology(public_memory: &Vec<PublicMemory>) -> Vec<FactTopology> {
        let mut page_sizes = HashMap::new();

        for item in public_memory {
            *page_sizes.entry(item.page).or_insert(0) += 1;
        }

        // Ignore the main page (page 0)
        page_sizes.remove(&0);

        let mut topologies = Vec::new();
        for (i, _) in page_sizes.iter().enumerate() {
            let page_size = page_sizes.get(&(i as u32 + 1)).cloned().unwrap_or(0);
            topologies.push(FactTopology {
                tree_structure: vec![1, 0],
                page_sizes: vec![page_size],
            });
        }

        topologies
    }

    fn keccak_ints(&self, values: &[U256]) -> Result<String, String> {
        let values_bytes = values
            .iter()
            .flat_map(|&value| {
                let mut bytes = [0u8; 32]; // U256 is 32 bytes
                value.to_big_endian(&mut bytes);
                bytes
            })
            .collect::<Vec<u8>>();

        let result = keccak256(values_bytes.as_slice());
        // convert result to hex string
        Ok(hex::encode(result))
    }

    fn generate_output_root(
        &self,
        program_output: &[U256],
        fact_topology: &FactTopology,
    ) -> Result<FactNode, String> {
        let mut page_sizes = fact_topology.page_sizes.clone();
        let tree_structure = &fact_topology.tree_structure;
        let mut offset = 0;
        let mut node_stack: Vec<FactNode> = Vec::new();

        let mut tree_iter = tree_structure.iter();
        while let (Some(&n_pages), Some(&n_nodes)) = (tree_iter.next(), tree_iter.next()) {
            if n_pages as usize > page_sizes.len() {
                return Err("Invalid tree structure: n_pages is out of range.".to_string());
            }

            for _ in 0..n_pages {
                let page_size = page_sizes.remove(0);
                let page_hash = self.keccak_ints(&program_output[offset..offset + page_size])?;

                offset += page_size;
                node_stack.push(FactNode {
                    node_hash: U256::from_str_radix(&page_hash, 16).unwrap(),
                    end_offset: offset,
                    size: page_size,
                    children: Vec::new(),
                });
            }

            if n_nodes as usize > node_stack.len() {
                return Err("Invalid tree structure: n_nodes is out of range.".to_string());
            }

            if n_nodes > 0 {
                let child_nodes = node_stack.split_off(node_stack.len() - n_nodes as usize);
                let node_data: Vec<U256> = child_nodes
                    .iter()
                    .flat_map(|node| vec![node.node_hash, U256::from(node.end_offset)])
                    .collect();

                let data_hash = self.keccak_ints(&node_data)?;
                let node_hash = U256::one() + U256::from_str_radix(&data_hash, 16).unwrap();

                let end_offset = child_nodes.last().unwrap().end_offset;
                let size = child_nodes.iter().map(|node| node.size).sum();

                node_stack.push(FactNode {
                    node_hash,
                    end_offset,
                    size,
                    children: child_nodes,
                });
            }
        }

        if node_stack.len() != 1 {
            return Err("Invalid tree structure: stack contains more than one node.".to_string());
        }
        if !page_sizes.is_empty() {
            return Err("Invalid tree structure: not all pages were processed.".to_string());
        }
        if offset != node_stack[0].end_offset || offset != program_output.len() {
            return Err("Invalid tree structure: offset mismatch.".to_string());
        }

        Ok(node_stack.pop().unwrap())
    }

    fn generate_program_fact(
        &self,
        program_hash: U256,
        program_output: Vec<U256>,
        fact_topology: &FactTopology,
    ) -> Result<String, String> {
        let output_root_node = self.generate_output_root(&program_output, fact_topology)?;
        let hash = self.keccak_ints(&[program_hash, output_root_node.node_hash])?;
        Ok(hash)
    }

    pub fn generate_tasks_metadata(
        &self,
        include_bootloader_config: bool,
        fact_topologies: Vec<FactTopology>,
    ) -> Result<Vec<U256>, String> {
        let bootloader_config_size = 2;
        let program_output_header = 2;
        let n_programs_entry = if include_bootloader_config {
            bootloader_config_size
        } else {
            0
        };
        let memory = Self::extract_public_memory(&self.public_input)?;
        let mut output = Self::extract_program_output(&self.public_input, &memory)?;

        let n_programs: usize = output
            .get(n_programs_entry)
            .ok_or("n_programs_entry index out of range")?
            .as_usize();

        if n_programs * program_output_header >= output.len() {
            return Err("output_length is too short.".to_string());
        }

        if include_bootloader_config {
            output = output[bootloader_config_size..].to_vec();
        }

        let n_tasks = output[0];
        let mut task_metadata = vec![n_tasks];
        let mut facts = vec![];
        let mut task_outputs = vec![];
        let mut expected_page_sizes = vec![];
        let mut ptr = 1;

        for fact_topology in fact_topologies {
            if ptr + 1 >= output.len() {
                return Err("Output index out of bounds.".to_string());
            }
            let task_output_size = output[ptr];
            let program_hash = output[ptr + 1];
            task_metadata.push(task_output_size);
            task_metadata.push(program_hash);
            task_metadata.push(U256::from(fact_topology.tree_structure.len()) / U256::from(2));
            task_metadata.extend_from_slice(
                &fact_topology
                    .tree_structure
                    .iter()
                    .map(|&x| U256::from(x))
                    .collect::<Vec<U256>>(),
            );

            let end = ptr + task_output_size.as_usize();
            if end > output.len() {
                return Err("Task output size exceeds output length.".to_string());
            }
            let task_output = &output[ptr + 2..end];
            task_outputs.push(task_output.to_vec());

            let fact =
                self.generate_program_fact(program_hash, task_output.to_vec(), &fact_topology)?;
            facts.push(fact);
            ptr += task_output_size.as_usize();

            if task_output_size.as_usize() != 2 + fact_topology.page_sizes.iter().sum::<usize>() {
                return Err("Page sizes do not match the task output size.".to_string());
            }

            expected_page_sizes.extend_from_slice(&fact_topology.page_sizes);
        }

        if ptr != output.len() {
            return Err(format!(
                "Not all of the bootloader output was processed: {} != {}",
                ptr,
                output.len()
            ));
        }

        Ok(task_metadata)
    }

    /// Construct `verifyProofAndRegister` contract call
    pub fn register_continuous_memory_page_call(
        &self,
        continuous_page: ContinuousMemoryPage,
    ) -> RegisterContinuousMemoryPageCall {
        RegisterContinuousMemoryPageCall {
            start_addr: continuous_page.start_address,
            values: continuous_page.values,
            z: self.interaction_z,
            alpha: self.interaction_alpha,
            prime: default_prime(),
        }
    }

    /// Initiate `verifyProofAndRegister` contract call
    pub fn register_continuous_memory_page(
        &self,
        address: Address,
        signer: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        continuous_page: ContinuousMemoryPage,
    ) -> ContractFunctionCall {
        let contract = MemoryPageFactRegistryContract::new(address, signer);

        let function_call = self.register_continuous_memory_page_call(continuous_page);
        contract
            .method("registerContinuousMemoryPage", function_call)
            .unwrap()
    }

    /// Construct `verifyProofAndRegister` contract call
    pub fn contract_function_call(&self, task_metadata: Vec<U256>) -> VerifyProofAndRegisterCall {
        VerifyProofAndRegisterCall {
            proof_params: self.proof_params(),
            proof: self.proof.clone(),
            task_metadata,
            cairo_aux_input: self.cairo_aux_input(),
            cairo_verifier_id: U256::from(6),
        }
    }

    /// Initiate `verifyProofAndRegister` contract call
    pub fn verify(
        &self,
        address: Address,
        signer: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        task_metadata: Vec<U256>,
    ) -> ContractFunctionCall {
        let contract = GpsStatementVerifierContract::new(address, signer);

        let function_call = self.contract_function_call(task_metadata);
        contract
            .method("verifyProofAndRegister", function_call)
            .unwrap()
    }
}
