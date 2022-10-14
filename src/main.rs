extern crate bigint;
extern crate sputnikvm;

use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::ops::Deref;

use sputnikvm::{Log, AccountChange, AccountCommitment, EmbeddedPatch, VMTestPatch, Patch, Context, VMStatus, SeqContextVM,
                HeaderParams, ValidTransaction, TransactionAction,
                VM, SeqTransactionVM};

use sputnikvm::errors::RequireError;
                // use sputnikvm::{Log, Context,
                //     AccountChange, AccountCommitment,
                //     HeaderParams};


use rlp;
use sha3::Keccak256;
use sha3::Digest;
                //use sputnikvm::{VM, SeqContextVM, Context, VMStatus, Patch};
use bigint::{Gas, M256, U256, H256, Address};

use hexutil::*;
use serde_json::Value;
use serde_json::from_str;

fn main() {
   let block_number = 1000;
   let transaction = ValidTransaction {
     caller: Some(Address::default()),
     gas_price: Gas::zero(),
     gas_limit: Gas::max_value(),
     action: TransactionAction::Create,
     value: U256::zero(),
     input: Rc::new(Vec::new()),
     nonce: U256::zero()
   };
   let header = HeaderParams {
     beneficiary: Address::default(),
     timestamp: 0,
     number: U256::zero(),
     difficulty: U256::zero(),
     gas_limit: Gas::zero()
   };
   //
   // To start a VM on the Transaction level, use the `TransactionVM` struct. 
   //
   // "Context execution, as with other EVM implementations, will not handle transaction-level gas reductions."
   // What does this mean?
   //
//    let vm: VM = if block_number < 500 {
//      SeqTransactionVM::<VMTestPatch>::new(
//        transaction, header);
//    } else {
//      SeqTransactionVM::<EmbeddedPatch>::new(
//        transaction, header);
//    };
    let mut vm = SeqTransactionVM::<VMTestPatch>::new(transaction, header);
    let peek_option = vm.peek();

    println!("{:?}", peek_option);
    let fire_result = vm.fire();
    let peek_option = vm.peek();

    println!("{:?}", peek_option);
    println!("{:?}", fire_result);
    let TESTS: Value = serde_json::from_str(include_str!("../test.json")).unwrap();
    for (name, value) in TESTS.as_object().unwrap().iter() {
        print!("\t{} ... ", name);
        match test_transaction::<VMTestPatch>(name, value, true) {
            Ok(false) => panic!("test inputLimits::{} failed", name),
            _ => (),
        }
    }

 }

 static EMPTY: [u8; 0] = [];

 pub struct JSONBlock {
    codes: HashMap<Address, Vec<u8>>,
    balances: HashMap<Address, U256>,
    storages: HashMap<Address, HashMap<U256, M256>>,
    nonces: HashMap<Address, U256>,

    beneficiary: Address,
    timestamp: u64,
    number: U256,
    difficulty: U256,
    gas_limit: Gas,

    logs: Vec<Log>,
}



impl JSONBlock {
    pub fn block_header(&self) -> HeaderParams {
        HeaderParams {
            beneficiary: self.beneficiary,
            timestamp: self.timestamp,
            number: self.number,
            difficulty: self.difficulty,
            gas_limit: self.gas_limit,
        }
    }

    pub fn request_account(&self, address: Address) -> AccountCommitment {
        let balance = self.balance(address);
        let code = self.account_code(address);
        let nonce = self.account_nonce(address);

        AccountCommitment::Full {
            address,
            balance,
            code: Rc::new(code.into()),
            nonce
        }
    }

    pub fn request_account_storage(&self, address: Address, index: U256) -> AccountCommitment {
        let hashmap_default = HashMap::new();
        let storage = self.storages.get(&address).unwrap_or(&hashmap_default);
        let value = match storage.get(&index) {
            Some(val) => *val,
            None => M256::zero(),
        };

        AccountCommitment::Storage {
            address,
            index,
            value,
        }
    }

    pub fn request_account_code(&self, address: Address) -> AccountCommitment {
        let default = Vec::new();
        let code = self.codes.get(&address).unwrap_or(&default);

        AccountCommitment::Code {
            address,
            code: Rc::new(code.clone()),
        }
    }

    pub fn apply_account(&mut self, account: AccountChange) {
        match account {
            AccountChange::Full {
                address,
                balance,
                changing_storage,
                code,
                nonce,
            } => {
                self.set_balance(address, balance);
                self.set_account_code(address, code.as_slice());
                self.storages.entry(address).or_insert_with(HashMap::new);
                let changing_storage: HashMap<U256, M256> = changing_storage.into();
                for (key, value) in changing_storage {
                    self.storages.get_mut(&address).unwrap().insert(key, value);
                }
                self.set_account_nonce(address, nonce);
            },
            AccountChange::Create {
                address, balance, storage, code, nonce, ..
            } => {
                self.set_balance(address, balance);
                self.set_account_code(address, code.as_slice());
                self.storages.insert(address, storage.into());
                self.set_account_nonce(address, nonce);
            },
            AccountChange::Nonexist(address) => {
                self.set_balance(address, U256::zero());
                self.set_account_code(address, &[]);
                self.storages.insert(address, HashMap::new());
                self.set_account_nonce(address, U256::zero());
            },
            AccountChange::IncreaseBalance(address, topup) => {
                let balance = self.balance(address);
                self.set_balance(address, balance + topup);
            },
        }
    }

    pub fn apply_log(&mut self, log: Log) {
        self.logs.push(log);
    }

    pub fn coinbase(&self) -> Address {
        self.beneficiary
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn number(&self) -> U256 {
        self.number
    }

    pub fn difficulty(&self) -> U256 {
        self.difficulty
    }

    pub fn gas_limit(&self) -> Gas {
        self.gas_limit
    }

    pub fn account_nonce(&self, address: Address) -> U256 {
        self.nonces.get(&address).map_or(U256::zero(), |&s| s)
    }

    pub fn set_account_nonce(&mut self, address: Address, nonce: U256) {
        self.nonces.insert(address, nonce);
    }

    pub fn account_code(&self, address: Address) -> &[u8] {
        self.codes.get(&address).map_or(EMPTY.as_ref(), |s| s.as_ref())
    }

    pub fn set_account_code(&mut self, address: Address, code: &[u8]) {
        self.codes.insert(address, code.into());
    }

    pub fn balance(&self, address: Address) -> U256 {
        self.balances.get(&address).map_or(U256::zero(), |&s| s)
    }

    pub fn set_balance(&mut self, address: Address, balance: U256) {
        self.balances.insert(address, balance);
    }

    pub fn nonce(&self, address: Address) -> U256 {
        self.nonces.get(&address).map_or(U256::zero(), |&s| s)
    }

    pub fn account_storage(&self, address: Address, index: U256) -> M256 {
        match self.storages.get(&address) {
            None => M256::zero(),
            Some(ref ve) => {
                match ve.get(&index) {
                    Some(&v) => v,
                    None => M256::zero()
                }
            }
        }
    }

    pub fn set_account_storage(&mut self, address: Address, index: U256, val: M256) {
        if self.storages.get(&address).is_none() {
            self.storages.insert(address, HashMap::new());
        }

        let v = self.storages.get_mut(&address).unwrap();
        v.insert(index, val);
    }

    pub fn find_log(&self, address: Address, data: &[u8], topics: &[H256]) -> bool {
        for log in &self.logs {
            if log.address == address && log.data.as_slice() == data && log.topics.as_slice() == topics {
                return true;
            }
        }
        false
    }

    pub fn logs_rlp_hash(&self) -> U256 {
        let encoded = rlp::encode_list(&self.logs[..]);
        let mut keccak = Keccak256::new();
        keccak.input(&encoded[..]);
        let hash = keccak.result();
        U256::from(&hash[..])
    }
}


pub fn apply_to_block<P: Patch>(machine: &SeqContextVM<P>, block: &mut JSONBlock) {
    for account in machine.accounts() {
        let account = (*account).clone();
        block.apply_account(account);
    }
    for log in machine.logs() {
        let log = (*log).clone();
        block.apply_log(log);
    }
}

pub fn fire_with_block<P: Patch>(machine: &mut SeqContextVM<P>, block: &JSONBlock) {
    loop {
        match machine.fire() {
            Err(RequireError::Account(address)) => {
                let account = block.request_account(address);
                machine.commit_account(account).unwrap();
            },
            Err(RequireError::AccountCode(address)) => {
                let account = block.request_account_code(address);
                machine.commit_account(account).unwrap();
            },
            Err(RequireError::AccountStorage(address, index)) => {
                let account = block.request_account_storage(address, index);
                machine.commit_account(account).unwrap();
            },
            Err(RequireError::Blockhash(number)) => {
                // The test JSON file doesn't expose any block
                // information. So those numbers are crafted by hand.
                let hash1 = H256::from_str("0xc89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6").unwrap();
                let hash2 = H256::from_str("0xad7c5bef027816a800da1736444fb58a807ef4c9603b7848673f7e3a68eb14a5").unwrap();
                let hash256 = H256::from_str("0x6ca54da2c4784ea43fd88b3402de07ae4bced597cbb19f323b7595857a6720ae").unwrap();

                let hash = if number == U256::from(1u64) {
                    hash1
                } else if number == U256::from(2u64) {
                    hash2
                } else if number == U256::from(256u64) {
                    hash256
                } else {
                    panic!();
                };

                machine.commit_blockhash(number, hash).unwrap();
            },
            Ok(()) => return,
        }
    }
}

pub fn create_context(v: &Value) -> Context {
    let address = Address::from_str(v["exec"]["address"].as_str().unwrap()).unwrap();
    let caller = Address::from_str(v["exec"]["caller"].as_str().unwrap()).unwrap();
    let code = read_hex(v["exec"]["code"].as_str().unwrap()).unwrap();
    let data = read_hex(v["exec"]["data"].as_str().unwrap()).unwrap();
    let gas_limit = Gas::from(read_u256(v["exec"]["gas"].as_str().unwrap()));
    let gas_price = Gas::from(read_u256(v["exec"]["gasPrice"].as_str().unwrap()));
    let origin = Address::from_str(v["exec"]["origin"].as_str().unwrap()).unwrap();
    let value = read_u256(v["exec"]["value"].as_str().unwrap());

    Context {
        address,
        caller,
        code: Rc::new(code),
        data: Rc::new(data),
        gas_limit,
        gas_price,
        origin,
        value,
        apprent_value: value,
        is_system: false,
        is_static: false,
    }
}


pub fn create_machine<P: Patch>(v: &Value, block: &JSONBlock) -> SeqContextVM<P> {
    let transaction = create_context(v);

    SeqContextVM::<P>::new(transaction, block.block_header())
}


pub fn create_block(v: &Value) -> JSONBlock {
    let mut block = {
        let env = &v["env"];

        let current_coinbase = env["currentCoinbase"].as_str().unwrap();
        let current_difficulty = env["currentDifficulty"].as_str().unwrap();
        let current_gas_limit = env["currentGasLimit"].as_str().unwrap();
        let current_number = env["currentNumber"].as_str().unwrap();
        let current_timestamp = env["currentTimestamp"].as_str().unwrap();

        JSONBlock {
            balances: HashMap::new(),
            storages: HashMap::new(),
            codes: HashMap::new(),
            nonces: HashMap::new(),

            beneficiary: Address::from_str(current_coinbase).unwrap(),
            difficulty: read_u256(current_difficulty),
            gas_limit: Gas::from(read_u256(current_gas_limit)),
            number: read_u256(current_number),
            timestamp: read_u256(current_timestamp).into(),
            logs: Vec::new(),
        }
    };

    let pre_addresses = &v["pre"];

    for (address, data) in pre_addresses.as_object().unwrap() {
        let address = Address::from_str(address.as_str()).unwrap();
        let balance = read_u256(data["balance"].as_str().unwrap());
        let code = read_hex(data["code"].as_str().unwrap()).unwrap();

        block.set_account_code(address, code.as_ref());
        block.set_balance(address, balance);

        let storage = data["storage"].as_object().unwrap();
        for (index, value) in storage {
            let index = read_u256(index.as_str());
            let value = M256::from_str(value.as_str().unwrap()).unwrap();
            block.set_account_storage(address, index, value);
        }
    }

    block
}

/// Read U256 number exactly the way go big.Int parses strings
/// except for base 2 and 8 which are not used in tests
pub fn read_u256(number: &str) -> U256 {
    if number.starts_with("0x") {
        U256::from_str(number).unwrap()
    } else {
        U256::from_dec_str(number).unwrap()
    }
}

// TODO: consider refactoring
#[cfg_attr(feature = "cargo-clippy", allow(cyclomatic_complexity))]
#[cfg_attr(feature = "cargo-clippy", allow(collapsible_if))]
pub fn test_machine<P: Patch>(v: &Value, machine: &SeqContextVM<P>, block: &JSONBlock, history: &[Context], debug: bool) -> bool {
    let callcreates = &v["callcreates"];

    if callcreates.as_array().is_some() {
        for (i, callcreate) in callcreates.as_array().unwrap().into_iter().enumerate() {
            let data = read_hex(callcreate["data"].as_str().unwrap()).unwrap();
            let destination = {
                let destination = callcreate["destination"].as_str().unwrap();
                if destination == "" {
                    None
                } else {
                    Some(Address::from_str(destination).unwrap())
                }
            };
            let gas_limit = Gas::from(read_u256(callcreate["gasLimit"].as_str().unwrap()));
            let value = read_u256(callcreate["value"].as_str().unwrap());

            if i >= history.len() {
                if debug {
                    println!();
                    println!("Transaction check failed, expected more than {} items.", i);
                }
                return false;
            }
            let transaction = &history[i];
            if destination.is_some() {
                if transaction.address != destination.unwrap() {
                    if debug {
                        println!();
                        println!("Transaction address mismatch. 0x{:x} != 0x{:x}.", transaction.address, destination.unwrap());
                    }
                    return false;
                }
            }
            if transaction.gas_limit != gas_limit || transaction.value != value || if destination.is_some() { transaction.data.deref() != &data } else { transaction.code.deref() != &data } {
                if debug {
                    println!();
                    println!("Transaction mismatch. gas limit 0x{:x} =?= 0x{:x}, value 0x{:x} =?= 0x{:x}, data {:?} =?= {:?}", transaction.gas_limit, gas_limit, transaction.value, value, transaction.data, data);
                }
                return false;
            }
        }
    }

    let out = v["out"].as_str();
    let gas = v["gas"].as_str();

    if let Some(out) = out {
        let out = read_hex(out).unwrap();
        let out_ref: &[u8] = out.as_ref();
        if machine.out() != out_ref {
            if debug {
                println!();
                println!("Return value check failed. {:?} != {:?}", machine.out(), out_ref);
            }

            return false;
        }
    }

    if let Some(gas) = gas {
        let gas = Gas::from(read_u256(gas));
        if machine.available_gas() != gas {
            if debug {
                println!();
                println!("Gas check failed, VM returned: 0x{:x}, expected: 0x{:x}",
                         machine.available_gas(), gas);
            }

            return false;
        }
    }

    let post_addresses = &v["post"];

    for (address, data) in post_addresses.as_object().unwrap() {
        let address = Address::from_str(address.as_str()).unwrap();
        let balance = read_u256(data["balance"].as_str().unwrap());
        let nonce = read_u256(data["nonce"].as_str().unwrap());
        let code = read_hex(data["code"].as_str().unwrap()).unwrap();
        let code_ref: &[u8] = code.as_ref();

        if code_ref != block.account_code(address) {
            if debug {
                println!();
                println!("Account code check failed for address 0x{:x}.", address);
                println!("Expected: {:x?}", block.account_code(address));
                println!("Actual:   {:x?}", code_ref);
            }

            return false;
        }

        if balance != block.balance(address) {
            if debug {
                println!();
                println!("Balance check failed for address 0x{:x}.", address);
                println!("Expected: 0x{:x}", balance);
                println!("Actual:   0x{:x}", block.balance(address));
            }

            return false;
        }

        if nonce != block.nonce(address) {
            if debug {
                println!();
                println!("Nonce check failed for address 0x{:x}.", address);
                println!("Expected: 0x{:x}", nonce);
                println!("Actual:   0x{:x}", block.nonce(address));
            }

            return false;
        }

        let storage = data["storage"].as_object().unwrap();
        for (index, value) in storage {
            let index = read_u256(index.as_str());
            let value = M256::from_str(value.as_str().unwrap()).unwrap();
            if value != block.account_storage(address, index) {
                if debug {
                    println!();
                    println!("Storage check failed for address 0x{:x} in storage index 0x{:x}",
                             address, index);
                    println!("Expected: 0x{:x}", value);
                    println!("Actual:   0x{:x}", block.account_storage(address, index));
                }
                return false;
            }
        }
    }

    let expect = &v["expect"];

    if expect.as_object().is_some() {
        for (address, data) in expect.as_object().unwrap() {
            let address = Address::from_str(address.as_str()).unwrap();

            let storage = data["storage"].as_object().unwrap();
            for (index, value) in storage {
                let index = read_u256(index.as_str());
                let value = M256::from_str(value.as_str().unwrap()).unwrap();
                if value != block.account_storage(address, index) {
                    if debug {
                        println!();
                        println!("Storage check (expect) failed for address 0x{:x} in storage index 0x{:x}",
                                 address, index);
                        println!("Expected: 0x{:x}", value);
                        println!("Actual:   0x{:x}", block.account_storage(address, index));
                    }
                    return false;
                }
            }
        }
    }


    let logs_hash = v["logs"].as_str().map(read_u256);

    if logs_hash.is_some() {
        let logs_hash = logs_hash.unwrap();
        let vm_logs_hash = block.logs_rlp_hash();
        if logs_hash != vm_logs_hash {
            if debug {
                println!();
                println!("Logs check failed (hashes mismatch)");
                println!("Expected: 0x{:x}", logs_hash);
                println!("Actual: 0x{:x}", vm_logs_hash);
            }
            return false;
        }
    }

    true
}

fn is_ok(status: &VMStatus) -> bool {
    match *status {
        VMStatus::ExitedOk => true,
        _ => false,
    }
}

 pub fn test_transaction<P: Patch>(_name: &str, v: &Value, debug: bool) -> Result<bool, VMStatus> {
    let _ = env_logger::try_init();

    let mut block = create_block(v);
    let history: Arc<Mutex<Vec<Context>>> = Arc::new(Mutex::new(Vec::new()));
    let history_closure = history.clone();
    let mut machine = create_machine::<P>(v, &block);
    machine.add_context_history_hook(move |context| {
        history_closure.lock().unwrap().push(context.clone());
    });
    fire_with_block(&mut machine, &block);
    apply_to_block(&machine, &mut block);

    if debug {
        println!("status: {:?}", machine.status());
    }
    let out = v["out"].as_str();

    if out.is_some() {
        if is_ok(&machine.status()) {
            if test_machine(v, &machine, &block, &history.lock().unwrap(), debug) {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(machine.status())
        }
    } else if !is_ok(&machine.status()) {
        Ok(true)
    } else {
        Ok(false)
    }
}


 pub fn run_test<P: Patch>(name: &str, test: &str) {
    let test: Value = serde_json::from_str(test).unwrap();
    //assert_eq!(test_transaction::<P>(name, &test, true), Ok(true));
}