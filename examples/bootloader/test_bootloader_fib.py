import json
import subprocess

from starkware.cairo.bootloaders.bootloader.objects import (BootloaderConfig,
                                                            BootloaderInput,
                                                            PlainPackedOutput)
from starkware.cairo.bootloaders.simple_bootloader.objects import RunProgramTask
from starkware.cairo.lang.compiler.program import Program

# Constants and parameters

# configs for bootloader input
TASK_PROGRAM_INPUT_PATH = "./fibonacci_input.json"
TASK_PROGRAM_COMPILED_PATH = "./fibonacci_compiled.json"
SIMPLE_BOOTLOADER_PROGRAM_HASH = 382450030162484995497251732956824096484321811411123989415157331925872358847 # on-chain hash
# Should hash to 2512868110374320373201527039528844198060791559490644211790716345994094747600
SUPPORTED_CAIRO_VERIFIER_PROGRAM_HASHES = [
    "0x97e831fcc22602fa025e89c9c6b7e7272686398de428136cf52f3f006a845e",
    "0x7b2acdc57670aff4eac1f72b41ef759f003c122ed6cece634b76581966eade2",
    "0x24a3890c0d0ee8f7dfed5d1f89e3991bbc1b20d506c0700b24977f16f4487",
    "0x49904b6ecb9e083a42a1f50eb336ecc7e7a7c3ce06aabea405847cf0e2c1b2",
    "0x64b111ddda7af6661f2d1e6255ad7576ce8281ec701b166f07debca3bd7a0eb",
    "0x29a7e7366aa18c837867443aed5975f55107a8fdb6f33c418b81463a4156abf",
    "0x1fbfa8a63b6197519c5fbbf3b9090b6fadea637c8afba051c7419fd1d3d7fb3",
    "0x7d9c440b45a189c29e5d544d5b3ed167d089e3dd21e154abede91f90afb35ca",
    "0x41e6fdf682dca5b1e1194a93da5312fe66c06f08550a62c9e27645ca3874483",
    "0x3b58154a414a8e66fb65b1f6f612cd4ca21d68815fb0c6252930d4ddb04c72c",
    "0x2c47af88d90c4acd90fa663713e02a1f0a8b1239882d2f6b58dc964529540c9",
    "0x571ed7fb8805802da530fcac931794462cb7909479ec0ffd24766913a88636c",
    "0x6ad5606d7e4e7bc01b38a36d3fdca7afcf24d5db25ed4d050a4e77c31c5527b",
    "0x323c8251dbd935f45105bc241765a5082d9a985cbc3bffece3382708fb88dc5"
]
SUPPORTED_CAIRO_VERIFIER_PROGRAM_HASHES = [int(x, 16) for x in SUPPORTED_CAIRO_VERIFIER_PROGRAM_HASHES]
BOOTLOADER_INPUT_PATH = "./gen/bootloader_input.json"
FACT_TOPOLOGIES_PATH = "./gen/fact_topologies.json"

# configs for cairo-run
CAIRO_RUN_PATH = "./starkware/cairo/lang/scripts/cairo-run"
BOOTLOADER_PROGRAM_PATH = "./test_compiled_bootloader.json"
MEMORY_FILE_PATH = "./gen/memory.bin"
TRACE_FILE_PATH = "./gen/trace.bin"
PUBLIC_INPUT_PATH = "./gen/public_input.json"
PRIVATE_INPUT_PATH = "./gen/private_input.json"
LAYOUT = "starknet"

# configs for generating proof and annotations
STONE_PROVER_PATH = "./stone-prover/cpu_air_prover"
STONE_VERIFIER_PATH = "./stone-prover/cpu_air_verifier"
PROOF_OUTPUT_PATH = "./gen/bootloader_proof.json"
PROVER_CONFIG_PATH = "./cpu_air_prover_config.json"
PARAMETER_PATH = "./cpu_air_params.json"
ANNOTATION_FILE_PATH = "./gen/bootloader_proof_annotation.txt"
EXTRA_ANNOTATION_FILE_PATH = "./gen/bootloader_proof_annotation_extra.txt"

# configs for generating annotated proof
STARK_EVM_ADAPTER = "stark_evm_adapter"
ANNOTATED_PROOF_PATH = "./gen/annotated_proof.json"

def main():
    task_program_input = json.load(open(TASK_PROGRAM_INPUT_PATH, "r"))
    task_program = Program.loads(open(TASK_PROGRAM_COMPILED_PATH, "r").read())

    bootloader_input = BootloaderInput(
        bootloader_config=BootloaderConfig(
            # These are hashes that the cairo verifier contracts initialized with. https://github.com/starkware-libs/starkex-contracts/blob/f81ba5fdbd68516db50ea9679de9d0ac2f8049d8/evm-verifier/solidity/contracts/gps/GpsStatementVerifier.sol#L47-L48
            # The verifier checks these values as part of the public memory verification process. https://github.com/starkware-libs/starkex-contracts/blob/f81ba5fdbd68516db50ea9679de9d0ac2f8049d8/evm-verifier/solidity/contracts/gps/GpsStatementVerifier.sol#L263
            simple_bootloader_program_hash=SIMPLE_BOOTLOADER_PROGRAM_HASH,
            supported_cairo_verifier_program_hashes=SUPPORTED_CAIRO_VERIFIER_PROGRAM_HASHES,
        ),
        packed_outputs=[PlainPackedOutput()],
        # Specify what program tasks to run via the bootloader program.
        # In this demo, it only runs a single fibonacci program. But it allows to run many program tasks in a batch.
        tasks=[
            RunProgramTask(
                program=task_program,
                program_input=task_program_input,
                use_poseidon=False,
            ),
        ],
        # After the bootloader program runs the tasks, it will generate a fact topology.
        # Fact topology is a list of fact paths that the verifier will use as a part of the public memory verification process.
        # These are basically the outputs of the tasks that the bootloader program ran.
        # To facilitate the verification, these outputs are structured as a merkle tree, which I think is why it's called a fact topology.
        fact_topologies_path=FACT_TOPOLOGIES_PATH,
        single_page=False,
    )

    # Dump the bootloader input to a json file.
    with open(BOOTLOADER_INPUT_PATH, "w") as f:
        json.dump(bootloader_input.dump(), f)


    print("Running bootloader cairo program...")
    # Run the bootloader cairo program, and generate
    res = subprocess.run(
        [
            # "/home/kata/.local/bin/cairo-run",
            "python3",
            CAIRO_RUN_PATH,
            # path to the bootloader program.
            # it is the result of compiling the bootloader cairo program.
            # for example: 
            # cairo-compile 'cairo-lang/src/starkware/cairo/bootloaders/bootloader/bootloader.cairo' --proof_mode --output example/test_compiled_bootloader.json
            f"--program={BOOTLOADER_PROGRAM_PATH}",
            # path to the bootloader input json file we created above.
            f"--program_input={BOOTLOADER_INPUT_PATH}",
            # path to save the memory file as the result of running this cairo program.
            f"--memory_file={MEMORY_FILE_PATH}",
            # path to save the trace file as the result of running this cairo program.
            f"--trace_file={TRACE_FILE_PATH}",
            # path to save public memory inputs.
            f"--air_public_input={PUBLIC_INPUT_PATH}",
            # path to save private inputs.
            f"--air_private_input={PRIVATE_INPUT_PATH}",
            # layout of the cairo program.
            # Different layouts have different optimizations. https://github.com/starkware-libs/starkex-contracts/blob/f4ed79bb04b56d587618c24312e87d81e4efc56b/evm-verifier/solidity/contracts/cpu/layout0/LayoutSpecific.sol#L24
            # The bootloader program needs to be compiled to starknet layout in order to be verified by the starknet verifier on L1.
            f"--layout={LAYOUT}",
            "--print_info",
            "--proof_mode",
            "--print_output"
        ],
        capture_output=True,
    )
    if res.returncode != 0:
        print("Error running bootloader cairo program:")
        print(res.stderr.decode())
        exit(1)
    print(res.stdout.decode())

    print("Generating bootloader proof...")
    # Generate a bootloader proof by stone-prover with the outputs of the bootloader program execution above.
    res = subprocess.run(
        [
            STONE_PROVER_PATH,
            f"--out_file={PROOF_OUTPUT_PATH}",
            f"--private_input_file={PRIVATE_INPUT_PATH}",
            f"--public_input_file={PUBLIC_INPUT_PATH}",
            f"--prover_config_file={PROVER_CONFIG_PATH}",
            f"--parameter_file={PARAMETER_PATH}",
        ],
        capture_output=True,
    )
    if res.returncode != 0:
        print("Error generating bootloader proof:")
        print(res.stderr.decode())
        exit(1)
    print(res.stdout.decode())

    print("Generating annotations for the bootloader proof...")
    # Generate the annotations for the bootloader proof and merge them into a single json file. 
    # This single json file is called the annotated proof, which will be split into multiple proofs by the stark-evm-adapter.
    # These split proofs are then submitted to the starknet verifier contract on L1.
    # https://zksecurity.github.io/stark-book/starkex/proof-splitter.html
    res = subprocess.run(
        [
            STONE_VERIFIER_PATH,
            f"--in_file={PROOF_OUTPUT_PATH}",
            f"--annotation_file={ANNOTATION_FILE_PATH}",
            f"--extra_output_file={EXTRA_ANNOTATION_FILE_PATH}",
        ],
        capture_output=True,
    )
    if res.returncode != 0:
        print("Error generating annotations for the bootloader proof:")
        print(res.stderr.decode())
        exit(1)
    print(res.stdout.decode())

    print("Generating annotated proof...")
    res = subprocess.run(
        [
            STARK_EVM_ADAPTER,
            "gen-annotated-proof",
            f"--stone-proof-file={PROOF_OUTPUT_PATH}",
            f"--stone-annotation-file={ANNOTATION_FILE_PATH}",
            f"--stone-extra-annotation-file={EXTRA_ANNOTATION_FILE_PATH}",
            f"--output={ANNOTATED_PROOF_PATH}",
        ],
        capture_output=True,
    )
    if res.returncode != 0:
        print("Error generating annotated proof:")
        print(res.stderr.decode())
        exit(1)
    print(res.stdout.decode())

main()
