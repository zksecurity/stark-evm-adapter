use ethers::{
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
    utils::hex,
};
use eyre::Result;
use stark_evm_adapter::annotation_parser::{MainProof, SplitProofs, AnnotatedProof};
use std::{convert::TryFrom, str::FromStr, sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Start a fresh local Anvil node to execute the transfer on.
    // let anvil = Anvil::new().fork("https://mainnet.infura.io/v3/ada64046a78844a1ba1365664f3126de").spawn();
    // let provider = Provider::<Http>::try_from(
    //     "https://rpc.tenderly.co/fork/978fdb5c-f69b-4514-95b3-b185a524511b",
    // )?;
    let provider = Provider::<Http>::try_from(
        "http://localhost:8545",
    )?;

    let block_number = provider.get_block_number().await?;
    println!("Latest Block Number: {:?}", block_number);

    // The private key as a hex string
    let from_key_bytes = hex::decode("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d").unwrap();
    // let from_key_bytes =
    //     hex::decode("0xc2f4d7c2d3da8977f1ef79447c2112c5619917cd5873fccacf485e690d72f233").unwrap();

    let from_signing_key = SigningKey::from_bytes(from_key_bytes.as_slice().into()).unwrap();

    // Create the wallet
    let from_wallet: LocalWallet = LocalWallet::from(from_signing_key);
    println!("From Wallet: {:?}", from_wallet);

    abigen!(
        MerkleStatementContract,
        r#"[
            function verifyMerkle(uint256[],uint256[],uint256,uint256)
        ]"#,
    );
    abigen!(
        FriStatementContract,
        r#"[
            function verifyFRI(uint256[],uint256[],uint256,uint256,uint256)
        ]"#,
    );
    abigen!(
        GpsStatementVerifierContract,
        r#"[
            function verifyProofAndRegister(uint256[],uint256[],uint256[],uint256[],uint256)
        ]"#,
    );

    // 2. Create the contract instance to let us call methods of the contract and let it sign
    // transactions with the sender wallet.
    let provider = Provider::<Http>::try_from(provider.url().to_string())?
        .interval(Duration::from_millis(10u64));
    let chain_id = provider.get_chainid().await?.as_u32();
    let signer = Arc::new(SignerMiddleware::new(
        provider,
        from_wallet.with_chain_id(chain_id),
    ));

    let split_proof_file = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/bootloader_split_proofs.json"
    ));
    let bootloader_split_proofs: SplitProofs = serde_json::from_str(split_proof_file).unwrap();

    println!("verifying merkle");
    let contract_address = Address::from_str("0x5899Efea757E0Dbd6d114b3375C23D7540f65fa4").unwrap();
    let contract = MerkleStatementContract::new(contract_address, signer.clone());
    for i in 0..bootloader_split_proofs.merkle_statements.len() {
        let key = format!("Trace {}", i);
        let trace_merkle = bootloader_split_proofs.merkle_statements.get(&key).unwrap();

        // let trace_merkle = bootloader_split_proofs.merkle_statements.get("Trace 0").unwrap();
        // println!("Bootloader split proofs: {:?}", trace_merkle.proof[0]);
        // println!("Trace merkle: {:?}", trace_merkle.proof);

        let tx = contract
            .verify_merkle(
                trace_merkle.proof.clone(),
                trace_merkle.merkle_queue.clone(),
                trace_merkle.merkle_height,
                trace_merkle.expected_root,
            )
            .gas(U256::from(21000000i64));

        // load split proof

        // println!("Tx: {:?}", tx.calldata());

        // let calldata: Bytes = tx.calldata().unwrap();
        // let decoded = VerifyMerkleCall::decode(&calldata)?;
        // println!("Decoded: {:?}", decoded);

        // get tx hash
        // let pending_tx = tx.send().await?;
        let pending_tx = match tx.send().await {
            Ok(pending_tx) => pending_tx,
            Err(err) => {
                if err.is_revert() {
                    println!(
                        "Execution failed: {:?}",
                        err.decode_revert::<String>().unwrap()
                    );
                } else {
                    println!("Other error: {:?}", err);
                }
                return Ok(());
            }
        };

        println!("pending tx: {:?}", pending_tx);

        let _mined_tx = pending_tx.await?;
        println!("Mined tx: {:?}", _mined_tx);
    }

    println!("verifying FRI");
    let contract_address = Address::from_str("0x3E6118DA317f7A433031F03bB71ab870d87dd2DD").unwrap();
    let contract = FriStatementContract::new(contract_address, signer.clone());
    for fri_statement in bootloader_split_proofs.fri_merkle_statements {
        let tx = contract
            .verify_fri(
                fri_statement.proof.clone(),
                fri_statement.fri_queue.clone(),
                fri_statement.evaluation_point,
                fri_statement.fri_step_size,
                fri_statement.expected_root,
            )
            .gas(U256::from(21000000i64));

        let pending_tx = match tx.send().await {
            Ok(pending_tx) => pending_tx,
            Err(err) => {
                if err.is_revert() {
                    println!(
                        "Execution failed: {:?}",
                        err.decode_revert::<String>().unwrap()
                    );
                } else {
                    println!("Other error: {:?}", err);
                }
                return Ok(());
            }
        };

        // println!("pending tx: {:?}", pending_tx);

        let _mined_tx = pending_tx.await?;
        println!("Mined tx: {:?}", _mined_tx);
    }

    println!("verifying main proof");
    let main_proof_file = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/bootloader_verify_proof_register_args.json"
    ));
    let bootloader_main_proof: MainProof = serde_json::from_str(main_proof_file).unwrap();

    let contract_address = Address::from_str("0x47312450B3Ac8b5b8e247a6bB6d523e7605bDb60").unwrap();
    // let contract_address = Address::from_str("0x6cB3EE90C50a38A0e4662bB7e7E6e40B91361BF6").unwrap();
    let contract = GpsStatementVerifierContract::new(contract_address, signer.clone());

    // load json file
    let proof_file = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/3rd_bootloader_proof_annotated.json"
    ));

    let annotated_proof: AnnotatedProof = serde_json::from_str(proof_file).unwrap();


    let tx = contract
        .verify_proof_and_register(
            bootloader_main_proof.proof_params,
            bootloader_split_proofs.main_proof,
            bootloader_main_proof.task_metadata,
            bootloader_main_proof.cairo_aux_input,
            U256::from(6),
            // bootloader_main_proof.cairo_verifier_id,
        )
        .gas(U256::from(21000000i64));

    let pending_tx = match tx.send().await {
        Ok(pending_tx) => pending_tx,
        Err(err) => {
            if err.is_revert() {
                println!(
                    "Execution failed: {:?}",
                    err.decode_revert::<String>().unwrap()
                );
            } else {
                println!("Other error: {:?}", err);
            }
            return Ok(());
        }
    };

    // println!("pending tx: {:?}", pending_tx);

    let _mined_tx = pending_tx.await?;
    println!("Mined tx: {:?}", _mined_tx);

    Ok(())
}
