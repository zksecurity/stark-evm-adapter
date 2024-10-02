use ethers::{
    contract::ContractError,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    types::{Address, U64},
    utils::{hex, Anvil},
};
use stark_evm_adapter::{
    annotated_proof::AnnotatedProof,
    annotation_parser::{split_fri_merkle_statements, SplitProofs},
    oods_statement::FactTopology,
    ContractFunctionCall,
};
use std::{convert::TryFrom, env, fs::read_to_string, str::FromStr, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fork_url = env::var("FORK_URL");
    let url = env::var("URL");
    // check either env FORK_URL or URL is set
    if url.is_err() && fork_url.is_err() {
        panic!(
            "Either URL or FORK_URL must be set in env. \
        Set FORK_URL to run locally using an Anvil instance, \
        or set URL to submit tx on-chain (or run on a forked instance hosted on https://tenderly.co/)"
        );
    }

    let mut anvil = None;

    let provider: Provider<Http> = if url.is_ok() {
        Provider::try_from(url.unwrap().as_str())?
    } else {
        let url = fork_url.unwrap();
        anvil = Some(Anvil::new().fork(url).block_time(1u8).spawn());
        let endpoint = anvil.as_ref().unwrap().endpoint();
        Provider::<Http>::try_from(endpoint.as_str())?
    };

    // a trick to make anvil process lives in the whole main function
    if anvil.is_some() {
        println!("Anvil is running.");
    }

    let private_key = env::var("PRIVATE_KEY").unwrap_or(
        "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d".to_string(),
    );
    let from_key_bytes = hex::decode(private_key.trim_start_matches("0x")).unwrap();

    let from_signing_key = SigningKey::from_bytes(from_key_bytes.as_slice().into()).unwrap();
    let from_wallet: LocalWallet = LocalWallet::from(from_signing_key);
    println!("Test wallet address: {:?}", from_wallet.address());

    let chain_id = provider.get_chainid().await?.as_u32();
    let signer: Arc<SignerMiddleware<_, _>> = Arc::new(SignerMiddleware::new(
        provider.clone(),
        from_wallet.with_chain_id(chain_id),
    ));

    // load annotated proof
    let origin_proof_file = read_to_string(env::var("ANNOTATED_PROOF")?)?;
    let annotated_proof: AnnotatedProof = serde_json::from_str(&origin_proof_file)?;
    // generate split proofs
    let split_proofs: SplitProofs = split_fri_merkle_statements(annotated_proof.clone()).unwrap();

    let topologies_file = read_to_string(env::var("FACT_TOPOLOGIES")?)?;
    let topology_json: serde_json::Value = serde_json::from_str(&topologies_file).unwrap();

    let fact_topologies: Vec<FactTopology> =
        serde_json::from_value(topology_json.get("fact_topologies").unwrap().clone()).unwrap();

    // start verifying all split proofs
    println!("Verifying trace decommitments:");
    let contract_address = Address::from_str("0x634dcf4f1421fc4d95a968a559a450ad0245804c").unwrap();
    for i in 0..split_proofs.merkle_statements.len() {
        let key = format!("Trace {}", i);
        let trace_merkle = split_proofs.merkle_statements.get(&key).unwrap();

        let call = trace_merkle.verify(contract_address, signer.clone());
        assert_call(call, &key).await?;
    }

    println!("Verifying FRI decommitments:");
    let contract_address = Address::from_str("0xdef8a3b280a54ee7ed4f72e1c7d6098ad8df44fb").unwrap();
    for (i, fri_statement) in split_proofs.fri_merkle_statements.iter().enumerate() {
        let call = fri_statement.verify(contract_address, signer.clone());

        assert_call(call, &format!("FRI statement: {}", i)).await?;
    }

    let (_, continuous_pages) = split_proofs.main_proof.memory_page_registration_args();

    let memory_fact_registry_address =
        Address::from_str("0x40864568f679c10ac9e72211500096a5130770fa").unwrap();

    for (index, page) in continuous_pages.iter().enumerate() {
        let register_continuous_pages_call =
            split_proofs.main_proof.register_continuous_memory_page(
                memory_fact_registry_address,
                signer.clone(),
                page.clone(),
            );

        let name = format!("register continuous page: {}", index);

        assert_call(register_continuous_pages_call, &name).await?;
    }

    println!("Verifying main proof:");
    let contract_address = Address::from_str("0xd51a3d50d4d2f99a345a66971e650eea064dd8df").unwrap();

    let task_metadata = split_proofs
        .main_proof
        .generate_tasks_metadata(true, fact_topologies)
        .unwrap();

    let call = split_proofs
        .main_proof
        .verify(contract_address, signer, task_metadata);

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
