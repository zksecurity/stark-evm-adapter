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
FACT_TOPOLOGIES_PATH = "./fact_topologies.json"
BOOTLOADER_INPUT_PATH = "./bootloader_input.json"
SIMPLE_BOOTLOADER_PROGRAM_HASH = 2962621603719000361370283216422448934312521782617806945663080079725495842070
SUPPORTED_CAIRO_VERIFIER_PROGRAM_HASHES = [3178097804922730583543126053422762895998573737925004508949311089390705597156]

# configs for cairo-run
# this should be updated to your local cairo-run executable
CAIRO_RUN_PATH = "../cairo-lang/src/starkware/cairo/lang/scripts/cairo-run"
BOOTLOADER_PROGRAM_PATH = "./test_compiled_bootloader.json"
MEMORY_FILE_PATH = "./memory.bin"
TRACE_FILE_PATH = "./trace.bin"
PUBLIC_INPUT_PATH = "./public_input.json"
PRIVATE_INPUT_PATH = "./private_input.json"
LAYOUT = "starknet"

# configs for generating proof and annotations
# this path should be updated to your local cpu_air_prover executable 
STONE_PROVER_PATH = "../stone-prover/cpu_air_prover"
# this path should be updated to your local cpu_air_verifier executable 
STONE_VERIFIER_PATH = "../stone-prover/cpu_air_verifier"
PROOF_OUTPUT_PATH = "./bootloader_proof.json"
PROVER_CONFIG_PATH = "./cpu_air_prover_config.json"
PARAMETER_PATH = "./cpu_air_params.json"
ANNOTATION_FILE_PATH = "./bootloader_proof_annotation.txt"
EXTRA_ANNOTATION_FILE_PATH = "./bootloader_proof_annotation_extra.txt"

# configs for generating aggregated proof
STARK_EVM_ADAPTER = "stark_evm_adapter"
AGGREGATED_PROOF_PATH = "./aggregated_proof.json"

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
    print(res.stdout.decode(), res.stderr)

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
    print(res.stdout.decode(), res.stderr)

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
    print(res.stdout.decode(), res.stderr)

    res = subprocess.run(
        [
            STARK_EVM_ADAPTER,
            "gen-annotated-proof",
            f"--stone-proof-file={PROOF_OUTPUT_PATH}",
            f"--stone-annotation-file={ANNOTATION_FILE_PATH}",
            f"--stone-extra-annotation-file={EXTRA_ANNOTATION_FILE_PATH}",
            f"--output={AGGREGATED_PROOF_PATH}",
        ],
        capture_output=True,
    )
    print(res.stdout.decode(), res.stderr)

main()
