#![allow(dead_code)]
#![allow(unused_variables)]

use crate::tests::openbao_testcontainer::OpenBao;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

const OPENBAO_IMAGE_NAME: &str = "quay.io/openbao/openbao";
const OPENBAO_IMAGE_TAG: &str = "2.5.5";
const OPENBAO_DEV_ROOT_TOKEN_ID: &str = "root";

pub async fn setup() -> Result<(VaultClient, ContainerAsync<OpenBao>), Box<dyn std::error::Error>> {
    let container = OpenBao::default().start().await.unwrap();

    let port = container.get_host_port_ipv4(8200).await.unwrap();

    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address(format!("http://127.0.0.1:{port}"))
            .token(OPENBAO_DEV_ROOT_TOKEN_ID)
            .build()
            .unwrap(),
    )?;

    Ok((client, container))
}
