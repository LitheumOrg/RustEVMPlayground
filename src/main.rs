use dialoguer::Input;

use ethabi::Contract;
use ethabi::param_type::ParamType;
use ethereum_types::{H160, U256, H256};

use evm::backend::OverlayedBackend;
use evm::backend::RuntimeBaseBackend;

use evm::standard::EtableResolver;
use evm::standard::Config;
use evm::standard::Invoker;
use evm::standard::TransactArgs;
use evm::ExitResult;
use evm::standard::PrecompileSet;

use evm::ExitSucceed;
use evm::RuntimeEnvironment;
use std::collections::BTreeSet;
use std::collections::{HashMap, BTreeMap};
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, exit};

struct ContractData {
    address: Option<H160>,
    abi: Contract,
}

#[derive(Clone, Debug)]
struct Account {
    address: H160,
    balance: U256,
    nonce: U256,
    code: Vec<u8>,
    storage: BTreeMap<H256, H256>,
    cold: bool,
}

impl Account {
    fn new(address: H160) -> Self {
        Account {
            address,
            balance: U256::zero(),
            nonce: U256::zero(),
            code: Vec::new(),
            storage: BTreeMap::new(),
            cold: true,
        }
    }
}

pub struct MockBlock {
    pub block_hash: H256,
    pub block_number: U256,
    pub block_coinbase: H160,
    pub block_timestamp: U256,
    pub block_difficulty: U256,
    pub block_randomness: Option<H256>,
    pub block_gas_limit: U256,
    pub block_base_fee_per_gas: U256,
    pub chain_id: U256,
}
struct MyBackend {
    // You can maintain a HashMap to store original storage values.
    // Not sure what this is or why it's needed....
    original_storage_map: HashMap<(H160, H256), H256>,
    // You can add a cold flag to the Account struct.
    accounts: BTreeMap<H160, Account>,
    // mock_block field
    mock_block: MockBlock, // Add mock_block field
}

impl MyBackend {
    pub fn new() -> Self {
        let mock_block = MockBlock {
            block_hash: H256::random(),
            block_number: U256::zero(),
            block_coinbase: H160::random(),
            block_timestamp: U256::zero(),
            block_difficulty: U256::zero(),
            block_randomness: Some(H256::random()),
            block_gas_limit: U256::zero(),
            block_base_fee_per_gas: U256::zero(),
            chain_id: U256::from(1173), 
        };
        println!("coinbase {:?}", mock_block.block_coinbase);
        MyBackend {
            original_storage_map: HashMap::new(),
            accounts: BTreeMap::new(),
            mock_block,
        }
    }
}

impl RuntimeBaseBackend for MyBackend {
    fn balance(&self, address: H160) -> U256 {
        println!("RuntimeBaseBackend balance {:?}", address);
        self.accounts.get(&address).map_or(U256::zero(), |account| account.balance)
    }

    fn code(&self, address: H160) -> Vec<u8> {
        println!("RuntimeBaseBackend code {:?}", address);
        self.accounts.get(&address).map_or(Vec::new(), |account| account.code.clone())
    }

    fn storage(&self, address: H160, index: H256) -> H256 {
        println!("RuntimeBaseBackend storage {:?} {:?}", address, index);
        self.accounts.get(&address).and_then(|account| account.storage.get(&index)).cloned().unwrap_or_default()
    }

    fn exists(&self, address: H160) -> bool {
        println!("RuntimeBaseBackend exists {:?}", address);
        self.accounts.contains_key(&address)
    }

    fn nonce(&self, address: H160) -> U256 {
        println!("RuntimeBaseBackend nonce {}", address);
        self.accounts.get(&address).map_or(U256::zero(), |account| account.nonce)
    }
}

struct MyPrecompileSet;

impl<S, H> PrecompileSet<S, H> for MyPrecompileSet {
    fn execute(
        &self,
        code_address: H160,
        _input: &[u8],
        _state: &mut S,
        _handler: &mut H
    ) -> Option<(ExitResult, Vec<u8>)> {
        println!("execute precompile... {:?}", code_address);
        
        match code_address {
            // Define your precompile addresses and logic here
            // We don't currently have any precompiles, so we just return None
            _ => None,
        }
    }
}

// Implement the RuntimeEnvironment trait for MyBackend using the mock block
impl RuntimeEnvironment for MyBackend {
    fn block_hash(&self, number: U256) -> H256 {
        println!("RuntimeEnvironment block_hash {}", self.mock_block.block_hash);
        self.mock_block.block_hash
    }

    fn block_number(&self) -> U256 {
        println!("RuntimeEnvironment lock_number");
        U256::zero()
    }

    fn block_coinbase(&self) -> H160 {
        println!("RuntimeEnvironment block_coinbase {:?}", self.mock_block.block_coinbase);
        self.mock_block.block_coinbase
    }

    fn block_timestamp(&self) -> U256 {
        println!("RuntimeEnvironment block_timestamp");
        self.mock_block.block_timestamp
    }

    fn block_difficulty(&self) -> U256 {
        println!("RuntimeEnvironment block_difficulty");
        self.mock_block.block_difficulty
    }

    fn block_randomness(&self) -> Option<H256> {
        println!("RuntimeEnvironment block_randomness {:?}", self.mock_block.block_randomness);
        self.mock_block.block_randomness
    }

    fn block_gas_limit(&self) -> U256 {
        println!("RuntimeEnvironment block_gas_limit");
        self.mock_block.block_gas_limit
    }

    fn block_base_fee_per_gas(&self) -> U256 {
        println!("RuntimeEnvironment block_base_fee_per_gas");
        self.mock_block.block_base_fee_per_gas
    }

    fn chain_id(&self) -> U256 {
        println!("RuntimeEnvironment chain_id");
        self.mock_block.chain_id
    }
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

fn deploy_contracts<'a, 'b, 'c>(
    contracts_data: &mut ContractsData,
    backend: &mut OverlayedBackend<MyBackend>,
    deployer: &mut Account,
    invoker: &Invoker<'a, 'b, EtableResolver<'a, 'b, 'c, MyPrecompileSet, evm::Etable<evm::standard::State<'c>, OverlayedBackend<MyBackend>, evm::trap::CallCreateTrap>>>,
) -> Result<(), io::Error> 
where
    'a: 'c, 
{
    for (contract_name, contract_data) in contracts_data.iter_mut() {
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
        // Create transaction arguments for contract creation
        let create_args = TransactArgs::Create {
            caller: deployer.address,
            value: U256::zero(),
            init_code: bytecode,
            salt: None,
            gas_limit: U256::from(1_000_000_000_000_000_000u64),
            gas_price: U256::from(1),
            access_list: Vec::new(),
        };

        // Execute the transaction
        let result = evm::transact(
            create_args,
            None, // or Some(heap_depth) if needed
            backend,
            invoker,
        );
        // Handle the result
        match result {

            // match from Invoker TransactValue:
            // Ok((ExitSucceed::Returned, Some(address))) => {
            //     -                contract_data.address = Some(address);
            //     -                println!("Contract {} deployed at address: {:?}", contract_name, address);
            //     -            },
            Ok(evm::standard::invoker::TransactValue::Create { succeed: ExitSucceed::Returned, address }) => {
                contract_data.address = Some(address);
                println!("Contract {} deployed at address: {:?}", contract_name, address);
            },
            Err(e) => {
                return Err(io::Error::new(io::ErrorKind::Other, format!("Deployment failed: {:?}", e)));
            },
            _ => {
                return Err(io::Error::new(io::ErrorKind::Other, "Unexpected deployment result"));
            }
        }

        // Increment the nonce for the deployer account
        deployer.nonce += U256::one();
    }

    Ok(())
}

fn call_contract_function<'a, 'b, 'c>(
    contract_data: &ContractData,
    function_name: &str,
    encoded_inputs: Vec<u8>,
    backend: &mut OverlayedBackend<MyBackend>,
    caller: &mut Account,
    invoker: &Invoker<'a, 'b, EtableResolver<'a, 'b, 'c, MyPrecompileSet, evm::Etable<evm::standard::State<'c>, OverlayedBackend<MyBackend>, evm::trap::CallCreateTrap>>>,
) -> Result<Vec<u8>, std::io::Error>
where
    'a: 'c, 
{
    
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
    let call_args = TransactArgs::Call {
        caller: caller.address,
        address: contract_data.address.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Contract address not set"))?,
        value: U256::zero(),
        data,
        gas_limit: U256::from(1_000_000_000_000_000_000u64),
        gas_price: U256::from(1),
        access_list: Vec::new(),
    };

    // Execute the transaction
    let result = evm::transact(
        call_args,
        None,
        backend,
        invoker,
    );

    // Handle the result
    match result {
        Ok(evm::standard::invoker::TransactValue::Call { succeed: ExitSucceed::Returned, retval }) => Ok(retval),
        Ok(_) => Err(io::Error::new(io::ErrorKind::Other, "Unexpected result")),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Call failed: {:?}", e))),
    }
}

// https://github.com/rust-blockchain/evm-tests
fn main() -> Result<(), io::Error> {

    let contracts_dir = "./contracts"; 
    let contract_names ; 
    let mut contracts_data: ContractsData = HashMap::new();
    let deployer_address = H160::random();
    let mut deployer = Account::new(deployer_address);
    println!("deployer address {:?}", deployer.address);

    // give gas to the deployer
    deployer.balance = U256::max_value();
    
    // Add deployer to Backend
    // TODO actually manage accounts properly in backend, don't clone things and let the backend keep the state
    let mut my_backend = MyBackend::new();
    my_backend.accounts.insert(deployer.address, deployer.clone());

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
 
    // Define the EVM configuration
    // The Merge (Paris) hard fork configuration
    let mut config = Config::merge();
    config.has_push0 = true;

    // Define your precompile set
    let precompiles = MyPrecompileSet;

    let mut overlayed_backend = OverlayedBackend::new(my_backend, BTreeSet::new());

    // Define your EVM table set
    let etable_set = evm::standard::Etable::<OverlayedBackend<MyBackend>>::runtime();
    
    // Create a standard EtableResolver
    let etable_resolver = EtableResolver::new(&config, &precompiles, &etable_set);
    // let resolver = Resolver::new(&config, &precompiles, &etable_set);
    // Create a standard Invoker
    // let invoker: Invoker<'_, '_, EtableResolver<'_, '_, '_, MyPrecompileSet, evm::Etable<evm::standard::State<'_>, OverlayedBackend<MyBackend>, evm::trap::CallCreateTrap>>> = evm::standard::Invoker::new(&config, &etable_resolver);
    let invoker: Invoker<'_, '_, EtableResolver<'_, '_, '_, MyPrecompileSet, evm::Etable<evm::standard::State<'_>, OverlayedBackend<MyBackend>, evm::trap::CallCreateTrap>>> = evm::standard::Invoker::new(&config, &etable_resolver);

    // Deploy the contracts
    println!("\n*** Start Deploying ***");
    if let Err(e) = deploy_contracts(
        &mut contracts_data,
        &mut overlayed_backend,
        &mut deployer,
        &invoker
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
        
        // Ask for function args
        let args = ask_for_function_inputs(&function.inputs.iter().map(|p| p.kind.clone()).collect::<Vec<_>>(), deployer.address)?;

        // Encode function args
        let encoded_args = encode_function_args(&function.inputs.iter().map(|p| p.kind.clone()).collect::<Vec<_>>(), args)?;
        
        // Call the function
        match call_contract_function(
            contract_data,
            &chosen_function_name,
            encoded_args,
            &mut overlayed_backend,
            &mut deployer,
            &invoker,
        ) {
            Ok(output) => {
                println!("Function call was successful. Output: {:?}", output);
            },
            Err(e) => {
                eprintln!("Function call failed: {}", e);
                // exit(1);
            }
        }
    }

    #[allow(unreachable_code)] 
    Ok(())
}


