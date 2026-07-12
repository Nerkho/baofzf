#![allow(unused_imports)]
use crate::{kv::list_secrets, tests::common};
use std::collections::HashMap;
use vaultrs::api::token::requests::CreateTokenRequest;
use vaultrs::client::VaultClient;
use vaultrs::client::VaultClientSettingsBuilder;
use vaultrs::kv1;
use vaultrs::kv2;
use vaultrs::sys::mount;
use vaultrs::sys::policy;
use vaultrs::token;

#[tokio::test]
async fn list_kvv2() -> Result<(), Box<dyn std::error::Error>> {
    let (client, _container) = common::setup().await.unwrap();

    let mut secret_list = Vec::from([
        "secret1",
        "secret2",
        "secret3",
        "app1/secret1",
        "app1/secret2",
        "app1/secret3",
        "app2/secret1",
        "app2/secret2",
        "app2/secret3",
    ]);

    mount::enable(&client, "kv2", "kv-v2", None).await?;

    let data = serde_json::json!({
        "foo": "bar".to_string()
    });

    for secret in &secret_list {
        kv2::set(&client, "kv2", secret, &data).await?;
    }

    let mut server_secret_list = list_secrets(&client, &"kv2", &"".to_string(), &"2").await?;

    secret_list.sort();
    server_secret_list.sort();

    assert_eq!(secret_list, server_secret_list);

    Ok(())
}

#[tokio::test]
async fn list_kvv1() -> Result<(), Box<dyn std::error::Error>> {
    let (client, _container) = common::setup().await.unwrap();

    let mut secret_list = Vec::from([
        "secret1",
        "secret2",
        "secret3",
        "app1/secret1",
        "app1/secret2",
        "app1/secret3",
        "app2/secret1",
        "app2/secret2",
        "app2/secret3",
    ]);

    mount::enable(&client, "kv1", "kv", None).await?;

    let data: HashMap<&str, serde_json::Value> = serde_json::from_str(r#"{"foo": "bar"}"#)?;

    for secret in &secret_list {
        kv1::set(&client, "kv1", secret, &data).await?;
    }

    let mut server_secret_list = list_secrets(&client, &"kv1", &"".to_string(), &"1").await?;

    secret_list.sort();
    server_secret_list.sort();

    assert_eq!(secret_list, server_secret_list);

    Ok(())
}
