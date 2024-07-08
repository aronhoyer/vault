use std::{env, fs::File, io::Read, path::PathBuf, process::exit};

use home::home_dir;

pub fn get_vault_path() -> PathBuf {
    if let Ok(vault_dir) = env::var("VAULT_PATH") {
        PathBuf::from(vault_dir)
    } else {
        home_dir()
            .expect("Failed to determine home directory")
            .join(".local/vault")
    }
}

pub fn get_key_id() -> String {
    let vault_path = get_vault_path();
    let key_id_path = vault_path.join(".keyid");
    if !key_id_path.exists() {
        println!("Vault not properly initialised.");
        println!("Run `vault init` to initialise vault.");
        exit(1);
    }
    let mut key_id_file = File::options()
        .read(true)
        .open(key_id_path)
        .expect("Failed to open GPG key ID file");
    let mut key_id = String::new();
    key_id_file
        .read_to_string(&mut key_id)
        .expect("Failed to read GPG key ID file");

    return key_id;
}
