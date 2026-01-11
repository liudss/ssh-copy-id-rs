# ssh-copy-id-rs

## Project Overview
`ssh-copy-id-rs` is a Rust implementation of the classic `ssh-copy-id` shell script. It automates the process of installing your public key on a remote server's `authorized_keys` file, allowing for password-less SSH login.

**Key Features:**
*   **Cross-Platform:** Designed to run on Windows, Linux, and macOS (requires `ssh` client in PATH).
*   **Key Auto-Discovery:** Automatically searches for common public key names (e.g., `id_rsa.pub`, `id_ed25519.pub`) in `~/.ssh` if not specified.
*   **Single-Binary:** compiles to a standalone executable.

## Building and Running

### Prerequisites
*   **Rust:** Ensure you have the Rust toolchain installed (stable).
*   **SSH Client:** An `ssh` executable must be available in your system's `PATH`.

### Build Commands
*   **Build (Dev):** `cargo build`
*   **Build (Release):** `cargo build --release`
*   **Run:** `cargo run -- [ARGS]`
*   **Test:** `cargo test`

## Usage

```bash
cargo run -- [OPTIONS] <DESTINATION>
```

**Arguments:**
*   `<DESTINATION>`: The remote destination (e.g., `user@host`).

**Options:**
*   `-i, --identity-file <FILE>`: path to the identity file (public or private key). If omitted, the tool attempts to auto-discover standard keys in `~/.ssh`.
*   `-p, --port <PORT>`: Port to connect to on the remote host.
*   `-h, --help`: Print help.

## Codebase Structure

*   **`src/main.rs`**: Contains the entire implementation, including:
    *   Argument parsing (via `clap`).
    *   Identity file resolution logic (`resolve_identity_file`).
    *   SSH command construction and execution.
*   **`Cargo.toml`**: Project configuration and dependencies (`anyhow`, `clap`, `dirs`).

## CI/CD
The project uses GitHub Actions (`.github/workflows/ci.yml`) to ensure the code builds and tests pass on Ubuntu, Windows, and macOS.
