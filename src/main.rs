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

struct Identity {
    /// Description of the source (e.g., file path or "ssh-agent")
    source: String,
    /// The actual public key content
    content: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Resolve identity (file or ssh-agent)
    let identity = resolve_identity(args.identity_file)?;
    println!("Source: {}", identity.source);

    // Basic validation to ensure we are sending a public key
    if identity.content.trim().is_empty() {
        bail!("Identity content is empty.");
    }
    
    // Clean up the key content (trim whitespace) to avoid issues with newlines
    let clean_key_content = identity.content.trim().to_string() + "\n";

    println!("Target: {}", args.destination);

    // 2. Construct the remote command
    // We use a robust command sequence:
    // - umask 077: ensures created files are private
    // - mkdir -p .ssh && chmod 700 .ssh: ensures the dir exists with right perms
    // - loop over stdin lines to handle multiple keys (e.g. from ssh-add -L)
    // - grep -qxF: checks if the exact key line already exists
    let remote_cmd = "umask 077; mkdir -p .ssh && chmod 700 .ssh; \
                      if [ ! -f .ssh/authorized_keys ]; then touch .ssh/authorized_keys && chmod 600 .ssh/authorized_keys; fi; \
                      while read -r key; do \
                        if [ -n \"$key\" ]; then \
                          if ! grep -qxF \"$key\" .ssh/authorized_keys; then \
                            echo \"$key\" >> .ssh/authorized_keys; \
                          fi; \
                        fi; \
                      done";

    // 3. Execute SSH
    let mut command = Command::new("ssh");
    
    if let Some(port) = args.port {
        command.arg("-p").arg(port);
    }

    command
        .arg(&args.destination)
        .arg(remote_cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    println!("Info: Attempting to log in with the new key(s) to filter out any that are already installed...");

    let mut child = command.spawn()
        .context("Failed to spawn ssh process. Make sure 'ssh' is in your PATH.")?;

    // Pipe the key to the SSH process
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(clean_key_content.as_bytes())
            .context("Failed to write key to ssh stdin")?;
    }

    let status = child.wait().context("Failed to wait on ssh process")?;

    if status.success() {
        println!("\nNumber of key(s) added: 1 (check output above if multiple)");
        println!("\nNow try logging into the machine, with:   \"ssh '{}'\"", args.destination);
        println!("and check to make sure that only the key(s) you wanted were added.");
    } else {
        bail!("ssh process exited with error code: {:?}", status.code());
    }

    Ok(())
}

fn resolve_identity(input: Option<String>) -> Result<Identity> {
    if let Some(mut path_str) = input {
        // Expand ~ to home directory
        if path_str.starts_with("~/") || path_str.starts_with("~\\") {
            let home = dirs::home_dir().context("Could not determine home directory for ~ expansion")?;
            path_str = path_str.replacen('~', &home.to_string_lossy(), 1);
        }

        let path = PathBuf::from(&path_str);
        
        // Logic to find .pub file if private key path given
        let final_path = if path.exists() {
             if path_str.ends_with(".pub") {
                path
            } else {
                let pub_path = PathBuf::from(format!("{}.pub", path_str));
                if pub_path.exists() {
                    pub_path
                } else {
                    path // Fallback to original
                }
            }
        } else {
             // Try appending .pub
             let pub_path = PathBuf::from(format!("{}.pub", path_str));
             if pub_path.exists() {
                 pub_path
             } else {
                 bail!("Identity file not found: {}", path_str);
             }
        };

        let content = fs::read_to_string(&final_path)
            .with_context(|| format!("Failed to read identity file: {:?}", final_path))?;
            
        Ok(Identity {
            source: final_path.to_string_lossy().into_owned(),
            content,
        })

    } else {
        // Auto-discovery
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let ssh_dir = home.join(".ssh");

        let candidates = [
            "id_rsa.pub",
            "id_ed25519.pub",
            "id_ecdsa.pub",
            "id_dsa.pub",
            "identity.pub",
        ];

        for filename in candidates {
            let candidate = ssh_dir.join(filename);
            if candidate.exists() {
                let content = fs::read_to_string(&candidate)
                    .with_context(|| format!("Failed to read identity file: {:?}", candidate))?;
                return Ok(Identity {
                    source: candidate.to_string_lossy().into_owned(),
                    content,
                });
            }
        }

        // Try ssh-add -L
        if let Ok(output) = Command::new("ssh-add").arg("-L").output() {
            if output.status.success() {
                let keys = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !keys.is_empty() && !keys.contains("The agent has no identities") {
                     return Ok(Identity {
                        source: "ssh-agent".to_string(),
                        content: keys,
                    });
                }
            }
        }

        bail!("No identity file found in default locations and no keys in ssh-agent. Please specify one with -i.");
    }
}
