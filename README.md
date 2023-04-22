# SudoStake

[![Rust](https://github.com/CodeMuhammed/sudostake_contracts/actions/workflows/rust.yml/badge.svg)](https://github.com/CodeMuhammed/sudostake_contracts/actions/workflows/rust.yml)

&nbsp;

SudoStake is a marketplace for trading liquidity request options created by vaults containing staked assets.

&nbsp;

| Contracts (tag: v0.1.0)                                                        | Description                                                        |
| :----------------------------------------------------------------------------- | :----------------------------------------------------------------- |
| [SudoMod](contracts/sudomod)                                                   | Creates and manages vaults and lp_groups                           |
| [LP-Group](contracts/lp_group)                                                 | Manages a group of liquidity providers                             |
| [Token-Swap](contracts/token-swap)                                             | Allows swapping between tokens                                     |
| [Vault](contracts/vault)                                                       | Allows users to manage staked tokens and request liquidity         |

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
