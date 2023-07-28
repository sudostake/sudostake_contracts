## User accounts

```zsh
# SudoMod owner account
# archway1aswxn6sq79tkxc9hu3en3wxsqa7m79jm04fy74

archwayd keys add sudomod_owner
```

```zsh
# Vault owner account
# archway157ulj3k2mp5ckftu5v7eh7razy67us9af90wfp

archwayd keys add vault_owner
```

```zsh
# Lender account
# archway1aph2pa2fteuvux4a3t6hs9h0cz8pg93pdwphyd

archwayd keys add lender 
```

&nbsp;

## Deploy sudostake to testnet

```zsh
# 1. Build sudomod.wasm

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6
```

```zsh
# 2. Store sudomod.wasm on-chain
# TX_HASH = 6878EA185C35760D4A0A977371127057697C2396B54AFF79481775EF48F46B7D
# CODE_ID = 104

export RES=(archwayd tx wasm store artifacts/sudomod.wasm --from sudomod_owner $TXFLAG -y --output json -b block)
```

```zsh
# 3. Create an instance of sudomod contract
# TX_HASH = D4BB1A897256BE01C03262EB8721DDC669391C8A718B9BD3B4ACA9C25701A733

export INIT='{}'
archwayd tx wasm instantiate 104 "$INIT" --from sudomod_owner --label "SUDOMOD" $TXFLAG -y --no-admin

# export sudomod_contract_address for use in terminal
export SUDOMOD_ADDR='archway18jw94kucrgc20dzzt5mfs29cxksyufecc9p5dynmxfwwlfmuc32q7f2ht9'
```

```zsh
# 4. Set sudomod_contract_address as INSTANTIATOR_ADDR in vault contract code
# This ensures that only sudomod_contract_address can instantiate
# the vault contract code stored in step 6
```

```zsh
# 5. Build vault.wasm

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6
```

```zsh
# 6. Store vault.wasm on-chain
# TX_HASH = 42C17F8BF9C9F7652B58EE075080AFF2E8392383FE8AFAF17BBAB023AAB901F9
# CODE_ID = 105

export RES=$(archwayd tx wasm store artifacts/vault_contract.wasm --from sudomod_owner $TXFLAG -y --output json -b block)
```

```zsh
# 7. SetVaultCodeId on sudomod_contract_address
# TX_HASH = 49B667917875B0B9BB49665B3504DC5422A02DA922BBDFF6273BD35FFE830BE5

export E_PAYLOAD='{"set_vault_code_id":{"code_id":105}}'
archwayd tx wasm execute $SUDOMOD_ADDR "$E_PAYLOAD" --from sudomod_owner $NODE $TXFLAG -y
```

```zsh
# 8. SetVaultCreationFee on sudomod_contract_address
# TX_HASH = EB44A063087124AD8D876C742105C638129488E455B86E7CE226439ED67391F0

export E_PAYLOAD='{"set_vault_creation_fee":{"amount":{"denom":"aconst","amount":"10000000000000000000"}}}'
archwayd tx wasm execute $SUDOMOD_ADDR "$E_PAYLOAD" --from sudomod_owner $NODE $TXFLAG -y
```

```zsh
# 9. Query info from sudomod_contract_address

export Q_PAYLOAD='{"info":{}}'
archwayd query wasm contract-state smart $SUDOMOD_ADDR "$Q_PAYLOAD" $NODE --output json
```

&nbsp;

## How to use a vault

```zsh
# 1. Mint Vault 
# TX_HASH = D5D368684C5DC95C95669F9A40C61FCC629D5BF290C9C4B26CF6123E10263251

export E_PAYLOAD='{"mint_vault":{}}'
archwayd tx wasm execute $SUDOMOD_ADDR "$E_PAYLOAD" --from vault_owner --amount=10000000000000000000aconst $NODE $TXFLAG -y

# export vault_address for use in terminal
export VAULT_ADDR='archway143rrddv6yahhc6nxsnfwl49t63xvv7llzctgeuwh8g9jsyk2338q20kgyj'
```

```zsh
# 2. Delegate 10,000ARCH to testnet_validator = archwayvaloper1sk23ewl2kzfu9mfh3sdh6gpm9xkq56m7tjnl25
# TX_HASH = 8AE3C4D49A67155F0D8EFD648C1547D1BB69375727048C4913CCF269546BD1D1
# TX_HASH = 8F7D5FFC8730724237CE5537DD26DBF476FA064D94C4CFBC03F7C9B292E629AE

export E_PAYLOAD='{"delegate":{"validator":"archwayvaloper1sk23ewl2kzfu9mfh3sdh6gpm9xkq56m7tjnl25", "amount":"9500000000000000000000"}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from vault_owner --amount=9500000000000000000000aconst $NODE $TXFLAG -y
```

&nbsp;

## Test fixed term rental

```zsh
# 1. vault_owner opens a fixed term rental that expires 3600 seconds after being accepted
# TX_HASH = AFA69B92924C4A16056E782F2DE527A4FE482BF4B5623767180655130058C6E5

export E_PAYLOAD='{"request_liquidity":{"option":{"fixed_term_rental":{"duration_in_seconds":3600,"can_cast_vote":false,"requested_amount":{"denom":"aconst","amount":"1000000000000000000"}}}}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from vault_owner $NODE $TXFLAG -y
```

```zsh
# 2. Lender accepts the fixed term rental by sending the requested amount to the vault
# TX_HASH = 1D46D9C79977A7EE0E641C1C925D568CAC15F1C818F27817808F02770519E74D

export E_PAYLOAD='{"accept_liquidity_request":{}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from lender --amount=1000000000000000000aconst $NODE $TXFLAG -y
```

```zsh
# 3. Lender waits for the fixed term rental to expire then calls claim delegator rewards.
#    The option is closed and lender gets the rewards claimed proportional to the duration they are entitled to
# TX_HASH = E7698B3572C101C1F9AA15C192E54AA9DBAFA57BFED2BB50424EE8168659A8C8

export E_PAYLOAD='{"claim_delegator_rewards":{}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from lender $NODE $TXFLAG -y
```

&nbsp;

## Test fixed interest rental

```zsh
# 1. vault_owner opens a fixed interest rental
# TX_HASH = A38F2D3733A8DDE88129D3B90B6A6D03128C2C8504C76F77D73C3CBC1FD12E17

export E_PAYLOAD='{"request_liquidity":{"option":{"fixed_interest_rental":{"claimable_tokens":"120000000000000000000","can_cast_vote":false,"requested_amount":{"denom":"aconst","amount":"1000000000000000000"}}}}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from vault_owner $NODE $TXFLAG -y
```

```zsh
# 2. Lender accepts the fixed interest rental by sending the requested amount to the vault
# TX_HASH = 324D0A5F8709F65181DBC4B884AB83394DFD8B831F656184006A16ABC0372C4C

export E_PAYLOAD='{"accept_liquidity_request":{}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from lender --amount=1000000000000000000aconst $NODE $TXFLAG -y
```


```zsh
# 3. Lender can call claim_delegator_rewards until claimable_tokens is complete
# TX_HASH = D3C9D65BCEDF66414F722BAB683998D58095D226DAFCC5A89596B7FC3F630CAB
# TX_HASH = 8BA55640087E574C1157D1EE2E255694F1374A3B79604EFBB932A83CB00B0C84

export E_PAYLOAD='{"claim_delegator_rewards":{}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from lender $NODE $TXFLAG -y
```

&nbsp;

## Test fixed term loan with repayment

```zsh
# 1. vault_owner opens a fixed term loan
# TX_HASH = 812EFD5590F6E6B4EEFCDEB3D69D9BFDAFC3DF2BFC305D7A758FCD7C783965E3

export E_PAYLOAD='{"request_liquidity":{"option":{"fixed_term_loan":{"duration_in_seconds":3600,"collateral_amount":"1000000000000000000000","interest_amount":"1000000000000000000","requested_amount":{"denom":"aconst","amount":"1000000000000000000"}}}}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from vault_owner $NODE $TXFLAG -y
```

```zsh
# 2. Lender accepts the fixed term loan by sending the requested amount to the vault
# TX_HASH = A30DC96180CB7E35930748E1FB7150963733D08C0F0D10BF099792018060EC53

export E_PAYLOAD='{"accept_liquidity_request":{}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from lender --amount=1000000000000000000aconst $NODE $TXFLAG -y
```

```zsh
# 3. Wait for the loan to expire then vault_owner call repay loan pricipal + interest
# TX_HASH = 26ADBA722C85377889531A5B71DDFD31C51FE20D24E9F1362C5822E70FA7D45A

export E_PAYLOAD='{"repay_loan":{}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from vault_owner $NODE $TXFLAG -y
```

&nbsp;

## Test fixed term loan with collateral liquidation

```zsh
# 1. vault_owner opens a fixed term loan
# TX_HASH = 464816927AD371098A817B04991E6403556F3240287E48B5C6462BC7346ABDEA

export E_PAYLOAD='{"request_liquidity":{"option":{"fixed_term_loan":{"duration_in_seconds":3600,"collateral_amount":"1000000000000000000000","interest_amount":"1000000000000000000","requested_amount":{"denom":"aconst","amount":"1000000000000000000"}}}}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from vault_owner $NODE $TXFLAG -y
```

```zsh
# 2. Lender accepts the fixed term loan by sending the requested amount to the vault
# TX_HASH = BCF3D5466FE19FE77EA6425A21E1E6FD44D473CB2657375AE1C99897114F579E

export E_PAYLOAD='{"accept_liquidity_request":{}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from lender --amount=1000000000000000000aconst $NODE $TXFLAG -y
```

```zsh
# 3. Wait for the the fixed term loan to expire, Lender begins the liquidation process
#    which clears the available balance plus accumulated rewards.
#    Then issues an unbonding request if there is still an outstanding debt
# TX_HASH = 58B434C80685190EA09859F6210E649BD1D4D225AC93E856D12059B9B4E9D133

export E_PAYLOAD='{"liquidate_collateral":{}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from lender $NODE $TXFLAG -y
```

```zsh
# 4. Wait for the unbonding period of the outstanding debt to be over, 
#    Then Lender calls liquidate collateral again which sends the available balance to them
# TX_HASH = 60FD147F33B8FC5435178D90869BCA36B540405624DF7D6A16376294418F2C11

export E_PAYLOAD='{"liquidate_collateral":{}}'
archwayd tx wasm execute $VAULT_ADDR "$E_PAYLOAD" --from lender $NODE $TXFLAG -y
```
