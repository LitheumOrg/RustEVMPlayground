use litheumcommon::keypair::Keypair;

pub fn greet() -> String {
    "Hello, world from Rust!".to_string()
}

pub fn get_address() -> String {
    let (keypair, _encrypted_key) = Keypair::make_encrypted_key_with_password(&"asdf");

    // let keypair_store = Arc::new(KeypairStore::new(keypair));
    keypair.get_address()
}
