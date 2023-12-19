cargo run --bin stark_evm_adapter gen-annotated-proof \
--stone-proof-file /home/kata/zksecurity/stark/stone-prover/e2e_test/4th_bootloader_proof.json \
--stone-annotation-file /home/kata/zksecurity/stark/stone-prover/e2e_test/4th_bootloader_proof_annotation.json \
--stone-extra-annotation-file /home/kata/zksecurity/stark/stone-prover/e2e_test/4th_bootloader_proof_annotation_extra.json \
--output ./tests/fixtures/4th_bootloader_proof_annotated.json

cargo run --bin stark_evm_adapter split-proof \
--annotated-proof-file ./tests/fixtures/4th_bootloader_proof_annotated.json \
--output ./tests/fixtures/4th_bootloader_split_proofs.json