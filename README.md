# baofzf

`baofzf` is a simple fuzzy finder for [OpenBao](https://openbao.org)/[HashiCorp Vault](https://www.vaultproject.io/) based on [skim](https://github.com/skim-rs/skim) and [vaultrs](https://docs.rs/vaultrs/latest/vaultrs/).

## Current feature

* KV (v1 + v2)
  * List secrets
  * Read secrets
  * Edit secrets

### KV partial listing

In general, capability to list all secrets under a specific path is required for `baofzf` to work.

That said, OpenBao is able to filter list results based on the token capabilites [using `list_scan_response_keys_filter_path` parameter in the policy](https://openbao.org/docs/concepts/policies/#filtering-list-or-scan-results).

## Install

```
cargo install baofzf
```

## Usage

Set your environment variables and login to OpenBao/Vault:

```bash
export BAO_ADDR=http://127.0.0.1:8200
# export BAO_NAMESPACE=
bao login # export BAO_TOKEN=

# fuzz through all secrets in "my-kv/" mount at "my_app/" supbath
$name -m my-kv -p my-app
```

### Flags

```
Usage: baofzf [OPTIONS]

Options:
  -m, --mount <MOUNT>  KV mount path (e.g. "secret") [default: kv]
  -k, --kvv <KVV>      KV mount version [default: 2]
  -p, --path <PATH>    Path prefix to list secrets from (e.g. "myapp/") [default: ""]
  -q, --query <QUERY>  Optional initial query to pre-fill the skim prompt [default: ""]
  -h, --help           Print help
  -V, --version        Print version
```

## OpenBao / Vault Support

While `baofzf` should be compatible with both OpenBao and Vault, I mainly focus on OpenBao. Somethings my not fully work with Vault going forward.

The OpenBao environement variables take precedence over the Vault ones.

### Supported environment variables

* BAO_ADDR
* BAO_NAMESPACE
* BAO_TOKEN
* VAULT_ADDR
* VAULT_NAMESPACE
* VAULT_TOKEN

## Contributing

### Tests

Tests relies on spinning up an OpenBao container with [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs). When using podman, the `DOCKER_HOST` env var needs to point a podman socket:

`export DOCKER_HOST=unix://$XDG_RUNTIME_DIR/podman/podman.sock`
