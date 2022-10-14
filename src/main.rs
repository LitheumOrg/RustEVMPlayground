extern crate bigint;
extern crate sputnikvm;

use sputnikvm::{EmbeddedPatch, VMTestPatch,
                HeaderParams, ValidTransaction, TransactionAction,
                VM, SeqTransactionVM};
use bigint::{Gas, U256, Address};
use std::rc::Rc;
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
   let vm = if block_number < 500 {
     SeqTransactionVM::<VMTestPatch>::new(
       transaction, header);
   } else {
     SeqTransactionVM::<EmbeddedPatch>::new(
       transaction, header);
   };
 }