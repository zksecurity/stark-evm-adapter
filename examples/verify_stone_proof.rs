use ethers::{
    contract::ContractError,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider, ProviderError},
    signers::{LocalWallet, Signer, Wallet},
    types::{Address, U256, U64},
    utils::{hex, Anvil},
};
use stark_evm_adapter::{
    annotated_proof::AnnotatedProof,
    annotation_parser::{split_fri_merkle_statements, SplitProofs},
    oods_statement::FactTopology,
    ContractFunctionCall,
};
use std::{convert::TryFrom, env, str::FromStr, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setup a local mainnet fork
    let url = env::var("MAINNET_RPC");
    let forked_url = env::var("FORKED_MAINNET_RPC");
    // check either env MAINNET_RPC or FORK_MAINNET_RPC is set
    if url.is_err() && forked_url.is_err() {
        panic!(
            "Either MAINNET_RPC or FORK_MAINNET_RPC must be set in env. \
        You can get a mainnet RPC url from https://infura.io/, \
        or forked mainnet RPC url from https://tenderly.co/"
        );
    }

    let mut anvil = None;

    let provider: Provider<Http> = if forked_url.is_ok() {
        Provider::try_from(forked_url.unwrap().as_str())?
    } else {
        let url = url.unwrap();
        anvil = Some(Anvil::new().fork(url).block_time(1u8).spawn());
        let endpoint = anvil.as_ref().unwrap().endpoint();
        Provider::<Http>::try_from(endpoint.as_str())?
    };

    // a trick to make anvil process lives in the whole main function
    if anvil.is_some() {
        println!("Anvil is running.");
    }

    // test private key from anvil node
    let from_key_bytes =
        hex::decode("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d").unwrap();

    let from_signing_key = SigningKey::from_bytes(from_key_bytes.as_slice().into()).unwrap();
    let from_wallet: LocalWallet = LocalWallet::from(from_signing_key);
    println!("Test wallet address: {:?}", from_wallet.address());

    let chain_id = provider.get_chainid().await?.as_u32();
    let signer: Arc<SignerMiddleware<_, _>> = Arc::new(SignerMiddleware::new(
        provider.clone(),
        from_wallet.with_chain_id(chain_id),
    ));

    // load annotated proof
    // let origin_proof_file = include_str!(concat!(
    //     env!("CARGO_MANIFEST_DIR"),
    //     "/tests/fixtures/new_annotated_proof.json"
    // ));
    // let annotated_proof: AnnotatedProof = serde_json::from_str(origin_proof_file).unwrap();
    // generate split proofs
    // let split_proofs: SplitProofs = split_fri_merkle_statements(annotated_proof.clone()).unwrap();

    let topologies_file = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/madara_fibonacci_proof_topologies.json"
    ));
    let topology_json: serde_json::Value = serde_json::from_str(topologies_file).unwrap();
    
    let fact_topologies: Vec<FactTopology> = serde_json::from_value(topology_json.get("fact_topologies").unwrap().clone()).unwrap();

    // split proof file from madara prover
    let split_proofs_file = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/madara_fibonacci_proof.json"
    ));
    let split_proofs_json: serde_json::Value = serde_json::from_str(split_proofs_file).unwrap();
    let split_proofs: SplitProofs = serde_json::from_value(split_proofs_json.get("split_proofs").unwrap().clone()).unwrap();

    // start verifying all split proofs
    println!("Verifying trace decommitments:");
    let contract_address = Address::from_str("0x5899Efea757E0Dbd6d114b3375C23D7540f65fa4").unwrap();
    for i in 0..split_proofs.merkle_statements.len() {
        let key = format!("Trace {}", i);
        let trace_merkle = split_proofs.merkle_statements.get(&key).unwrap();

        let call = trace_merkle.verify(contract_address, signer.clone());

        assert_call(call, &key).await?;
    }

    println!("Verifying FRI decommitments:");
    let contract_address = Address::from_str("0x3E6118DA317f7A433031F03bB71ab870d87dd2DD").unwrap();
    for (i, fri_statement) in split_proofs.fri_merkle_statements.iter().enumerate() {
        let call = fri_statement.verify(contract_address, signer.clone());

        assert_call(call, &format!("FRI statement: {}", i)).await?;
    }

    let (_, continuous_pages) = split_proofs.main_proof.memory_page_registration_args();

    let memory_fact_registry_address =
        Address::from_str("0xFD14567eaf9ba941cB8c8a94eEC14831ca7fD1b4").unwrap();

    for (index, page) in continuous_pages.iter().enumerate() {
        let register_continuous_pages_call = split_proofs.main_proof.register_continuous_memory_page(
            memory_fact_registry_address,
            signer.clone(),
            page.clone(),
        );

        let name = format!("register continuous page: {}", index);

        assert_call(register_continuous_pages_call, &name).await?;
    }

    println!("Verifying main proof:");
    let contract_address = Address::from_str("0x47312450B3Ac8b5b8e247a6bB6d523e7605bDb60").unwrap();

    let task_metadata = split_proofs
        .main_proof
        .generate_tasks_metadata(true, fact_topologies)
        .unwrap();

    println!("task_metadata: {:?}", task_metadata);

    let call = split_proofs
        .main_proof
        .verify(contract_address, signer, task_metadata);
        // .gas(U256::from(5_000_000));

    assert_call(call, "Main proof").await?;

    Ok(())
}

async fn assert_call(
    call: ContractFunctionCall,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match call.send().await {
        Ok(pending_tx) => match pending_tx.await {
            Ok(mined_tx) => {
                let tx_receipt = mined_tx.unwrap();
                if tx_receipt.status.unwrap_or_default() == U64::from(1) {
                    println!("Verified: {}", name);
                    Ok(())
                } else {
                    Err(format!("Transaction failed: {}, but did not revert.", name).into())
                }
            }
            Err(e) => Err(decode_revert_message(e.into()).into()),
        },
        Err(e) => {
            Err(decode_revert_message(e).into())
            // Err(e.into())
        }
    }
}

fn decode_revert_message(
    e: ContractError<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
) -> String {
    match e {
        ContractError::Revert(err) => {
            println!("Revert data: {:?}", err.0);
            err.to_string()
        }
        _ => format!("Transaction failed: {:?}", e),
    }
}
