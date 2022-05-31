use std::sync::Arc;

use litheumcommon::{
    constants::Constants,
    keypair::Keypair,
    keypair_store::KeypairStore,
    timestamp_generator::{AbstractTimestampGenerator, SystemTimestampGenerator},
    wallet_database::WalletDatabase,
};

pub fn greet() -> String {
    "Hello, world from Rust!".to_string()
}

pub fn get_address() -> String {
    let (keypair, _encrypted_key) = Keypair::make_encrypted_key_with_password(&"asdf");

    // let keypair_store = Arc::new(KeypairStore::new(keypair));
    keypair.get_address()
}

pub fn get_balance() -> u64 {
    let constants = Arc::new(Constants::new());
    let timestamp_generator: Arc<Box<dyn AbstractTimestampGenerator + Send + Sync>> =
        Arc::new(Box::new(SystemTimestampGenerator::new()));

    // let keypair_store = Arc::new(KeypairStoreStorage::keypairstore_initialized_from_storage(
    //     &litheum_config.datafiles.key_path,
    //     &command_line_opts.password,
    //     &*NATIVE_SYSTEM_UTILS,
    // ));
    let (keypair, _encrypted_key) = Keypair::make_encrypted_key_with_password(&"asdf");
    let keypair_store = Arc::new(KeypairStore::new(keypair));
    let wallet_db = WalletDatabase::new(
        constants.clone(),
        keypair_store.clone(),
        timestamp_generator.clone(),
    );

    wallet_db.get_balance().unwrap()
}
