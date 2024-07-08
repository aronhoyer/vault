use std::{
    fs::{create_dir_all, File},
    io::Write,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    process::{exit, Command, Stdio},
};

use crate::{
    crypto::generate_password,
    util::{get_key_id, get_vault_path},
};

use super::prompt;

pub fn init(key_id: String) {
    let vault_dir = get_vault_path();
    let key_id_path = vault_dir.join(".keyid");

    if key_id_path.exists() {
        println!("Vault already initalised for {key_id}");
        exit(1);
    }

    if !vault_dir.exists() {
        create_dir_all(vault_dir).expect("Failed to create vault directory");
    }

    let mut key_id_file = File::create(&key_id_path)
        .expect(format!("Failed to created {:?}", &key_id_path.display()).as_str());
    key_id_file
        .write(key_id.as_bytes())
        .expect("Failed to write GPG key ID");

    println!("Initialised vault for {key_id}");
}

pub fn create(name: String) {
    let key_id = get_key_id();
    let vault_path = get_vault_path();

    let mut password =
        prompt("Enter password (leave empty to generate): ", true).expect("failed to read stdin");

    if password.len() == 0 {
        password = generate_password(20);
    }

    let entry_path = vault_path.join(format!("{name}.gpg"));

    if entry_path.exists() {
        println!("{:?} is already registered.", name);
        todo!("implement overwrite");
    }

    if let Some(entry_dir) = entry_path.parent() {
        if !entry_dir.exists() {
            create_dir_all(entry_dir).expect("Failed to create parent directory for entry")
        }
    }

    let entry_file = File::create(entry_path).expect("Failed to create entry file");

    let mut perms = entry_file.metadata().unwrap().permissions();
    perms.set_mode(0o600);
    entry_file.set_permissions(perms).unwrap();

    let gpg = Command::new("gpg")
        .args(["--encrypt", "--recipient", key_id.as_str()])
        .stdin(Stdio::piped())
        .stdout(Stdio::from(entry_file))
        .spawn()
        .expect("Failed to spawn gpg");

    gpg.stdin.unwrap().write(password.as_bytes()).unwrap();

    println!("{password}");
}

pub fn get(name: String) {
    let vault_path = get_vault_path();
    let entry_path = vault_path.join(format!("{name}.gpg"));

    Command::new("gpg")
        .args(["--quiet", "--decrypt", entry_path.to_str().unwrap()])
        .exec();
}

pub fn edit(_name: String) {
    // TODO: implement this ish
    // decrypt entry to into tmp file
    // exec $EDITOR on tmp file
    // re-encrypt entry with tmp file contents

    unimplemented!("command `edit` not yet implemented");
}
