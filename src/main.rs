use ethereum_types::{H160, U256, H256};
use evm::backend::MemoryBackend;
use sha3::{Digest, Keccak256};
    
use evm::executor::stack::{StackExecutor, MemoryStackState, StackSubstateMetadata};
use evm::ExitReason;
use evm::ExitReason::Succeed;

// https://github.com/rust-blockchain/evm-tests
fn main() {
    let contract_bytecode = "608060405234801562000010575f80fd5b5060405162000b9038038062000b908339818101604052810190620000369190620001d3565b805f908162000046919062000459565b50506200053d565b5f604051905090565b5f80fd5b5f80fd5b5f80fd5b5f80fd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b620000af8262000067565b810181811067ffffffffffffffff82111715620000d157620000d062000077565b5b80604052505050565b5f620000e56200004e565b9050620000f38282620000a4565b919050565b5f67ffffffffffffffff82111562000115576200011462000077565b5b620001208262000067565b9050602081019050919050565b5f5b838110156200014c5780820151818401526020810190506200012f565b5f8484015250505050565b5f6200016d6200016784620000f8565b620000da565b9050828152602081018484840111156200018c576200018b62000063565b5b620001998482856200012d565b509392505050565b5f82601f830112620001b857620001b76200005f565b5b8151620001ca84826020860162000157565b91505092915050565b5f60208284031215620001eb57620001ea62000057565b5b5f82015167ffffffffffffffff8111156200020b576200020a6200005b565b5b6200021984828501620001a1565b91505092915050565b5f81519050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f60028204905060018216806200027157607f821691505b6020821081036200028757620002866200022c565b5b50919050565b5f819050815f5260205f209050919050565b5f6020601f8301049050919050565b5f82821b905092915050565b5f60088302620002eb7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82620002ae565b620002f78683620002ae565b95508019841693508086168417925050509392505050565b5f819050919050565b5f819050919050565b5f620003416200033b62000335846200030f565b62000318565b6200030f565b9050919050565b5f819050919050565b6200035c8362000321565b620003746200036b8262000348565b848454620002ba565b825550505050565b5f90565b6200038a6200037c565b6200039781848462000351565b505050565b5b81811015620003be57620003b25f8262000380565b6001810190506200039d565b5050565b601f8211156200040d57620003d7816200028d565b620003e2846200029f565b81016020851015620003f2578190505b6200040a62000401856200029f565b8301826200039c565b50505b505050565b5f82821c905092915050565b5f6200042f5f198460080262000412565b1980831691505092915050565b5f6200044983836200041e565b9150826002028217905092915050565b620004648262000222565b67ffffffffffffffff81111562000480576200047f62000077565b5b6200048c825462000259565b62000499828285620003c2565b5f60209050601f831160018114620004cf575f8415620004ba578287015190505b620004c685826200043c565b86555062000535565b601f198416620004df866200028d565b5f5b828110156200050857848901518255600182019150602085019450602081019050620004e1565b8683101562000528578489015162000524601f8916826200041e565b8355505b6001600288020188555050505b50505050505056fea2646970667358221220815e220e3bdf5fed30260a091bab30efda9ab9cf84034796b57b5d57534888ec64736f6c63430008170033";

    let coinbase = H160::random();
    let account1 = H160::random();
    let account2 = H160::random();
    let address = H160::random(); // Randomly generate an address for the contract
    let wei_per_ether = 1_000_000_000_000_000_000u64;
    let gas_per_block = 40_000_000;
    
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

    // Backends store state information of the VM, and exposes it to runtime.
    // Initialize the backend with the vicinity and an empty state
    let mut backend = MemoryBackend::new(&vicinity, Default::default());

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
        has_push0: false,
        estimate: false,
    };
        
    let metadata = StackSubstateMetadata::new(u64::MAX, &config);

    // Now provide both the backend and the metadata to MemoryStackState::new
    let mut stackState = MemoryStackState::new(metadata, &mut backend);

    let account = stackState.account_mut(account1);

    account.basic.nonce = U256::from(0);
    account.basic.balance = U256::max_value();
    println!("{:?}", account.basic);

    let precompiles = ();
    
    let mut executor = StackExecutor::new_with_precompiles(stackState, &config, &precompiles);
    
    let transact_create_data = executor.transact_create(account1, U256::from(1_000_000_000_000u64), contract_bytecode.into(), gas_per_block, Default::default());
    //let transact_create_data = executor.transact_create2(account1, U256::zero(), contract_bytecode.into(), H256::zero(), u64::MAX, Default::default());

    match transact_create_data {
        (Succeed(_), create_data ) =>  {
            println!("{:?}", &create_data.len());

            // Assuming the function signature for "update" is "update(string)"
            let function_signature = "update(string)";

            // Create a new Keccak256 hasher
            let mut hasher = Keccak256::new();
            // Write input message
            hasher.update(function_signature);
            // Obtain the function selector (first 4 bytes of the hash)
            let function_selector = &hasher.finalize()[..4];

            // Encode the new message to update the contract with
            let new_message = "Hello, World!"; // The new message

            // Note: Properly encoding the string as per ABI specifications is required here
            // Create the transaction data by concatenating the function selector and encoded message
            let transaction_data = [function_selector, new_message.as_bytes()].concat();

            // Execute the transaction
            let result = executor.transact_call(address, address, U256::zero(), transaction_data, u64::MAX, Default::default());

            println!("{:?}", backend.state().contains_key(&address));
            //println!("{:?}", backend.state().contains_key(&H160::from_slice(&transact_create_data.1)));
            
            
            match result {
                (Succeed(_), _ ) => println!("Transaction successful!"),
                (ExitReason::Error(exit_error), why) => println!("Transaction failed! {:?} {:?}", exit_error, why),
                (ExitReason::Revert(_), _) => println!("Transaction reverted!"),
                (ExitReason::Fatal(_), _) => println!("Transaction failed fatally!")
            }

        }
        (exit_reason, _) => {
            println!("create contract transaction failed:\n     {:?}", &exit_reason);
        }
    }


  
    // q: why is this giving an outofgas error?
    // a: because the gas limit is too low


}


