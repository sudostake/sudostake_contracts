# SudoStake

[![Rust](https://github.com/CodeMuhammed/sudostake_contracts/actions/workflows/rust.yml/badge.svg)](https://github.com/CodeMuhammed/sudostake_contracts/actions/workflows/rust.yml)

&nbsp;

Non-Custodial | Smart Contract Staking | Peer-to-Peer Options Trading Platform.

&nbsp;

| Contracts                                                                      | Description                                                        |
| :----------------------------------------------------------------------------- | :----------------------------------------------------------------- |
| [SudoMod](contracts/sudomod)                                                   | Proxy for minting vaults                                           |
| [Vault](contracts/vault)                                                       | Staking with peer-to-peer options trading                          |

&nbsp;

## Preparing for merge

Before you merge the code, make sure it builds and passes all tests using the command below.

`$ cargo test`

&nbsp;

## Release builds

You can build release artifacts manually like this, which creates a reproducible
optimized build for each contract and saves them to the `./artifacts` directory:

```zsh
$ docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6
```

&nbsp;

## Working with smart contracts (archwayd)

(See [instructions](https://docs.archway.io/developers) on how install `archwayd`)

&nbsp;

### View archwayd config variables

```zsh
$ open ~/.archway/config/config.toml
```

&nbsp;

### Testnet block explorer

[Constantine-3 test-net on mintscan](https://testnet.mintscan.io/archway-testnet) 

&nbsp;

### Add test accounts

```zsh
$ archwayd keys add <wallet_name> --recover
```

&nbsp;

### List test accounts

```zsh
$ archwayd keys list
```

&nbsp;

### Export variables for use in terminal - Constantine-3

```zsh
# Export path to your go installation
$ source ~/.profile

$ export CHAIN_ID="constantine-3"

$ export RPC="https://rpc.constantine.archway.tech:443"

$ export NODE=(--node $RPC)

$ export TXFLAG=($NODE --chain-id $CHAIN_ID --gas-prices 20000000000000aconst --gas auto --gas-adjustment 1.3)
```

&nbsp;

### Export variables for use in terminal - Triomphe

```zsh
# export path to your go installation
$ source ~/.profile

$ export CHAIN_ID="archway-1"

$ export RPC="https://rpc.mainnet.archway.io:443"

$ export NODE=(--node $RPC)

$ export TXFLAG=($NODE --chain-id $CHAIN_ID --gas-prices 900000000000aarch --gas auto --gas-adjustment 1.3)
```

&nbsp;

### Query balances

```zsh
$ archwayd query bank total $NODE

$ archwayd query bank balances $(archwayd keys show -a palingram) $NODE
```

&nbsp;

### Send funds to other account

```zsh
$ archwayd tx bank send <sender_account_name> <receiver_address> <amount><denom> $TXFLAG
```

&nbsp;

### See the list of code uploaded to the testnet

```zsh
$ archwayd query wasm list-code $NODE
```

&nbsp;


### Store the contract on the blockchain and get the <CODE_ID>

```zsh
$ export RES=$(archwayd tx wasm store artifacts/<contract_name.wasm> --from <account_name> $TXFLAG -y --output json -b block)

$ export CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[1].value')

$ echo $CODE_ID
```

&nbsp;

### Get a list of contracts instantiated for <CODE_ID>

```zsh
$ archwayd query wasm list-contract-by-code $CODE_ID $NODE --output json
```

&nbsp;

### Prepare the json message payload

```zsh
$ export INIT='{}'
```

&nbsp;

### Instantiate the contract

```zsh
$ archwayd tx wasm instantiate $CODE_ID "$INIT" --from <account_name> --label "CONTRACT LABEL" $TXFLAG -y --no-admin
```

&nbsp;

### Get the latest contract instantiated for contract with $CODE_ID

```zsh
$ export CONTRACT=$(archwayd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')

$ echo $CONTRACT
```

&nbsp;

### Check the contract details

```zsh
$ archwayd query wasm contract $CONTRACT $NODE
```

&nbsp;

### Check the contract balance

```zsh
$ archwayd query bank balances $CONTRACT $NODE
```

&nbsp;

### query the entire contract state

```zsh
$ archwayd query wasm contract-state all $CONTRACT $NODE
```

&nbsp;

### query the data for a storage key in the contract-state directly

```zsh
$ archwayd query wasm contract-state raw $CONTRACT 636F6E666967 $NODE  --output "json" | jq -r '.data' | base64 -d
```

&nbsp;

### Calling execute methods

```zsh
$ export E_PAYLOAD='{"<prop>":{}}'

$ archwayd tx wasm execute $CONTRACT "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

&nbsp;

### calling query methods

```zsh
$ export Q_PAYLOAD='{"query_list":{}}'

$ archwayd query wasm contract-state smart $CONTRACT "$Q_PAYLOAD" $NODE --output json
```

&nbsp;

## Testnet Docs

See [TESTNET.md](./TESTNET.md) for the complete SudoStake smart contracts testing docs
