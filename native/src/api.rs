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
    "Hello, world from Rust!".to_string()
}

pub fn generate_keypair() -> Vec<u8> {
    let (_keypair, encrypted_key) = Keypair::make_encrypted_key_with_password(&"asdf");
    encrypted_key
}

pub fn get_address(slice: Vec<u8>) -> String {
    let decrypted_buffer = decrypt_key_file(slice, &String::from("asdf")).unwrap();
    Keypair::from_secret_slice(&decrypted_buffer)
        .unwrap()
        .get_address()
}

pub fn get_balance() -> u64 {
    let constants = Arc::new(Constants::new());
    let timestamp_generator: Arc<Box<dyn AbstractTimestampGenerator + Send + Sync>> =
        Arc::new(Box::new(SystemTimestampGenerator::new()));

    let (keypair, _encrypted_key) = Keypair::make_encrypted_key_with_password(&"asdf");
    let keypair_store = Arc::new(KeypairStore::new(keypair));
    let wallet_db = WalletDatabase::new(
        constants.clone(),
        keypair_store.clone(),
        timestamp_generator.clone(),
    );

    wallet_db.get_balance().unwrap()
}
