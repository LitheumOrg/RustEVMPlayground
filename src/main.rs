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

use ethabi::{Contract, Param};
use ethabi::param_type::ParamType;
use std::path::PathBuf;

struct ContractData {
    address: Option<H160>,
    abi: Contract,
}

type ContractsData = HashMap<String, ContractData>;

fn choose_contract(contracts: &HashMap<String, ContractData>) -> Result<String, io::Error> {
    println!("Available contracts:");
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
                function.inputs.is_empty() && function.constant.unwrap_or(false)
            });
            let return_types = functions.iter()
                .flat_map(|function| function.outputs.clone())
                .map(|output| output.kind)
                .collect();
            (name.clone(), is_getter, return_types)
        })
        .collect();

    println!("Available functions:");
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

fn ask_for_function_inputs(params: &[ParamType], deployer_address: H160) -> Result<Vec<String>, io::Error> {

    let mut args = Vec::new();

    // Debug: Print the number of parameters to expect
    println!("Expecting {} constructor arguments", params.len());

    for (i, param) in params.iter().enumerate() {
        // Debug: Print the type of each parameter
        println!("Asking for parameter {}: {:?}", i + 1, param);

        match param {
            ParamType::Address => {
                let default_address = format!("{:?}", deployer_address);
                let input: String = Input::new()
                    .with_prompt(&format!("Enter address (press enter for default: {})", default_address))
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let input = if input.trim().is_empty() {
                    default_address // Use the default deployer address
                } else {
                    input
                };
                args.push(input);
            },
            ParamType::Uint(size) => {
                let input: String = Input::new()
                    .with_prompt(&format!("Enter uint{} value (press enter for 0)", size))
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let input = if input.trim().is_empty() {
                    "0".to_string() // Default value: 0
                } else {
                    input
                };
                args.push(input);
            },
            ParamType::String => {
                let input: String = Input::new()
                    .with_prompt("Enter string value (press enter for empty)")
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let input = if input.trim().is_empty() {
                    "".to_string() // Default value: empty string
                } else {
                    input
                };
                args.push(input);
            },
            ParamType::Bool => {
                let input: String = Input::new()
                    .with_prompt("Enter boolean value (true or false, press enter for false)")
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let input = if input.trim().is_empty() {
                    "false".to_string() // Default value: false
                } else {
                    input
                };
                args.push(input);
            },
            ParamType::FixedBytes(size) => {
                println!("Matched FixedBytes with size {}", size);
                if *size == 32 {
                    println!("Processing FixedBytes of size 32");
                    let input: String = Input::new()
                        .with_prompt("Enter bytes32 value (64 hex characters, press enter for zeroed bytes)")
                        .allow_empty(true)
                        .interact_text()
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        
                    let input = if input.trim().is_empty() {
                        "0".repeat(64) // Default value: zeroed bytes32
                    } else if input.len() == 64 && input.chars().all(|c| c.is_digit(16) || "abcdefABCDEF".contains(c)) {
                        input
                    } else {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Input must be 64 hex characters or empty for zeroed bytes"));
                    };
        
                    args.push(input);
                } else {
                    println!("Encountered FixedBytes with size other than 32: {}", size);
                    // Handle other sizes of FixedBytes or return an error
                    unimplemented!("FixedBytes of size other than 32 are not supported yet");
                }
            },
        
            // ... other match arms ...
        
            _ => {
                println!("Unhandled parameter type: {:?}", param);
                unimplemented!("Type not supported yet: {:?}", param);
            },
        }
    }

    Ok(args)
}

fn encode_constructor_args(params: &[ParamType], args: Vec<String>) -> Result<Vec<u8>, io::Error> {
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

fn parse_abi(abi_path: &str) -> Result<Contract, io::Error> {
    let path = PathBuf::from(abi_path);
    let file = fs::File::open(path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?; // Convert std::io::Error to io::Error if needed

    let contract = Contract::load(file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?; // Convert ethabi::Error to io::Error

    Ok(contract)
}

fn get_constructor_params(contract: &Contract) -> Option<&Vec<Param>> {
    contract.constructor.as_ref().map(|constructor| &constructor.inputs)
}
fn compute_contract_address(sender: H160, nonce: U256) -> H160 {
    let mut stream = RlpStream::new_list(2);
    stream.append(&sender);
    stream.append(&nonce);
    let hash = Keccak256::digest(&stream.out());
    H160::from_slice(&hash[12..])
}

fn collect_contract_names(contracts_dir: &str) -> Result<Vec<String>, io::Error> {
    let mut contract_names = Vec::new(); // Vector to store the contract names

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

        // Print the standard output and standard error of the solc command
        // Create owned strings from the trimmed stdout and stderr
        let stdout = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim_end().to_string();

        // Print the standard output of the solc command if not empty
        if !stdout.is_empty() {
            println!("solc stdout: {}", stdout);
        }

        // Check if there is any standard error output before printing
        if !stderr.is_empty() {
            println!("solc stderr: {}", stderr);
        }
        // Check if the compilation was successful
        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            eprintln!("Failed to compile {}: {}", contract_file_path.display(), error_message);
            
            // Exit the program with a non-zero status code
            exit(1);
        }
        // Check if the compilation was successful
        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            eprintln!("Failed to compile {}: {}", contract_file_path.display(), error_message);
            return Err(io::Error::new(io::ErrorKind::Other, "Compilation failed"));
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
    sender: H160,
) -> Result<(), io::Error> {
    // Initialize sender's nonce
    let mut sender_nonce = U256::zero();

    for (contract_name, contract_data) in contracts_data.iter_mut() {
        // Load bytecode
        let bytecode_path = format!("./build/contracts/{}/{}.bin", contract_name, contract_name);
        println!("Looking for bytecode at: {}", bytecode_path);
        let bytecode_path = fs::canonicalize(&bytecode_path)?;
        println!("Reading bytecode from: {}", bytecode_path.display());
        let mut bytecode = fs::read_to_string(&bytecode_path)
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
            let args = ask_for_function_inputs(&params.iter().map(|p| p.kind.clone()).collect::<Vec<_>>(), sender)?;

            // Encode constructor args
            let encoded_args = encode_constructor_args(&params.iter().map(|p| p.kind.clone()).collect::<Vec<_>>(), args)?;

            // Append encoded args to bytecode
            bytecode.extend(encoded_args);
        }

        // Deploy the contract
        let (exit_reason, _output) = executor.transact_create(
            sender,
            U256::zero(), // value
            bytecode,
            u64::MAX, // gas_limit
            Vec::new(), // access_list
        );

        sender_nonce = sender_nonce + U256::one();
        
        // Check if the transaction was successful
        if let ExitReason::Succeed(_) = exit_reason {
            // Use sender_nonce to compute the contract address
            let contract_address = compute_contract_address(sender, sender_nonce);
            println!("Contract {} deployed at: {:?}\n", contract_name, contract_address);

            // Update the address in contract_data
            contract_data.address = Some(contract_address);

            // Increment sender's nonce for the next contract
            sender_nonce += U256::one();
        } else {
            eprintln!("Failed to deploy contract {}: {:?}", contract_name, exit_reason);
            return Err(io::Error::new(io::ErrorKind::Other, "Contract deployment failed"));
        }
    }

    Ok(())
}

// https://github.com/rust-blockchain/evm-tests
fn main() -> Result<(), io::Error> {

    let contracts_dir = "./contracts"; // Path to the contracts directory
    let mut contract_names = Vec::new(); 
    let mut contract_addresses: HashMap<String, ContractData> = HashMap::new();
    let deployer_address = H160::random();
    
    match collect_contract_names(contracts_dir) {
        Ok(names) => {
            contract_names = names; // Assign the names here
            println!("Contracts found:");
            for contract in &contract_names {
                println!("{}", contract);
            }
            println!("\n");

            // Compile the contracts
            match compile_contracts(contracts_dir, &contract_names) {
                Ok(_) => println!("Compilation successful."),
                Err(e) => eprintln!("Compilation error: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
    
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
        //create_contract_limit: Some(0x6000),
        create_contract_limit: None,
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

    let account = stack_state.account_mut(deployer_address);
    account.basic.nonce = U256::from(0);
    account.basic.balance = U256::max_value();
    println!("deployer_address account {:?}", account);
        
    let mut executor = StackExecutor::<'_, '_, _, _>::new_with_precompiles(stack_state, &config, &precompiles);

    // Compile contracts and deploy them
    compile_contracts(contracts_dir, &contract_names).expect("Failed to compile contracts");

    let mut contracts_data: ContractsData = HashMap::new();

    // Load ABIs and create ContractData entries
    for contract_name in &contract_names {
        // Extract the contract base name without the .sol extension
        let contract_base_name = Path::new(contract_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid contract file name"))?
            .to_string();
    
        let abi_path = format!("./build/contracts/{}/{}.abi", contract_base_name, contract_base_name);
        println!("ABI path: {}", abi_path); // Add this line
        let abi = parse_abi(&abi_path).expect("Failed to parse ABI");
    
        contracts_data.insert(contract_base_name.clone(), ContractData {
            address: None,  // Address to be filled in after deployment
            abi,
        });
    }
    if let Err(e) = deploy_contracts(
        &mut contracts_data,
        &mut executor,
        deployer_address,
    ) {
        eprintln!("Error deploying contracts: {}", e);
    }

    // Interaction loop
    loop {
        // Ask the user which contract they want to interact with
        let chosen_contract_name = choose_contract(&contracts_data).expect("Failed to choose a contract");

        // Get the chosen contract data
        let contract_data = contracts_data.get(&chosen_contract_name).expect("Contract not found");

        // Ask the user which function of the contract they want to call
        let chosen_function_name = choose_function(&contract_data.abi).expect("Failed to choose a function");

        // Get the chosen function
        let (chosen_function_name, is_getter, return_types) = choose_function(&contract_data.abi)?;
        let function = contract_data.abi.function(&chosen_function_name).expect("Function not found");
        
        // Ask for function inputs
        let inputs = ask_for_function_inputs(
            &function.inputs.iter().map(|p| p.kind.clone()).collect::<Vec<_>>(),
            deployer_address
        ).expect("Failed to get function inputs");

        // Encode function inputs
        let encoded_inputs = encode_constructor_args(
            &function.inputs.iter().map(|p| p.kind.clone()).collect::<Vec<_>>(),
            inputs
        ).expect("Failed to encode inputs");

        // Call the function (you'll need to implement this part based on your requirements)
        // call_contract_function(&executor, &contract_data.address.expect("Address not set"), &function, &encoded_inputs);

        // Optionally, you can add some logic to break the loop or continue based on user input or function call results.
    }


    // loop {
    //     // Let the user choose a contract
    //     let contract_name = choose_contract(&contracts_data)?;
    //     let contract_data = contracts_data.get(&contract_name).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Contract not found"))?;

    //     // Let the user choose a function from the ABI
    //     let function_name = choose_function(&contract_data.abi)?;

    //     // Get the function from the ABI
    //     let function = contract_data.abi.function(&function_name).map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Function not found in ABI"))?;

    //     // Ask for input parameters for the chosen function
    //     // TODO: Implement a function to ask for parameters based on the function inputs
    //     // let inputs = ask_for_function_inputs(&function.inputs)?;

    //     // Encode the function call with the provided parameters
    //     // let data = function.encode_input(&inputs).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    //     // TODO: Call the function with the provided parameters
    //     // You need to implement the logic to send the transaction to the blockchain
    //     // call_function(contract_data.address, data)?;
            
    // }


    Ok(())
}


