use anyhow::{Context, Result, bail};
use clap::Parser;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(name = "ssh-copy-id-rs")]
#[command(about = "A Rust implementation of ssh-copy-id", long_about = None)]
struct Args {
    /// Identity file, e.g., ~/.ssh/id_rsa.pub
    #[arg(short = 'i', long)]
    identity_file: Option<String>,

    /// Port to connect to on the remote host
    #[arg(short = 'p', long)]
    port: Option<String>,

    /// The remote destination (user@host)
    #[arg(required = true)]
    destination: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Locate the public key
    let pub_key_path = resolve_identity_file(args.identity_file)?;
    println!("Source: {}", pub_key_path.display());

    // 2. Read the public key content
    let key_content = fs::read_to_string(&pub_key_path)
        .with_context(|| format!("Failed to read identity file: {:?}", pub_key_path))?;

    // Basic validation to ensure we are sending a public key
    if !key_content.contains("ssh-") && !key_content.contains("ecdsa-") {
        eprintln!("Warning: The file '{}' does not look like a public key.", pub_key_path.display());
    }
    
    // Clean up the key content (trim whitespace) to avoid issues with newlines
    let clean_key_content = key_content.trim().to_string() + "\n";

    println!("Target: {}", args.destination);

    // 3. Construct the remote command
    // We use a robust command sequence:
    // - umask 077: ensures created files are private
    // - mkdir -p .ssh && chmod 700 .ssh: ensures the dir exists with right perms
    // - grep -qxF: checks if the exact key line already exists
    let remote_cmd = "umask 077; mkdir -p .ssh && chmod 700 .ssh; \
                      if [ ! -f .ssh/authorized_keys ]; then touch .ssh/authorized_keys && chmod 600 .ssh/authorized_keys; fi; \
                      key=$(cat); \
                      if ! grep -qxF \"$key\" .ssh/authorized_keys; then \
                        echo \"$key\" >> .ssh/authorized_keys; \
                      fi";

    // 4. Execute SSH
    let mut command = Command::new("ssh");
    
    if let Some(port) = args.port {
        command.arg("-p").arg(port);
    }

    // Add verbose flag if you want to see ssh output, but standard ssh-copy-id is usually quiet-ish
    // command.arg("-v");

    command
        .arg(&args.destination)
        .arg(remote_cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    println!("Executing: {:?}", command);

    let mut child = command.spawn()
        .context("Failed to spawn ssh process. Make sure 'ssh' is in your PATH.")?;

    // Pipe the key to the SSH process
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(clean_key_content.as_bytes())
            .context("Failed to write key to ssh stdin")?;
    }

    let status = child.wait().context("Failed to wait on ssh process")?;

    if status.success() {
        println!("\nNumber of key(s) added: 1");
        println!("\nNow try logging into the machine, with:   \"ssh '{}'\"", args.destination);
        println!("and check to make sure that only the key(s) you wanted were added.");
    } else {
        bail!("ssh process exited with error code: {:?}", status.code());
    }

    Ok(())
}

fn resolve_identity_file(input: Option<String>) -> Result<PathBuf> {
    if let Some(mut path_str) = input {
        // Expand ~ to home directory
        if path_str.starts_with("~/") || path_str.starts_with("~\\") {
            let home = dirs::home_dir().context("Could not determine home directory for ~ expansion")?;
            path_str = path_str.replacen('~', &home.to_string_lossy(), 1);
        }

        // If the user provided a path, check if it's the public key or private key
        let path = PathBuf::from(&path_str);
        if path.exists() {
            // If it ends in .pub, assume it's the one we want
            if path_str.ends_with(".pub") {
                return Ok(path);
            } 
            // If it's the private key (no .pub), try to find the .pub counterpart
            let pub_path = PathBuf::from(format!("{}.pub", path_str));
            if pub_path.exists() {
                return Ok(pub_path);
            }
            // Fallback: Use the file provided, maybe they named it non-standardly
            return Ok(path);
        } else {
            // If the user provided a file ending in .pub that doesn't exist, fail.
            // If they provided a name like "id_rsa", try adding ".pub"
            let pub_path = PathBuf::from(format!("{}.pub", path_str));
            if pub_path.exists() {
                return Ok(pub_path);
            }
             bail!("Identity file not found: {}", path_str);
        }
    } else {
        // Auto-discovery
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let ssh_dir = home.join(".ssh");

        // Priority list matches standard ssh behavior roughly
        let candidates = [
            "id_rsa.pub",
            "id_ed25519.pub",
            "id_ecdsa.pub",
            "id_dsa.pub",
            "identity.pub", // Legacy
        ];

        for filename in candidates {
            let candidate = ssh_dir.join(filename);
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        bail!("No identity file found in default locations. Please specify one with -i.");
    }
}