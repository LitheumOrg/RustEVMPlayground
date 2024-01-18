#!/bin/bash

# Directory containing the .sol files
SOL_DIR="./contracts"
# Directory where the build files will be stored
BUILD_DIR="./build/contracts"

# Ensure the build directory exists
mkdir -p "$BUILD_DIR"

# Loop through all .sol files in the specified directory
for sol_file in "$SOL_DIR"/*.sol; do
    # Extract the base name of the file
    base_name=$(basename "$sol_file" .sol)

    # Create a directory for the contract if it doesn't exist
    contract_dir="$BUILD_DIR/$base_name"
    mkdir -p "$contract_dir"

    # Compile the contract to get the ABI and bytecode
    solc --abi --bin "$sol_file" -o "$contract_dir" --overwrite

    # Rename the output files to have the same base name as the .sol file
    # Note: This assumes there's only one contract per .sol file or takes the first contract in the file
    for file in "$contract_dir"/*; do
        if [[ $file == *".abi" ]]; then
            mv "$file" "$contract_dir/${base_name}_ABI.json"
        elif [[ $file == *".bin" ]]; then
            mv "$file" "$contract_dir/${base_name}_Bytecode.bin"
        fi
    done
done

echo "Compilation finished."
