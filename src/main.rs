use dialoguer::Input;
use ethereum_types::{H160, U256};
use evm::backend::MemoryBackend;
use evm::executor::stack::{StackExecutor, MemoryStackState, StackSubstateMetadata};
use evm::ExitReason;
use rlp::RlpStream;
use sha3::{Digest, Keccak256};

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};
use std::io;

use ethabi::Contract;
use ethabi::param_type::ParamType;
use std::path::PathBuf;

struct ContractData {
    address: Option<H160>,
    abi: Contract,
}

struct Account {
    address: H160,
    nonce: U256,
}
type ContractsData = HashMap<String, ContractData>;

fn parse_abi(abi_path: &str) -> Result<Contract, io::Error> {
    let path = PathBuf::from(abi_path);
    let file = fs::File::open(path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?; // Convert std::io::Error to io::Error if needed

    let contract = Contract::load(file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?; // Convert ethabi::Error to io::Error

    Ok(contract)
}

fn compute_contract_address(sender: H160, nonce: U256) -> H160 {
    let mut stream = RlpStream::new_list(2);
    stream.append(&sender);
    stream.append(&nonce);
    let hash = Keccak256::digest(&stream.out());
    H160::from_slice(&hash[12..])
}

fn collect_contract_names(contracts_dir: &str) -> Result<Vec<String>, io::Error> {

    let mut contract_names = Vec::new();

    // Check if the directory exists
    if !Path::new(contracts_dir).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Contracts directory does not exist."));
    }

    // Read the contents of the directory
    let entries = fs::read_dir(contracts_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Check if the entry is a file and has a .sol extension
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sol") {
            // Get the file name as a string
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                contract_names.push(file_name.to_string());
            }
        }
    }

    Ok(contract_names)
}

fn choose_contract(contracts: &HashMap<String, ContractData>) -> Result<String, io::Error> {
    println!("\nAvailable contracts:");
    for (i, name) in contracts.keys().enumerate() {
        println!("{}: {}", i + 1, name); // Display index starting from 1
    }

    let chosen_index: usize = Input::new()
    .with_prompt("Choose a contract by number")
    .interact_text()
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    if let Some(chosen_name) = contracts.keys().nth(chosen_index - 1) { // Subtract 1 to get the correct index
        Ok(chosen_name.to_string())
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid contract number"))
    }
}

fn choose_function(abi: &ethabi::Contract) -> Result<(String, bool, Vec<ParamType>), io::Error> {
    
    // Collect function names, check if they are getters, and get their return types
    let functions_info: Vec<(String, bool, Vec<ParamType>)> = abi.functions.iter()
        .map(|(name, functions)| {
            let is_getter = functions.iter().any(|function| {
                function.state_mutability == ethabi::StateMutability::View || function.state_mutability == ethabi::StateMutability::Pure
            });
            let return_types = functions.iter()
                .flat_map(|function| function.outputs.clone())
                .map(|output| output.kind)
                .collect();
            (name.clone(), is_getter, return_types)
        })
        .collect();

    
    for (i, (name, is_getter, return_types)) in functions_info.iter().enumerate() {
        let getter_marker = if *is_getter { " (getter)" } else { "" };
        let return_types_str = return_types.iter()
            .map(|param_type| format!("{:?}", param_type))
            .collect::<Vec<_>>()
            .join(", ");
        println!("{}: {}{} -> [{}]", i + 1, name, getter_marker, return_types_str);
    }
    
    let chosen_index: usize = Input::new()
        .with_prompt("Choose a function by number")
        .interact_text()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;


    if let Some(chosen_name) = functions_info.get(chosen_index - 1) { // Subtract 1 to get the correct index
        Ok(chosen_name.clone())
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid function number"))
    }
}

fn call_contract_function<'a>(
    executor: &mut StackExecutor<'a, 'a, MemoryStackState<'a, 'a, MemoryBackend<'a>>, ()>,
    contract_data: &ContractData,
    function_name: &str,
    encoded_inputs: Vec<u8>,
    caller_address: H160,   // Caller's address
) -> Result<Vec<ethabi::Token>, io::Error> {
    
    let function = contract_data
        .abi
        .function(function_name)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Function not found in ABI"))?;

    println!("Calling function '{}' on contract at address: {:?}", function_name, contract_data.address);
    println!("Function: {:?}", function);
    
    // The data should start with the function selector
    let function_selector = function.short_signature(); // This gives you the first 4 bytes of the hash of the function signature.
    println!("Function selector: {:?}", function_selector);

    // Combine the function selector and the encoded arguments
    let data = [function_selector.to_vec(), encoded_inputs].concat();

    println!("stack data: {:?}", data);
    // Execute the function call
    let (exit_reason, output) = executor.transact_call(
        caller_address,
        contract_data.address.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Contract address not set"))?,
        U256::zero(), // value sent with the call
        data,
        u64::MAX, // gas_limit
        Vec::new(), // access_list
    );
    match exit_reason {
        ExitReason::Succeed(_) => {
            // Decode the function output if there is any
            let decoded_output = if !output.is_empty() {
                function.decode_output(&output).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
            } else {
                Vec::new()
            };
            println!("Raw output: {:?}", output);
            println!("Output    : {:?}", decoded_output);
            Ok(decoded_output)
        }
        _ => {
            eprintln!("Call failed: {:?}", exit_reason);
            Err(io::Error::new(io::ErrorKind::Other, "Call failed"))
        }
    }
}

fn ask_for_function_inputs(params: &[ParamType], deployer_address: H160) -> Result<Vec<String>, io::Error> {

    let mut args = Vec::new();
    for (i, param) in params.iter().enumerate() {
        let input = match param {
            ParamType::Address => {
                let default_address = format!("{:?}", deployer_address);
                let input: String = Input::new()
                    .with_prompt(&format!("Parameter {} [address] (press enter for default: {})", i, default_address))
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let input = if input.trim().is_empty() {
                    default_address // Use the default deployer address
                } else {
                    input
                };
                input
            },
            ParamType::Uint(size) => {
                let input: String = Input::new()
                    .with_prompt(&format!("Parameter {} [uint{}] value (press enter for 0)", i, size))
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let input = if input.trim().is_empty() {
                    "0".to_string() // Default value: 0
                } else {
                    input
                };
                input
            },
            ParamType::String => {
                let input: String = Input::new()
                    .with_prompt(&format!("Parameter {} [string] (press enter for empty)", i))
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let input = if input.trim().is_empty() {
                    "".to_string() // Default value: empty string
                } else {
                    input
                };
                input
            },
            ParamType::Bool => {
                let input: String = Input::new()
                    .with_prompt(&format!("Parameter {} [boolean] (true or false, press enter for false)",i))
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let input = if input.trim().is_empty() {
                    "false".to_string() // Default value: false
                } else {
                    input
                };
                input
            },
            ParamType::FixedBytes(size) => {
                let input: String = Input::new()
                    .with_prompt(&format!("Parameter {} [bytes32] (up to 64 hex characters, press enter for zeroed bytes)", i))
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            
                let input = if input.trim().is_empty() {
                    "0".repeat(64) // Default value: zeroed bytes32
                } else {
                    let trimmed_input = input.trim();
                    if trimmed_input.len() > size * 2 {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Input must be up to {} hex characters", size * 2)));
                    }
            
                    // If the length of trimmed_input is odd, prepend a zero to make it even length.
                    let even_length_input = if trimmed_input.len() % 2 == 1 {
                        format!("0{}", trimmed_input)
                    } else {
                        trimmed_input.to_string()
                    };
            
                    // Pad the input with zeros to make it fit into 32 bytes.
                    format!("{:0<width$}", even_length_input, width = size * 2)
                };
            
                input
            },            
            _ => {
                println!("Unhandled parameter type: {:?}", param);
                unimplemented!("Type not supported yet: {:?}", param);
            },
        };
        args.push(input);
    }

    Ok(args)
}

fn encode_function_args(params: &[ParamType], args: Vec<String>) -> Result<Vec<u8>, io::Error> {

    let tokens = params.iter().zip(args.iter()).map(|(param, arg)| {
        match param {
            ParamType::Address => {
                let parsed_address = arg.trim_start_matches("0x").parse::<H160>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
                Ok(ethabi::Token::Address(parsed_address))
            },
            ParamType::Uint(_) => {
                let parsed_uint = U256::from_dec_str(arg)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
                Ok(ethabi::Token::Uint(parsed_uint))
            },
            ParamType::String => {
                Ok(ethabi::Token::String(arg.clone()))
            },
            ParamType::Bool => {
                let value = match arg.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid bool input")),
                };
                Ok(ethabi::Token::Bool(value))
            },
            ParamType::FixedBytes(size) if *size == 32 => {
                if arg.len() != 64 {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "FixedBytes(32) must be 64 hex characters"));
                }
                let mut bytes = Vec::new();
                for i in 0..(arg.len() / 2) {
                    let byte = u8::from_str_radix(&arg[i*2..i*2+2], 16)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
                    bytes.push(byte);
                }
                Ok(ethabi::Token::FixedBytes(bytes))
            },
            _ => unimplemented!("Type not supported yet: {:?}", param),
        }
    }).collect::<Result<Vec<_>, _>>()
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    

    let encoded = ethabi::encode(&tokens);
    Ok(encoded)
}

fn compile_contracts(contracts_dir: &str, contract_names: &[String]) -> Result<(), io::Error> {
    for contract_name in contract_names {
        // Extract the contract base name without the .sol extension
        let contract_base_name = Path::new(contract_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid contract file name"))?
            .to_string();

        // Construct the path for the contract-specific build directory
        let contract_build_dir = format!("./build/contracts/{}", contract_base_name);
        fs::create_dir_all(&contract_build_dir)?; // Create the contract-specific build directory

        // Construct the full path for the contract file
        let contract_file = format!("{}/{}", contracts_dir, contract_name);
        // Convert the contract file path to an absolute path
        let contract_file_path = fs::canonicalize(&contract_file)?;

        println!("Compiling: {}", contract_file_path.display());

        // Run the solc command to compile the contract
        let output = Command::new("solc")
            .args(&[
                "--abi",
                "--bin",
                contract_file_path.to_str().unwrap(),
                "-o",
                &contract_build_dir, // Output to the contract-specific build directory
                "--overwrite"
            ])
            .output()?;

        // Create owned strings from the trimmed stdout and stderr
        let stdout = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim_end().to_string();

        // Print the stdout and stderr of the solc command if not empty
        if !stdout.is_empty() {
            println!("solc stdout: {}", stdout);
        }
        if !stderr.is_empty() {
            println!("solc stderr: {}", stderr);
        }

        // Exit if the compilation failed
        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            eprintln!("Failed to compile {}: {}", contract_file_path.display(), error_message);
            
            // Exit the program with a non-zero status code
            exit(1);
        }

        // Collect all paths first
        let paths: Vec<_> = fs::read_dir(&contract_build_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect();

        // Process the files for renaming
        for path in paths {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                // Check if the file is .bin or .abi and rename it
                if (filename.ends_with(".bin") || filename.ends_with(".abi")) && !filename.starts_with(&contract_base_name) {
                    let extension = path.extension().unwrap().to_str().unwrap();
                    let new_filename = format!("{}.{}", contract_base_name, extension);
                    let new_path = path.with_file_name(new_filename);

                    // Perform the rename
                    fs::rename(&path, &new_path)?;
                    println!("Renamed {} to {}", path.display(), new_path.display());
                }
            }
        }
    }
    Ok(())
}

fn deploy_contracts<'a>(
    contracts_data: &mut ContractsData,
    executor: &mut StackExecutor<'a, 'a, MemoryStackState<'a, 'a, MemoryBackend<'a>>, ()>,
    deployer: &mut Account,
) -> Result<(), io::Error> {
    
    for (contract_name, contract_data) in contracts_data.iter_mut() {
        println!("\nDeploying contract: {}.sol", contract_name);
        // Load bytecode
        let bytecode_path = format!("./build/contracts/{}/{}.bin", contract_name, contract_name);
        let bytecode_path = fs::canonicalize(&bytecode_path)?;
        let bytecode = fs::read_to_string(&bytecode_path)
            .map_err(|e| io::Error::new(io::ErrorKind::NotFound, e.to_string()))?
            .trim_end()
            .to_string();

        // Decode the hex bytecode
        let mut bytecode = hex::decode(bytecode)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        // Get constructor parameters from contract_data.abi
        if let Some(constructor) = contract_data.abi.constructor() {
            let params = &constructor.inputs;

            // Ask for constructor args
            let args = ask_for_function_inputs(&params.iter().map(|p| p.kind.clone()).collect::<Vec<_>>(), deployer.address)?;

            // Encode constructor args
            let encoded_args = encode_function_args(&params.iter().map(|p| p.kind.clone()).collect::<Vec<_>>(), args)?;

            // Append encoded args to bytecode
            bytecode.extend(encoded_args);
        }

        // Deploy the contract
        let (exit_reason, _output) = executor.transact_create(
            deployer.address,
            U256::zero(), // value
            bytecode,
            u64::MAX, // gas_limit
            Vec::new(), // access_list
        );

        // Check if the transaction was successful
        if let ExitReason::Succeed(_) = exit_reason {
            // Use sender_nonce to compute the contract address
            let contract_address = compute_contract_address(deployer.address, deployer.nonce);
            println!("Contract {} deployed at: {:?}", contract_name, contract_address);

            // Update the address in contract_data
            contract_data.address = Some(contract_address);

            // Increment sender's nonce for the next contract
            deployer.nonce += U256::one();
        } else {
            eprintln!("Failed to deploy contract {}: {:?}", contract_name, exit_reason);
            return Err(io::Error::new(io::ErrorKind::Other, "Contract deployment failed"));
        }
    }

    Ok(())
}

// https://github.com/rust-blockchain/evm-tests
fn main() -> Result<(), io::Error> {

    let contracts_dir = "./contracts"; 
    let contract_names ; 
    let mut contracts_data: ContractsData = HashMap::new();
    let mut deployer = Account {
        address: H160::random(),
        nonce: U256::zero(),
    };
      
	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::zero(),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: Default::default(),
		block_coinbase: Default::default(),
		block_timestamp: Default::default(),
		block_difficulty: Default::default(),
		block_gas_limit: Default::default(),
		chain_id: U256::one(),
		block_base_fee_per_gas: U256::zero(),
		block_randomness: None,
	};

    // Runtime configuration.
    let config = evm::Config {
        gas_ext_code: 700,
        gas_ext_code_hash: 700,
        gas_balance: 700,
        gas_sload: 800,
        gas_sload_cold: 0,
        gas_sstore_set: 20000,
        gas_sstore_reset: 5000,
        refund_sstore_clears: 15000,
        max_refund_quotient: 2,
        gas_suicide: 5000,
        gas_suicide_new_account: 25000,
        gas_call: 700,
        gas_expbyte: 50,
        gas_transaction_create: 53000,
        gas_transaction_call: 21000,
        gas_transaction_zero_data: 4,
        gas_transaction_non_zero_data: 16,
        gas_access_list_address: 0,
        gas_access_list_storage_key: 0,
        gas_account_access_cold: 0,
        gas_storage_read_warm: 0,
        sstore_gas_metering: true,
        sstore_revert_under_stipend: true,
        increase_state_access_gas: false,
        decrease_clears_refund: false,
        disallow_executable_format: false,
        warm_coinbase_address: false,
        err_on_call_with_more_gas: false,
        empty_considered_exists: false,
        create_increase_nonce: true,
        call_l64_after_gas: true,
        stack_limit: 1024,
        memory_limit: usize::MAX,
        call_stack_limit: 1024,
        create_contract_limit: Some(0x6000),
        max_initcode_size: None,
        call_stipend: 2300,
        has_delegate_call: true,
        has_create2: true,
        has_revert: true,
        has_return_data: true,
        has_bitwise_shifting: true,
        has_chain_id: true,
        has_self_balance: true,
        has_ext_code_hash: true,
        has_base_fee: false,
        has_push0: true,
        estimate: false,
    };

    let precompiles = ();

    // Backends store state information of the VM, and exposes it to runtime.
    // Initialize the backend with the vicinity and an empty state
    let mut backend = MemoryBackend::new(&vicinity, Default::default());
  
    let metadata = StackSubstateMetadata::new(u64::MAX, &config);

    // Now provide both the backend and the metadata to Memorystack_state::new
    let mut stack_state = MemoryStackState::new(metadata, &mut backend);
    
    // Make it rain for the deployer
    let account = stack_state.account_mut(deployer.address);
    account.basic.balance = U256::max_value();
    account.basic.nonce = U256::from(0);
    
    // EVM setup ready, make the executor       
    let mut executor = StackExecutor::<'_, '_, _, _>::new_with_precompiles(stack_state, &config, &precompiles);
    
    // Find the contracts in the contracts directory
    match collect_contract_names(contracts_dir) {
        Ok(names) => {
            contract_names = names; // Assign the names here
            println!("*** Contracts found ***");
            for contract in &contract_names {
                println!("{}", contract);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }

    println!("\n*** Compiling contracts ***");
    compile_contracts(contracts_dir, &contract_names).expect("Failed to compile contracts");

    // Load ABIs and create ContractData entries
    for contract_name in &contract_names {
        // Extract the contract base name without the .sol extension
        let contract_base_name = Path::new(contract_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid contract file name"))?
            .to_string();
    
        let abi_path = format!("./build/contracts/{}/{}.abi", contract_base_name, contract_base_name);
        let abi = parse_abi(&abi_path).expect("Failed to parse ABI");
    
        contracts_data.insert(contract_base_name.clone(), ContractData {
            address: None,  // Address to be filled in after deployment
            abi,
        });
    }

    // Deploy the contracts
    println!("\n*** Start Deploying ***");
    println!("deployer: {:?}", deployer.address);
    if let Err(e) = deploy_contracts(
        &mut contracts_data,
        &mut executor,
        &mut deployer,
    ) {
        eprintln!("Error deploying contracts: {}", e);
        exit(1);
    }

    // Interaction loop
    loop {
        // Ask the user which contract they want to interact with
        let chosen_contract_name = choose_contract(&contracts_data).expect("Failed to choose a contract");

        // Get the chosen contract data
        let contract_data = contracts_data.get(&chosen_contract_name).expect("Contract not found");

        // Ask the user which function of the contract they want to call
        println!("\nAvailable functions:");
        let (chosen_function_name, _is_getter, _return_types) = choose_function(&contract_data.abi).expect("Failed to choose a function");

        // Get the function so we can iterate over it's inputs
        let function = contract_data.abi.function(&chosen_function_name).expect("Function not found");
        
        // Ask for constructor args
        let args = ask_for_function_inputs(&function.inputs.iter().map(|p| p.kind.clone()).collect::<Vec<_>>(), deployer.address)?;

        // Encode constructor args
        let encoded_args = encode_function_args(&function.inputs.iter().map(|p| p.kind.clone()).collect::<Vec<_>>(), args)?;

        // Call the function
        match call_contract_function(&mut executor, contract_data, &chosen_function_name, encoded_args, deployer.address) {
            Ok(output) => {
                println!("Function call was successful. Output: {:?}", output);
            },
            Err(e) => {
                eprintln!("Function call failed: {}", e);
                exit(1);
            }
        }
    }
    #[allow(unreachable_code)] 
    Ok(())
}


