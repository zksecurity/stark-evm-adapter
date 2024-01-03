use std::{collections::HashMap, sync::Arc};

use ethers::{
    abi::Token,
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, U256},
    utils::keccak256,
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

abigen!(
    GpsStatementVerifierContract,
    r#"[
        function verifyProofAndRegister(uint256[] proof_params,uint256[] proof,uint256[] task_metadata,uint256[] cairo_aux_input,uint256 cairo_verifier_id)
    ]"#,
    derives(serde::Deserialize, serde::Serialize)
);

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
        for (_, page_prod) in page_prods {
            result.push(page_prod);
        }

        result
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
