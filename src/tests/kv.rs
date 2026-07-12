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
async fn test_list_kvv2() -> Result<(), Box<dyn std::error::Error>> {
    let (client, _container) = common::setup().await.unwrap();

    let mut secret_list = [
        "secret1",
        "secret2",
        "secret3",
        "app1/secret1",
        "app1/secret2",
        "app1/secret3",
        "app2/secret1",
        "app2/secret2",
        "app2/secret3",
    ];

    mount::enable(&client, "kv2", "kv-v2", None).await?;

    let data = serde_json::json!({
        "foo": "bar".to_string()
    });

    for secret in secret_list {
        kv2::set(&client, "kv2", secret, &data).await?;
    }

    let mut server_secret_list = list_secrets(
        &client,
        &"kv2".to_string(),
        &"".to_string(),
        &"2".to_string(),
    )
    .await?;

    assert_eq!(secret_list.sort(), server_secret_list.sort());

    Ok(())
}

#[tokio::test]
async fn test_list_kvv1() -> Result<(), Box<dyn std::error::Error>> {
    let (client, _container) = common::setup().await.unwrap();

    let mut secret_list = [
        "secret1",
        "secret2",
        "secret3",
        "app1/secret1",
        "app1/secret2",
        "app1/secret3",
        "app2/secret1",
        "app2/secret2",
        "app2/secret3",
    ];

    mount::enable(&client, "kv1", "kv", None).await?;

    let data: HashMap<&str, serde_json::Value> = serde_json::from_str(r#"{"foo": "bar"}"#)?;

    for secret in secret_list {
        kv1::set(&client, "kv1", secret, &data).await?;
    }

    let mut server_secret_list = list_secrets(
        &client,
        &"kv1".to_string(),
        &"".to_string(),
        &"1".to_string(),
    )
    .await?;

    assert_eq!(secret_list.sort(), server_secret_list.sort());

    Ok(())
}

#[tokio::test]
async fn test_partial_list_kvv2() -> Result<(), Box<dyn std::error::Error>> {
    let (client, _container) = common::setup().await.unwrap();

    let secret_list = [
        "secret1",
        "secret2",
        "secret3",
        "app1/secret1",
        "app1/secret2",
        "app1/secret3",
        "app2/secret1",
        "app2/secret2",
        "app2/secret3",
    ];

    mount::enable(&client, "kv2", "kv-v2", None).await?;

    let data = serde_json::json!({
        "foo": "bar".to_string()
    });

    for secret in secret_list {
        kv2::set(&client, "kv2", secret, &data).await?;
    }

    let mut expected_secret_list = [
        "secret1",
        "secret2",
        "secret3",
        "app2/secret1",
        "app2/secret2",
        "app2/secret3",
    ];

    let policy = r#"
        path "kv2/metadata/*" {
            capabilities = ["read","list"]
        }
        path "kv2/metadata/app1/*" {
            capabilities = ["deny"]
        }
        "#;

    policy::set(&client, "partial-policy", policy).await?;

    let mut token_builder = CreateTokenRequest::builder();
    token_builder
        .policies(vec!["partial-policy".to_string()])
        .ttl("15m");
    let token = token::new(&client, Some(&mut token_builder)).await?;

    let new_client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address(client.settings.address)
            .token(&token.client_token)
            .build()?,
    )?;

    let mut server_secret_list = list_secrets(
        &new_client,
        &"kv2".to_string(),
        &"".to_string(),
        &"2".to_string(),
    )
    .await?;

    assert_eq!(expected_secret_list.sort(), server_secret_list.sort());

    Ok(())
}

#[tokio::test]
async fn partial_list_kvv1() -> Result<(), Box<dyn std::error::Error>> {
    let (client, _container) = common::setup().await.unwrap();

    let secret_list = [
        "secret1",
        "secret2",
        "secret3",
        "app1/secret1",
        "app1/secret2",
        "app1/secret3",
        "app2/secret1",
        "app2/secret2",
        "app2/secret3",
    ];

    mount::enable(&client, "kv1", "kv", None).await?;

    let data: HashMap<&str, serde_json::Value> = serde_json::from_str(r#"{"foo": "bar"}"#)?;

    for secret in secret_list {
        kv1::set(&client, "kv1", secret, &data).await?;
    }

    let mut expected_secret_list = [
        "secret1",
        "secret2",
        "secret3",
        "app2/secret1",
        "app2/secret2",
        "app2/secret3",
    ];

    let policy = r#"
        path "kv1/*" {
            capabilities = ["read","list"]
        }
        path "kv1/app1/*" {
            capabilities = ["deny"]
        }
        "#;

    policy::set(&client, "partial-policy", policy).await?;

    let mut token_builder = CreateTokenRequest::builder();
    token_builder
        .policies(vec!["partial-policy".to_string()])
        .ttl("15m");
    let token = token::new(&client, Some(&mut token_builder)).await?;

    let new_client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address(client.settings.address)
            .token(&token.client_token)
            .build()?,
    )?;

    let mut server_secret_list = list_secrets(
        &new_client,
        &"kv1".to_string(),
        &"".to_string(),
        &"1".to_string(),
    )
    .await?;

    assert_eq!(expected_secret_list.sort(), server_secret_list.sort());

    Ok(())
}
