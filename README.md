# Vault &ndash; Password manager CLI

Vault is a passion project for me to learn Rust.

I would not suggest you actually use this to store your passwords.

Also, see [Known issues](#known-issues)

## Installation

### Prerequisites

To run installation, you need to have [`cargo`](https://www.rust-lang.org/tools/install) installed.

Additionally, make sure [`gpg`](https://gnupg.org) is installed.
This is required in order to perform file encryption.

### Install Vault

```sh
git clone https://github.com/aronhoyer/vault.git
cd vault
cargo build --release
sudo cp target/release/vault /usr/local/bin # or anywhere else in your $PATH
```

## Vault directory

Vault will use either `$VAULT_PATH` or default to `$HOME/.local/vault` if `$VAULT_PATH` is not set.

## Known issues

- The `--clip` option doesn't work on Linux due to an [issue with the clipboard dependency](https://github.com/1Password/arboard/blob/master/README.md?plain=1#L22-L28) being used (PRs more than welcome on this).

## License

Vault is licensed under the [MIT license](./LICENSE).
