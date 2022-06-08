use anyhow::{anyhow, Result}; // i used anyhow::Result instead bec http://cjycode.com/flutter_rust_bridge/integrate/deps.html?highlight=anyhow#rust-dependencies
use std::sync::Arc;

use litheumcommon::{
    constants::Constants,
    crypto::decrypt_key_file,
    keypair::Keypair,
    keypair_store::KeypairStore,
    timestamp_generator::{AbstractTimestampGenerator, SystemTimestampGenerator},
    wallet_database::WalletDatabase,
};

pub fn greet() -> String {
    "Hello world from Rust!".to_string()
}

pub fn generate_keypair() -> Vec<u8> {
    let (_keypair, encrypted_key) = Keypair::make_encrypted_key_with_password("asdf");
    encrypted_key
}

pub fn get_address(slice: Vec<u8>) -> Result<String> {
    if let Ok(decrypted_buffer) = decrypt_key_file(slice, &String::from("asdf")) {
        if let Ok(keypair) = Keypair::from_secret_slice(&decrypted_buffer) {
            Ok(keypair.get_address())
        } else {
            Err(anyhow!("Keypair store not set"))
        }
    } else {
        Err(anyhow!("Could not decrypt keyfile"))
    }
}

pub fn get_balance() -> Result<u64> {
    let constants = Arc::new(Constants::new());
    let timestamp_generator: Arc<Box<dyn AbstractTimestampGenerator + Send + Sync>> =
        Arc::new(Box::new(SystemTimestampGenerator::new()));

    let (keypair, _encrypted_key) = Keypair::make_encrypted_key_with_password("asdf");
    let keypair_store = Arc::new(KeypairStore::new(keypair));
    let wallet_db = WalletDatabase::new(constants, keypair_store, timestamp_generator.clone());

    if let Ok(balance) = wallet_db.get_balance() {
        Ok(balance)
    } else {
        Err(anyhow!("Wallet db not set"))
    }
}
