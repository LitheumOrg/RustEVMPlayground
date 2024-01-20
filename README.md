# Rust EVM Playground

A playground for taking Rust EVM for a spin.

Requires solc.

Currently only supports string, uint, address, bool, and bytes32 types on contracts. May break for bytesXX other than 32.

### Usage: 

1) Put contracts into the contracts folder
2) cargo run
   
Contract build file and abi will be placed in build/contracts.