use ethereum_types::{H160, U256};
use evm::backend::MemoryBackend;
use evm::executor::stack::{StackExecutor, MemoryStackState, StackSubstateMetadata};
use evm::ExitReason;
use evm::ExitReason::Succeed;
use rlp::RlpStream;
use sha3::{Digest, Keccak256};
use std::collections::BTreeMap;

fn compute_contract_address(sender: H160, nonce: U256) -> H160 {
    let mut stream = RlpStream::new_list(2);
    stream.append(&sender);
    stream.append(&nonce);
    let hash = Keccak256::digest(&stream.out());
    H160::from_slice(&hash[12..])
}
// https://github.com/rust-blockchain/evm-tests
fn main() {
    let contract_bytecode_hex = "608060405234801562000010575f80fd5b5060405162000b9038038062000b908339818101604052810190620000369190620001d3565b805f908162000046919062000459565b50506200053d565b5f604051905090565b5f80fd5b5f80fd5b5f80fd5b5f80fd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b620000af8262000067565b810181811067ffffffffffffffff82111715620000d157620000d062000077565b5b80604052505050565b5f620000e56200004e565b9050620000f38282620000a4565b919050565b5f67ffffffffffffffff82111562000115576200011462000077565b5b620001208262000067565b9050602081019050919050565b5f5b838110156200014c5780820151818401526020810190506200012f565b5f8484015250505050565b5f6200016d6200016784620000f8565b620000da565b9050828152602081018484840111156200018c576200018b62000063565b5b620001998482856200012d565b509392505050565b5f82601f830112620001b857620001b76200005f565b5b8151620001ca84826020860162000157565b91505092915050565b5f60208284031215620001eb57620001ea62000057565b5b5f82015167ffffffffffffffff8111156200020b576200020a6200005b565b5b6200021984828501620001a1565b91505092915050565b5f81519050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f60028204905060018216806200027157607f821691505b6020821081036200028757620002866200022c565b5b50919050565b5f819050815f5260205f209050919050565b5f6020601f8301049050919050565b5f82821b905092915050565b5f60088302620002eb7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82620002ae565b620002f78683620002ae565b95508019841693508086168417925050509392505050565b5f819050919050565b5f819050919050565b5f620003416200033b62000335846200030f565b62000318565b6200030f565b9050919050565b5f819050919050565b6200035c8362000321565b620003746200036b8262000348565b848454620002ba565b825550505050565b5f90565b6200038a6200037c565b6200039781848462000351565b505050565b5b81811015620003be57620003b25f8262000380565b6001810190506200039d565b5050565b601f8211156200040d57620003d7816200028d565b620003e2846200029f565b81016020851015620003f2578190505b6200040a62000401856200029f565b8301826200039c565b50505b505050565b5f82821c905092915050565b5f6200042f5f198460080262000412565b1980831691505092915050565b5f6200044983836200041e565b9150826002028217905092915050565b620004648262000222565b67ffffffffffffffff81111562000480576200047f62000077565b5b6200048c825462000259565b62000499828285620003c2565b5f60209050601f831160018114620004cf575f8415620004ba578287015190505b620004c685826200043c565b86555062000535565b601f198416620004df866200028d565b5f5b828110156200050857848901518255600182019150602085019450602081019050620004e1565b8683101562000528578489015162000524601f8916826200041e565b8355505b6001600288020188555050505b505050505050565b610645806200054b5f395ff3fe608060405234801561000f575f80fd5b5060043610610034575f3560e01c80633d7403a314610038578063e21f37ce14610054575b5f80fd5b610052600480360381019061004d919061025c565b610072565b005b61005c610084565b604051610069919061031d565b60405180910390f35b805f90816100809190610540565b5050565b5f80546100909061036a565b80601f01602080910402602001604051908101604052809291908181526020018280546100bc9061036a565b80156101075780601f106100de57610100808354040283529160200191610107565b820191905f5260205f20905b8154815290600101906020018083116100ea57829003601f168201915b505050505081565b5f604051905090565b5f80fd5b5f80fd5b5f80fd5b5f80fd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b61016e82610128565b810181811067ffffffffffffffff8211171561018d5761018c610138565b5b80604052505050565b5f61019f61010f565b90506101ab8282610165565b919050565b5f67ffffffffffffffff8211156101ca576101c9610138565b5b6101d382610128565b9050602081019050919050565b828183375f83830152505050565b5f6102006101fb846101b0565b610196565b90508281526020810184848401111561021c5761021b610124565b5b6102278482856101e0565b509392505050565b5f82601f83011261024357610242610120565b5b81356102538482602086016101ee565b91505092915050565b5f6020828403121561027157610270610118565b5b5f82013567ffffffffffffffff81111561028e5761028d61011c565b5b61029a8482850161022f565b91505092915050565b5f81519050919050565b5f82825260208201905092915050565b5f5b838110156102da5780820151818401526020810190506102bf565b5f8484015250505050565b5f6102ef826102a3565b6102f981856102ad565b93506103098185602086016102bd565b61031281610128565b840191505092915050565b5f6020820190508181035f83015261033581846102e5565b905092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f600282049050600182168061038157607f821691505b6020821081036103945761039361033d565b5b50919050565b5f819050815f5260205f209050919050565b5f6020601f8301049050919050565b5f82821b905092915050565b5f600883026103f67fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff826103bb565b61040086836103bb565b95508019841693508086168417925050509392505050565b5f819050919050565b5f819050919050565b5f61044461043f61043a84610418565b610421565b610418565b9050919050565b5f819050919050565b61045d8361042a565b6104716104698261044b565b8484546103c7565b825550505050565b5f90565b610485610479565b610490818484610454565b505050565b5b818110156104b3576104a85f8261047d565b600181019050610496565b5050565b601f8211156104f8576104c98161039a565b6104d2846103ac565b810160208510156104e1578190505b6104f56104ed856103ac565b830182610495565b50505b505050565b5f82821c905092915050565b5f6105185f19846008026104fd565b1980831691505092915050565b5f6105308383610509565b9150826002028217905092915050565b610549826102a3565b67ffffffffffffffff81111561056257610561610138565b5b61056c825461036a565b6105778282856104b7565b5f60209050601f8311600181146105a8575f8415610596578287015190505b6105a08582610525565b865550610607565b601f1984166105b68661039a565b5f5b828110156105dd578489015182556001820191506020850194506020810190506105b8565b868310156105fa57848901516105f6601f891682610509565b8355505b6001600288020188555050505b50505050505056fea2646970667358221220815e220e3bdf5fed30260a091bab30efda9ab9cf84034796b57b5d57534888ec64736f6c63430008170033";
    
    // Offset to the location of the data part of the string (32 bytes, since strings are dynamically sized)
    // Length of the string (13 bytes for "Hello, World!")
    // The string "Hello, World!" UTF-8 encoded, right-padded to 32 bytes
    
    let init_message_encoded = "\
        0000000000000000000000000000000000000000000000000000000000000020\
        000000000000000000000000000000000000000000000000000000000000000d\
        48656c6c6f2c20576f726c642100000000000000000000000000000000000000\
    ";
    let combined_bytecode_hex = format!("{}{}", contract_bytecode_hex, init_message_encoded);
    let contract_bytecode = match hex::decode(combined_bytecode_hex) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error decoding hex: {}", e);
            return;
        }
    };

    
    let coinbase = H160::random();
    let mut account1 = H160::random();
    let account2 = H160::random();
    let address = H160::random(); // Randomly generate an address for the contract
    let wei_per_ether = 1_000_000_000_000_000_000u64;
    let gas_per_block = 40_000_000;
    
    let contract_address = compute_contract_address(account1, U256::zero());
    println!("Contract Address: {:?}", contract_address);


    // The Vicinity basically seems to be metadata of the block that the transaction/VM will be run in.
    let vicinity = evm::backend::MemoryVicinity {
        gas_price: U256::from(1),
        origin: H160::default(),
        block_hashes: Vec::new(),
        block_number: U256::zero(), 
        block_coinbase: H160::from(coinbase),
        block_timestamp: U256::from(1_000_000_000),
        block_difficulty: U256::from(20_000_000_000_000_000u64),
        block_gas_limit: U256::from(20_000_000_000_000_000u64),
        chain_id: U256::from(1337),
        block_base_fee_per_gas: U256::from(1),
        // Litheum will provide this via the algorithm designed in the white paper (similar to Slasher)
        block_randomness: Default::default(), // Provide appropriate randomness
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

    // get MemoryAccount at address of account1 and edit it's balance
    // backend.state_mut().insert(account1, evm::backend::MemoryAccount {
    //     nonce: U256::from(0),
    //     balance: U256::max_value(),
    //     storage: BTreeMap::new(),
    //     code: vec!(),
    // });
    // // Get account from backend state()
    // let account = backend.state().get(&account1).unwrap();    
    
    let metadata = StackSubstateMetadata::new(u64::MAX, &config);

    // Now provide both the backend and the metadata to Memorystack_state::new
    let mut stack_state = MemoryStackState::new(metadata, &mut backend);

    let account = stack_state.account_mut(account1);

    println!("{:?}", account1);
    println!("{:?}", account2);
    account.basic.nonce = U256::from(0);
    account.basic.balance = U256::max_value();
    println!("{:?}", account.basic);
    println!("{:?}", account);
    
    let mut executor = StackExecutor::new_with_precompiles(stack_state, &config, &precompiles);
    
    let transact_create_data = executor.transact_create(
        account1, U256::zero(), contract_bytecode, u64::MAX, Default::default()
    );
    //let transact_create_data = executor.transact_create2(account1, U256::zero(), contract_bytecode.into(), H256::zero(), u64::MAX, Default::default());
    
    match transact_create_data {
        (ExitReason::Succeed(_), create_data ) =>  {
            println!("create_data {:?}", &create_data.len());

            // Assuming the function signature for "update" is "update(string)"
            // update(string memory newMessage) public
            let function_signature = "update(string)";
            
            

            // Create a new Keccak256 hasher
            let mut hasher = Keccak256::new();
            // Write input message
            hasher.update(function_signature);
            // Obtain the function selector (first 4 bytes of the hash)
            let function_selector = &hasher.finalize()[..4];
            
            let new_message_str = "Hello, WeiTang!"; // The new message
            let new_message_len = new_message_str.len();
            let new_message_encoded = format!(
                "{:064x}{:064x}{}",
                32, // Offset to the start of the data part of the string (32 bytes, since strings are dynamically sized)
                new_message_len, // Length of the string
                hex::encode(new_message_str) // The string UTF-8 encoded, right-padded to 32 bytes
            );

            // Note: Properly encoding the string as per ABI specifications is required here
            // Create the transaction data by concatenating the function selector and encoded message
            // let transaction_data = [function_selector, new_message.as_bytes()].concat();

            // Combine the function selector and the encoded message
            let transaction_data_hex = format!("{}{}", hex::encode(function_selector), new_message_encoded);
            let transaction_data = hex::decode(transaction_data_hex).expect("Decoding failed");



            // Execute the transaction
            let result = executor.transact_call(address, contract_address, U256::zero(), transaction_data, u64::MAX, Default::default());

            match result {
                (Succeed(_), _ ) => println!("Transaction successful!"),
                (ExitReason::Error(exit_error), why) => println!("Transaction failed! {:?} {:?}", exit_error, why),
                (ExitReason::Revert(exit_reason), _) => println!("Transaction reverted {:?}",exit_reason),
                (ExitReason::Fatal(_), _) => println!("Transaction failed fatally!")
            }

        }
        (ExitReason::Error(exit_reason),_)=> {
            println!("create contract transaction error:\n     {:?}", &exit_reason);
        }
        (ExitReason::Revert(exit_reason),_)=> {
            println!("create contract reverted:\n     {:?}", &exit_reason);
        }
        (ExitReason::Fatal(exit_reason),_)=> {
            println!("create contract transaction fatal:\n     {:?}", &exit_reason);
        }
        
    }
}


