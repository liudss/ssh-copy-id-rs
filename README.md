# ssh-copy-id-rs

A fast, cross-platform Rust implementation of the classic `ssh-copy-id` tool.

`ssh-copy-id-rs` allows you to install your SSH public keys on a remote server's `authorized_keys` file with ease, enabling secure, password-less logins.

## Features

- **Cross-Platform**: Works on Windows, Linux, and macOS.
- **Auto-Discovery**: Automatically finds your public keys in `~/.ssh/` if no identity file is specified.
- **Simple & Secure**: Uses a robust command sequence to ensure `.ssh` directory permissions are set correctly on the remote host.

## Installation

### From Source

Ensure you have [Rust](https://www.rust-lang.org/) installed, then:

```bash
git clone https://github.com/liudss/ssh-copy-id-rs.git
cd ssh-copy-id-rs
cargo build --release
```

The binary will be available at `target/release/ssh-copy-id-rs`.

## Usage

```bash
ssh-copy-id-rs [OPTIONS] <DESTINATION>
```

### Examples

**Basic usage (auto-discovers your key):**
```bash
ssh-copy-id-rs user@192.168.1.10
```

**Specifying a specific identity file:**
```bash
ssh-copy-id-rs -i ~/.ssh/id_ed25519.pub user@example.com
```

**Connecting via a custom port:**
```bash
ssh-copy-id-rs -p 2222 user@example.com
```

### Options

- `-i, --identity-file <FILE>`: Path to the public key file.
- `-p, --port <PORT>`: SSH port on the remote host.
- `-h, --help`: Print help information.

## Requirements

- **Local**: `ssh` client must be in your `PATH`.
- **Remote**: The remote server must have an SSH server running and allow password/interactive login for the initial setup.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details (if applicable).
