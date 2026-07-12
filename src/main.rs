use crate::app::setup_skim;
use clap::Parser;
use kv::list_secrets;
use std::env;
use std::error::Error;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
mod app;
mod kv;
mod tests;

/// Interactive fuzzy search for OpenBao/Vault secrets
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// KV mount path (e.g. "secret")
    #[arg(short, long, default_value = "kv")]
    mount: String,

    /// KV mount version
    #[arg(short, long, default_value = "2")]
    kvv: String,

    /// Path prefix to list secrets from (e.g. "myapp/")
    #[arg(short, long, default_value = "")]
    path: String,

    /// Optional initial query to pre-fill the skim prompt
    #[arg(short, long, default_value = "")]
    query: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Setup OpenBao/Vault client
    let client = setup_client()?;

    println!("Fetching secrets from '{}/{}'...", args.mount, args.path);
    let keys = list_secrets(&client, &args.mount, &args.path, &args.kvv).await?;

    setup_skim(
        &keys,
        &client,
        &args.mount,
        &args.path,
        &args.query,
        &args.kvv,
    )
    .await?;

    Ok(())
}

/// Reads the BAO_ADDR/VAULT_ADDR, BAO_TOKEN/VAULT_TOKEN, and BAO_NAMESPACE/VAULT_NAMESPACE environment variables
/// and returns a vaultrs::VaultClient.
/// If BAO_TOKEN/VAULT_TOKEN are not set, attempt to read the token from ~/.vault-token.
/// The OpenBao environment variables take precedence over the VAULT ones.
fn setup_client() -> Result<VaultClient, Box<dyn Error>> {
    let addr = env::var("BAO_ADDR")
        .or_else(|_| env::var("VAULT_ADDR"))
        .map_err(|_| "BAO_ADDR/VAULT_ADDR not set".to_string())?;

    let mut token =
        env::var("BAO_TOKEN").unwrap_or_else(|_| env::var("VAULT_TOKEN").unwrap_or_default());

    // if BAO_TOKEN/VAULT_TOKEN are not set attempt to read the token from ~/.vault-token
    if token == "" {
        let token_path = format!(
            "{}/.vault-token",
            env::var("HOME").unwrap_or_else(|_| "~".to_string())
        );
        if let Ok(contents) = std::fs::read_to_string(&token_path) {
            token = contents.trim().to_string();
        } else {
            return Err(
                ("OpenBao/Vault token not found. Set BAO_TOKEN/VAULT_TOKEN environment variable and try again.")
                    .into(),
            );
        }
    }

    // if BAO_NAMESPACE/VAULT_NAMESPACE are not set, default to "root"
    let namespace = env::var("BAO_NAMESPACE")
        .unwrap_or_else(|_| env::var("VAULT_NAMESPACE").unwrap_or_else(|_| "root".to_string()));

    Ok(VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address(addr)
            .token(token)
            .namespace(Some(namespace))
            .build()
            .unwrap(),
    )
    .unwrap())
}
