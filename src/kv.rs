use json_to_table::json_to_table;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use vaultrs::client::VaultClient;
use vaultrs::kv1;
use vaultrs::kv2;

/// Lists all secrets under the given mount and path recursively.
/// Returns a list of secret and their path relative to the mount point.
pub async fn list_secrets(
    client: &VaultClient,
    mount: &str,
    path: &String,
    kvv: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut secret_list = Vec::new();

    if kvv == "2" {
        for item in kv2::list(client, mount, path).await? {
            if item.ends_with("/") {
                let s = Box::pin(list_secrets(
                    client,
                    mount,
                    &format!("{}{}", path, item),
                    kvv,
                ))
                .await?;
                secret_list.extend(s.iter().map(|i| format!("{}{}", item, i)));
            } else {
                secret_list.push(item);
            }
        }
    }
    if kvv == "1" {
        let kv_v1_list = kv1::list(client, mount, path).await?;
        for item in kv_v1_list.data.keys {
            if item.ends_with("/") {
                let s = Box::pin(list_secrets(
                    client,
                    mount,
                    &format!("{}{}", path, item),
                    kvv,
                ))
                .await?;
                secret_list.extend(s.iter().map(|i| format!("{}{}", item, i)));
            } else {
                secret_list.push(item);
            }
        }
    }

    Ok(secret_list)
}

/// Read secret, transform json into a nice table using json_to_table and return it as String.
pub async fn read_secret(
    client: &VaultClient,
    mount: &str,
    path: &str,
    kvv: &str,
) -> Result<String, Box<dyn Error>> {
    let secret = match kvv {
        "2" => kv2::read::<serde_json::Value>(client, mount, path).await?,
        "1" => kv1::get::<serde_json::Value>(client, mount, path).await?,
        _ => return Err("Unknown KV version".into()),
    };

    let table = json_to_table(&json!(secret)).to_string();

    Ok(table)
}

/// Fetch selected secret and open the default editor. Update secret upon leaving the editor.
pub async fn edit_secret(
    client: &VaultClient,
    mount: &str,
    path: &str,
    kvv: &str,
) -> Result<(), Box<dyn Error>> {
    let current_secret = match kvv {
        "2" => kv2::read::<serde_json::Value>(client, mount, path).await?,
        "1" => kv1::get::<serde_json::Value>(client, mount, path).await?,
        _ => return Err("Unknown KV version".into()),
    };

    let edited = match edit::edit(serde_json::to_string_pretty(&current_secret)?) {
        Ok(edited) => edited,
        Err(e) => return Err(format!("Failed to open secret in default editor - {e}").into()),
    };

    match kvv {
        "2" => {
            let kv2_value: serde_json::Value = match serde_json::from_str(&edited) {
                Ok(value) => value,
                Err(e) => return Err(format!("Failed to parse edited secret - {e}").into()),
            };

            match kv2::set(client, mount, path, &kv2_value).await {
                Ok(_) => {}
                Err(e) => return Err(format!("Failed to update secret - {e}").into()),
            };
        }
        "1" => {
            let kv1_value: HashMap<&str, serde_json::Value> = match serde_json::from_str(&edited) {
                Ok(value) => value,
                Err(e) => return Err(format!("Failed to parse edited secret - {e}").into()),
            };
            match kv1::set(client, mount, path, &kv1_value).await {
                Ok(_) => {}
                Err(e) => return Err(format!("Failed to update secret - {e}").into()),
            };
        }
        _ => return Err("Unknown KV version".into()),
    }

    Ok(())
}
