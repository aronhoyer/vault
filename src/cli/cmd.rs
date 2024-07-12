use std::{
    env,
    fs::{create_dir_all, remove_file, rename, File},
    io::{stdout, Error, ErrorKind, Read, Result, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{exit, Command, Stdio},
};

use arboard::Clipboard;

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

pub fn get(name: String, clip: bool) {
    let vault_path = get_vault_path();
    let entry_path = vault_path.join(format!("{name}.gpg"));

    let gpg = Command::new("gpg")
        .args(["--quiet", "--decrypt", entry_path.to_str().unwrap()])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute gpg");
    let mut password = String::new();
    gpg.stdout.unwrap().read_to_string(&mut password).unwrap();

    if clip {
        let mut clipboard = Clipboard::new().expect("Failed to access clipboard");
        clipboard
            .set_text(password)
            .expect("Failed to copy password to clipboard");

        println!("Password has been copied to clipboard.");
        println!("However, you might not be able to paste it anywhere.");
        println!("See https://github.com/1Password/arboard/blob/master/src/lib.rs#L49-L53 for more info.");
    } else {
        println!("{password}");
    }
}

pub fn edit(name: String) {
    let entry_path = get_vault_path().join(format!("{}.gpg", name));
    if !entry_path.exists() {
        stdout()
            .write(format!("no such entry: {}\n", name).as_bytes())
            .unwrap();
        exit(1);
    }

    let tmp_vault = PathBuf::from(env::var("TMPDIR").unwrap_or(String::from("/tmp"))).join("vault");
    if !tmp_vault.exists() {
        create_dir_all(&tmp_vault).expect("Failed to create tmp vault");
    }

    let tmp_entry_path = tmp_vault.join(&name);
    if !tmp_entry_path.parent().unwrap().exists() {
        create_dir_all(tmp_entry_path.parent().unwrap())
            .expect("Failed to create temp entry parent dir");
    }

    let tmp_entry = File::create(&tmp_entry_path).expect("Failed to create temp entry");

    Command::new("gpg")
        .args(["-q", "-d", entry_path.to_str().unwrap()])
        .stdout(Stdio::from(tmp_entry))
        .spawn()
        .expect("Failed to spawn gpg")
        .wait()
        .unwrap();

    let editor_cmd = env::var("EDITOR").unwrap_or(String::from("/usr/bin/vi"));

    Command::new(&editor_cmd)
        .arg(&tmp_entry_path)
        .spawn()
        .expect(format!("Failed to spawn {}", &editor_cmd).as_str())
        .wait()
        .unwrap();

    let mut tmp_entry = File::options()
        .read(true)
        .open(&tmp_entry_path)
        .expect("Failed to open temp entry in read-only mode");

    let mut entry_buf = String::new();
    tmp_entry
        .read_to_string(&mut entry_buf)
        .expect("Couldn't read entry to string");

    let vault_entry = File::options()
        .write(true)
        .open(entry_path)
        .expect("Failed to open entry file in write-only mode");

    let mut gpg = Command::new("gpg")
        .args(["-e", "-r", get_key_id().as_str()])
        .stdin(Stdio::piped())
        .stdout(Stdio::from(vault_entry))
        .spawn()
        .expect("Failed to encrypt entry");

    entry_buf = entry_buf.trim_end_matches("\n").to_string();

    gpg.stdin
        .as_mut()
        .unwrap()
        .write(entry_buf.as_bytes())
        .expect("Failed to write to gpg stdin");

    gpg.wait().unwrap();

    remove_file(&tmp_entry_path).expect("Failed to delete temp entry");
}

pub fn remove(name: String) {
    let ans: Result<bool> =
        match prompt("Are you sure you want to delete this entry? [y/N] ", false)
            .unwrap_or("".to_string())
            .as_ref()
        {
            "y" => Ok(true),
            "" | "n" | "N" => Ok(false),
            _ => Err(Error::new(ErrorKind::InvalidInput, "invalid input")),
        };

    if ans.is_err() {
        stdout()
            .write(ans.err().unwrap().to_string().as_bytes())
            .unwrap();
        exit(1);
    }

    if !ans.unwrap_or(false) {
        println!("Nothing to do");
        exit(0);
    }

    let entry_path = get_vault_path().join(format!("{name}.gpg"));
    if !entry_path.exists() {
        stdout()
            .write(format!("{name} does not exist in vault").as_bytes())
            .unwrap();
        exit(1);
    }

    remove_file(entry_path).expect(format!("Couldn't delete {name}").as_str());
}

fn canonicalize_path(p: PathBuf) -> PathBuf {
    let mut steps_up = 0;

    for part in p.display().to_string().split("/").into_iter() {
        if part == ".." {
            steps_up += 1;
        }
    }

    let mut resolved = p.clone();
    for _ in 0..=steps_up {
        resolved = resolved.parent().unwrap().to_path_buf();
    }

    return resolved.join(p.file_name().unwrap().to_str().unwrap());
}

pub fn move_entry(source: String, target: String) {
    let source_path = get_vault_path().join(format!("{}.gpg", &source));
    let target_path = canonicalize_path(get_vault_path().join(format!("{}.gpg", &target)));

    let target_path_parent = target_path.parent().unwrap();
    if !target_path_parent.exists() {
        create_dir_all(target_path_parent).expect("Failed to create target parent dir");
    }

    if let Err(mv_err) = rename(source_path, target_path) {
        stdout().write(mv_err.to_string().as_bytes()).unwrap();
        exit(1);
    }
}
