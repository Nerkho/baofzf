use crate::kv::{edit_secret, read_secret};
use crossterm::event::{KeyCode, KeyModifiers};
use skim::prelude::*;
use std::error::Error;
use std::sync::Arc;
use vaultrs::client::VaultClient;

// A simple wrapper so our secret keys implement SkimItem
struct SecretItem(String);

impl SkimItem for SecretItem {
    fn text(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(&self.0)
    }
}

pub async fn setup_skim(
    keys: &[String],
    client: &VaultClient,
    mount: &String,
    path: &String,
    query: &String,
    kvv: &String,
) -> Result<(), Box<dyn Error>> {
    // Feed keys into skim via a channel
    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    let item = keys
        .iter()
        .map(|s| Arc::new(SecretItem(s.clone())) as Arc<dyn SkimItem>)
        .collect();
    tx.send(item).unwrap();

    // Drop to skim
    drop(tx);

    // Configure and run skim
    let options = SkimOptionsBuilder::default()
        .height("75%".to_string())
        .multi(false)
        .query(query)
        .prompt(">".to_string())
        .bind(vec![
            "enter:accept".to_string(),
            "ctrl-e:accept".to_string(),
        ])
        .header("Enter: view | Ctrl-E: edit")
        .build()
        .unwrap();

    match Skim::run_with(options, Some(rx)) {
        Ok(output) if !output.is_abort => {
            if let Some(selection) = output.selected_items.first() {
                let selection = selection.output().to_string();
                let full_path = format!("{}/{}", path.trim_end_matches('/'), selection);
                handle_key_event(output, client, mount, full_path, kvv).await?;
            }
        }
        Ok(_) => println!("No selection made."),
        Err(e) => return Err(format!("Skim error - {e}").into()),
    }

    Ok(())
}

async fn handle_key_event(
    output: SkimOutput,
    client: &VaultClient,
    mount: &String,
    full_path: String,
    kvv: &String,
) -> Result<(), Box<dyn Error>> {
    match (output.final_key.code, output.final_key.modifiers) {
        (KeyCode::Enter, _) => {
            println!("\nSelected secret: {mount}{full_path}");
            match read_secret(client, mount, &full_path, kvv).await {
                Ok(secret) => {
                    println!("{secret}");
                }
                Err(e) => {
                    return Err(format!("Could not read secret value - {e}").into());
                }
            }
        }
        (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
            edit_secret(client, mount, &full_path, kvv).await?;
            match read_secret(client, mount, &full_path, kvv).await {
                Ok(secret) => {
                    println!("\nEdited secret: {mount}{full_path}");
                    println!("{secret}",);
                }
                Err(e) => return Err(format!("Could not read secret value - {e}").into()),
            }
        }
        _ => {}
    }
    Ok(())
}
