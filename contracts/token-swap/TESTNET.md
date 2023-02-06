## Store contract code on testnet

```zsh
export RES=$(chihuahuad tx wasm store artifacts/<contract_name.wasm> --from <account_name> $TXFLAG -y --output json -b block)

export CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[1].value')

echo $CODE_ID
```

`cw20_code_id for cw20_base.wasm : 10`

`amm_code_id for token_swap.wasm : 11`

&nbsp;

## Instantiate contracts

```zsh
$ export INIT='<init_msg>'

$ chihuahuad tx wasm instantiate $CODE_ID "$INIT" --from <account_name> --label "<contract label>" $TXFLAG -y --no-admin

$ export CONTRACT=$(chihuahuad query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')

$ echo $CONTRACT
```

#### Cw20 token (SUDO)

`SUDO_CONTRACT = 'chihuahua1999u8suptza3rtxwk7lspve02m406xe7l622erg3np3aq05gawxsya7444'`

```javascript
let init_msg = JSON.stringify({
    name: 'Sudo Stake',
    symbol: 'SUDO',
    decimals: 6,
    initial_balances: [{
        address: '<palingram>',
        amount: '1000000000000'
    }],
    mint: {
        minter: '<palingram>',
        cap: '2000000000000',
    },
    marketing: {
        project: 'Sudo Stake',
        description: 'Self Custodial Liquid Staking',
        marketing: '<palingram>',
        logo: {
            url: 'https://example.com/image.jpg',
        },
    },
});
```

#### Cw20 token (PGRM)

`PGRM_CONTRACT = 'chihuahua13we0myxwzlpx8l5ark8elw5gj5d59dl6cjkzmt80c5q5cv5rt54qw87x4e'`

```javascript
let init_msg = JSON.stringify({
    name: 'Palingram',
    symbol: 'PGRM',
    decimals: 6,
    initial_balances: [{
        address: '<palingram>',
        amount: '1000000000000'
    }],
    mint: {
        minter: '<palingram>',
        cap: '2000000000000',
    },
    marketing: {
        project: 'Palingram Inc.',
        description: 'Smart contracts implementation / auditing',
        marketing: '<palingram>',
        logo: {
            url: 'https://example.com/image.jpg',
        },
    },
});
```

#### Amm contract (uhuahua/SUDO)

`amm_c_addr_uhuahua_SUDO = 'chihuahua1gnpgscdagzl8sfhpv64ex2ahuqg7fkhky6d06krgyt30lfdym78qkae3cz'`

`SUDO_LP_TOKEN_ADDR = 'chihuahua1sslrhe0vthvnykwp2gz89rxx0kuaghstrpm6mtvfj6qszppd5g2qzhjk9z'`

```javascript
let init_msg = JSON.stringify({
   native_denom: {native:'uhuahua'},
   base_denom: {native:'uhuahua'},
   quote_denom: {cw20:'<SUDO_CONTRACT>'},
   lp_token_code_id: 10,
});
```

#### Amm contract uhuahua/PGRM

`amm_c_addr_uhuahua_PGRM = 'chihuahua1vh2p4x96m0qcvhzh3g86dxg9zu8pzwj4xuuwyf2z8dpmshcf0qms5h53kw'`

`PGRM_LP_TOKEN_ADDR = 'chihuahua1ad8qnqvljn2kjgvff9cqgyjqcdrls9cvuyxkhudpwf42ecd9hptshg7gs8'`

```javascript
let init_msg = JSON.stringify({
   native_denom: {"native":"uhuahua"},
   base_denom: {"native":"uhuahua"},
   quote_denom: {"cw20":"<PGRM_CONTRACT>"},
   lp_token_code_id: 10,
});
```

#### Amm contract uhuahua/samoleons

`amm_c_addr_uhuahua_samoleons = 'chihuahua1dup6ratg6cgqp6p5ndnp4v4m033uaj8txke84pwdlx9vd4f9crxsyc8cfe'`

`samoleons_LP_TOKEN_ADDR = 'chihuahua19rrlvk0cj5dpqval73gmql4gj32h5d80tl50hr4jw3nnv07rutjqcvgsul'`

```javascript
let init_msg = JSON.stringify({
   native_denom: {"native":"uhuahua"},
   base_denom: {"native":"uhuahua"},
   quote_denom: {"native":"samoleons"},
   lp_token_code_id: 10,
});
```

#### Amm contract uhuahua/stake

`amm_c_addr_uhuahua_stake = 'chihuahua1qlgsf49atw7qu6mxj7g627m8hxu4ejg5394ccaactd499srp4kzq62667p'`

`stake_LP_TOKEN_ADDR = 'chihuahua15zyps8a8ln75vg7q6rjev2x66xwg290vvq9q983znqpesxvlx74se5gwq9'`

```javascript
let init_msg = JSON.stringify({
   native_denom: {"native":"uhuahua"},
   base_denom: {"native":"uhuahua"},
   quote_denom: {"native":"stake"},
   lp_token_code_id: 10,
});
```

&nbsp;

## Adding Liquidity

We add liquidity to all 4 pools created above `amm_c_addr_uhuahua_SUDO`, `amm_c_addr_uhuahua_PGRM`, `amm_c_addr_uhuahua_samoleons` and `amm_c_addr_uhuahua_stake`.

&nbsp;

### Add liquidity to uhuahua/SUDO

Here we going to be adding 300_000_000uhuahua and 300_000_000uSUDO to `amm_c_addr_uhuahua_SUDO`

#### Increase allowance for amm_c_addr_uhuahua_SUDO on SUDO_CONTRACT

```zsh
$ export E_PAYLOAD='{"increase_allowance":{"spender":"<amm_c_addr_uhuahua_SUDO>", "amount":"300000000"}}'

$ chihuahuad tx wasm execute <SUDO_CONTRACT> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

#### Verify that the allowance was granted to amm_c_addr_uhuahua_SUDO

```zsh
$ export Q_PAYLOAD='{"allowance":{"owner":"<palingram>", "spender":"<amm_c_addr_uhuahua_SUDO>"}}'

$ chihuahuad query wasm contract-state smart <SUDO_CONTRACT> "$Q_PAYLOAD" $NODE --output json
```

#### Add liquidity

```zsh
$ export E_PAYLOAD='{"add_liquidity":{"base_token_amount":"300000000", "max_quote_token_amount":"300000000"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_SUDO> "$E_PAYLOAD" --from palingram --amount=300000000uhuahua $NODE $TXFLAG -y
```

#### Verify the pool reserves for amm_c_addr_uhuahua_SUDO

```zsh
$ export Q_PAYLOAD='{"info":{}}'

$ chihuahuad query wasm contract-state smart <amm_c_addr_uhuahua_SUDO> "$Q_PAYLOAD" $NODE --output json
```

#### Verify native balance

```zsh
$ chihuahuad query bank balances $(chihuahuad keys show -a palingram) $NODE
```

#### Verify <palingram> balance on SUDO_CONTRACT

```zsh
$ export Q_PAYLOAD='{"balance":{"address":"<palingram>"}}'

$ chihuahuad query wasm contract-state smart <SUDO_CONTRACT> "$Q_PAYLOAD" $NODE --output json
```

&nbsp;

### Add liquidity to uhuahua/PGRM

Here we going to be adding 300_000_000uhuahua and 300_000_000uPGRM to `amm_c_addr_uhuahua_PGRM`

#### Increase allowance for amm_c_addr_uhuahua_PGRM on PGRM_CONTRACT

```zsh
$ export E_PAYLOAD='{"increase_allowance":{"spender":"<amm_c_addr_uhuahua_PGRM>", "amount":"300000000"}}'

$ chihuahuad tx wasm execute <PGRM_CONTRACT> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

#### Add liquidity

```zsh
$ export E_PAYLOAD='{"add_liquidity":{"base_token_amount":"300000000", "max_quote_token_amount":"300000000"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_PGRM> "$E_PAYLOAD" --from palingram --amount=300000000uhuahua $NODE $TXFLAG -y
```

&nbsp;

### Add liquidity to uhuahua/samoleons

Here we going to be adding 300_000_000uhuahua and 300_000_000samoleons to `amm_c_addr_uhuahua_samoleons`

#### Add liquidity

```zsh
$ export E_PAYLOAD='{"add_liquidity":{"base_token_amount":"300000000", "max_quote_token_amount":"300000000"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_samoleons> "$E_PAYLOAD" --from palingram --amount=300000000uhuahua,300000000samoleons $NODE $TXFLAG -y
```

&nbsp;

### Add liquidity to uhuahua/stake

Here we going to be adding 300_000_000uhuahua and 300_000_000stake to `amm_c_addr_uhuahua_stake`

#### Add liquidity

```zsh
$ export E_PAYLOAD='{"add_liquidity":{"base_token_amount":"300000000", "max_quote_token_amount":"300000000"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_stake> "$E_PAYLOAD" --from palingram --amount=300000000uhuahua,300000000stake $NODE $TXFLAG -y
```

&nbsp;

## Swapping tokens

We test swapping on `amm_c_addr_uhuahua_SUDO` and `amm_c_addr_uhuahua_stake` contracts

&nbsp;

### Exchange 50000000uhuahua to 42728571uSUDO

```
When exchanging uhuahua for SUDO
B = 300_000_000
Q = 300_000_000
b = 50_000_000

Where q = Qb / (B + b)

q = (300_000_000 * 50_000_000) / (300_000_000 + 50_000_000)
q = 42857142.8571 - 0.3%
q = 42728571
```

#### Do swap

```zsh
$ export E_PAYLOAD='{"swap":{"input_token":"Base","input_amount":"50000000","output_amount":"42728571"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_SUDO> "$E_PAYLOAD" --from palingram --amount=50000000uhuahua $NODE $TXFLAG -y
```

&nbsp;

### Exchange 43_007_207uSUDO for 50_000_000uhuahua

```
When exchanging SUDO for uhuahua
B = 350_000_000
Q = 257_271_429
b = 50_000_000

Where q = Qb / (B - b)

q = (257_271_429 * 50_000_000) / (350_000_000 - 50_000_000)
q = 42878571.5 + 0.3%
q = 43007207
```

#### Increase allowance for amm_c_addr_uhuahua_SUDO on SUDO_CONTRACT

```zsh
$ export E_PAYLOAD='{"increase_allowance":{"spender":"<amm_c_addr_uhuahua_SUDO>", "amount":"43007207"}}'

$ chihuahuad tx wasm execute <SUDO_CONTRACT> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

#### Do swap

```zsh
$ export E_PAYLOAD='{"swap":{"input_token":"Quote","input_amount":"43007207","output_amount":"50000000"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_SUDO> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

&nbsp;

### Exchange 50_000_000uhuahua to 42_728_571stake

```
When exchanging uhuahua for stake
B = 300_000_000
Q = 300_000_000
b = 50_000_000

Where q = Qb / (B + b)

q = (300_000_000 * 50_000_000) / (300_000_000 + 50_000_000)
q = 42857142.8571 - 0.3%
q = 42728571
```

#### Do swap

```zsh
$ export E_PAYLOAD='{"swap":{"input_token":"Base","input_amount":"50000000","output_amount":"42728571"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_stake> "$E_PAYLOAD" --from palingram --amount=50000000uhuahua $NODE $TXFLAG -y
```

&nbsp;

### Exchange 43_007_207stake for 50_000_000uhuahua

```
When exchanging stake for uhuahua
B = 350_000_000
Q = 257_271_429
b = 50_000_000

Where q = Qb / (B - b)

q = (257_271_429 * 50_000_000) / (350_000_000 - 50_000_000)
q = 42878571.5 + 0.3%
q = 43007207
```

#### Do swap

```zsh
$ export E_PAYLOAD='{"swap":{"input_token":"Quote","input_amount":"43007207","output_amount":"50000000"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_stake> "$E_PAYLOAD" --from palingram --amount=43007207stake $NODE $TXFLAG -y
```

&nbsp;

## Pass through swap
We are going to be doing a round trip of passthrough swaps 
from SUDO => PGRM => stake => samoleons => SUDO, fun see how much fees was paid in the process.

&nbsp;

### Swap from SUDO to PGRM

This swap uses two amm contracts `amm_c_addr_uhuahua_SUDO` and `amm_c_addr_uhuahua_PGRM`

```
Given the current state of the the amm_s
When quote_input_amount to amm_c_addr_uhuahua_SUDO = 50_000_000uSUDO, 
Calculate min_quote_output_amount from amm_c_addr_uhuahua_PGRM

To output the intermediate token b from amm_c_addr_uhuahua_SUDO , 
where B = 300_000_000 and Q = 300_278_635 and q = 50_000_000

b = Bq / (Q + q)
b = 300_000_000 * 50_000_000 / (300_278_635 + 50_000_000)
b = 42823051.4259 - fees
b = 42823051.4259 - (0.3% of 42823051.4259)
b = 42_694_582

Now b becomes the base input to amm_c_addr_uhuahua_PGRM
To calculate min_quote_output_amount q, where b = 42_694_582, B = 300_000_000 and Q = 300_000_000
q = Qb / (B + b)
q = 300_000_000 * 42_694_582 / (300_000_000 + 42_694_582)
q = 37375480.3045 - fees
q = 37375480.3045 - (0.3% of 37375480.3045)
q = 37_263_354
```

#### Increase allowance for amm_c_addr_uhuahua_SUDO on SUDO_CONTRACT

```zsh
$ export E_PAYLOAD='{"increase_allowance":{"spender":"<amm_c_addr_uhuahua_SUDO>", "amount":"50000000"}}'

$ chihuahuad tx wasm execute <SUDO_CONTRACT> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

#### Do pass through swap

```zsh
$ export E_PAYLOAD='{"pass_through_swap":{"quote_input_amount":"50000000","output_amm_address":<amm_c_addr_uhuahua_PGRM>,"min_quote_output_amount":"37263354"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_SUDO> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

&nbsp;

### Swap from PGRM to stake

This swap uses two amm contracts `amm_c_addr_uhuahua_PGRM` and `amm_c_addr_uhuahua_stake`

```
From the last swap we did from SUDO to PGRM tokens,
quote_input_amount to amm_c_addr_uhuahua_PGRM now becomes q = 37_263_354
Calculate min_quote_output_amount from amm_c_addr_uhuahua_stake

To output the intermediate token b from amm_c_addr_uhuahua_PGRM, 
where B = 342694582 and Q = 262736646 and q = 37_263_354

b = Bq / (Q + q)
b = 342694582 * 37_263_354 / (262736646 + 37_263_354)
b = 42566498.4098 - fees
b = 42566498.4098 - (0.3% of 42566498.4098)
b = 42_438_798

Now b becomes the base input to amm_c_addr_uhuahua_stake
To calculate min_quote_output_amount q, where b = 42_438_798, B = 300_000_000 and Q = 300_278_635
q = Qb / (B + b)
q = 300_278_635 * 42_438_798 / (300_000_000 + 42_438_798)
q = 37213844.9525 - fees
q = 37213844.9525 - (0.3% of 37213844.9525)
q = 37_102_203
```

#### Increase allowance for amm_c_addr_uhuahua_PGRM on PGRM_CONTRACT

```zsh
$ export E_PAYLOAD='{"increase_allowance":{"spender":"<amm_c_addr_uhuahua_PGRM>", "amount":"37263354"}}'

$ chihuahuad tx wasm execute <PGRM_CONTRACT> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

#### Do pass through swap

```zsh
$ export E_PAYLOAD='{"pass_through_swap":{"quote_input_amount":"37263354","output_amm_address":<amm_c_addr_uhuahua_stake>,"min_quote_output_amount":"37102203"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_PGRM> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

&nbsp;

### Swap from stake to samoleons

This swap uses two amm contracts `amm_c_addr_uhuahua_stake` and `amm_c_addr_uhuahua_samoleons`

```
From the last swap we did from PGRM to STAKE,
quote_input_amount to amm_c_addr_uhuahua_stake now becomes q = 37_102_203, 
Calculate min_quote_output_amount from amm_c_addr_uhuahua_samoleons

To output the intermediate token b from amm_c_addr_uhuahua_stake,
where B = 342438799 and Q = 263176431 and q = 37_102_203

b = Bq / (Q + q)
b = 342438799 * 37_102_203 / (263176431 + 37_102_203)
b = 42311481.3942 - fees
b = 42311481.3942 - (0.3% of 42311481.3942)
b = 42_184_546

Now b becomes the base input to amm_c_addr_uhuahua_samoleons
To calculate min_quote_output_amount q, where b = 42_184_546, B = 300_000_000 and Q = 300_000_000
q = Qb / (B + b)
q = 300_000_000 * 42_184_546 / (300_000_000 + 42_184_546)
q = 36984030.8335 - fees
q = 36984030.8335 - (0.3% of 36984030.8335)
q = 36_873_079
```

#### Do pass through swap

```zsh
$ export E_PAYLOAD='{"pass_through_swap":{"quote_input_amount":"37102203","output_amm_address":<amm_c_addr_uhuahua_samoleons>,"min_quote_output_amount":"36873078"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_stake> "$E_PAYLOAD" --from palingram --amount=37102203stake $NODE $TXFLAG -y
```

&nbsp;

### Swap from samoleons to SUDO

This swap uses two amm contracts `amm_c_addr_uhuahua_samoleons` and `amm_c_addr_uhuahua_SUDO`

```
From the last swap we did from STAKE to samoleons,
quote_input_amount to amm_c_addr_uhuahua_samoleons now becomes q = 36_873_079, 
Calculate min_quote_output_amount from amm_c_addr_uhuahua_SUDO

To output the intermediate token b from amm_c_addr_uhuahua_samoleons,
where B = 342184547 and Q = 263126921 and q = 36_873_079

b = Bq / (Q + q)
b = 342184547 * 36_873_079 / (263126921 + 36_873_079)
b = 42057992.7804 - fees
b = 42057992.7804 - (0.3% of 42057992.7804)
b = 41_931_819

Now b becomes the base input to amm_c_addr_uhuahua_SUDO
To calculate min_quote_output_amount q, where b = 41_931_819, B = 257305418 and Q = 350278635
q = Qb / (B + b)
q = 350278635 * 41_931_819 / (257305418 + 41_931_819)
q = 49084199.7795 - fees
q = 49084199.7795 - (0.3% of 49084199.7795)
q = 48_936_947
```

#### Do pass through swap

```zsh
$ export E_PAYLOAD='{"pass_through_swap":{"quote_input_amount":"36873079","output_amm_address":<amm_c_addr_uhuahua_SUDO>,"min_quote_output_amount":"48936947"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_samoleons> "$E_PAYLOAD" --from palingram --amount=36873079samoleons $NODE $TXFLAG -y
```

&nbsp;

## Remove liquidity

We are going to be removing liquidity from all amm pools `amm_c_addr_uhuahua_PGRM`,  `amm_c_addr_uhuahua_SUDO`, `amm_c_addr_uhuahua_stake` and `amm_c_addr_uhuahua_samoleons`.

&nbsp;

### Remove liquidity from `amm_c_addr_uhuahua_SUDO`

Remove liquidity then verify outputs

#### Increase allowance for amm_c_addr_uhuahua_SUDO on SUDO_LP_TOKEN_ADDR

```zsh
$ export E_PAYLOAD='{"increase_allowance":{"spender":"<amm_c_addr_uhuahua_SUDO>", "amount":"300000000"}}'

$ chihuahuad tx wasm execute <SUDO_LP_TOKEN_ADDR> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
``` 

#### Do remove liquidity

```zsh
$ export E_PAYLOAD='{"remove_liquidity":{"amount":"300000000","min_base_token_output":"299237237","min_quote_token_output":"301341688"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_SUDO> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

&nbsp;

### Remove liquidity from `amm_c_addr_uhuahua_PGRM`

Remove liquidity then verify outputs

#### Increase allowance for amm_c_addr_uhuahua_PGRM on PGRM_LP_TOKEN_ADDR

```zsh
$ export E_PAYLOAD='{"increase_allowance":{"spender":"<amm_c_addr_uhuahua_PGRM>", "amount":"300000000"}}'

$ chihuahuad tx wasm execute <PGRM_LP_TOKEN_ADDR> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
``` 

#### Do remove liquidity

```zsh
$ export E_PAYLOAD='{"remove_liquidity":{"amount":"300000000","min_base_token_output":"300255783","min_quote_token_output":"300000000"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_SUDO> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

&nbsp;

### Remove liquidity from `amm_c_addr_uhuahua_samoleons`

Remove liquidity then verify outputs

#### Increase allowance for amm_c_addr_uhuahua_samoleons on samoleons_LP_TOKEN_ADDR

```zsh
$ export E_PAYLOAD='{"increase_allowance":{"spender":"<amm_c_addr_uhuahua_samoleons>", "amount":"300000000"}}'

$ chihuahuad tx wasm execute <samoleons_LP_TOKEN_ADDR> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
``` 

#### Do remove liquidity

```zsh
$ export E_PAYLOAD='{"remove_liquidity":{"amount":"300000000","min_base_token_output":"300252728","min_quote_token_output":"300000000"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_samoleons> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```

&nbsp;

### Remove liquidity from `amm_c_addr_uhuahua_stake`

Remove liquidity then verify outputs

#### Increase allowance for amm_c_addr_uhuahua_stake on stake_LP_TOKEN_ADDR

```zsh
$ export E_PAYLOAD='{"increase_allowance":{"spender":"<amm_c_addr_uhuahua_stake>", "amount":"300000000"}}'

$ chihuahuad tx wasm execute <stake_LP_TOKEN_ADDR> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
``` 

#### Do remove liquidity

```zsh
$ export E_PAYLOAD='{"remove_liquidity":{"amount":"300000000","min_base_token_output":"300254252","min_quote_token_output":"300278634"}}'

$ chihuahuad tx wasm execute <amm_c_addr_uhuahua_stake> "$E_PAYLOAD" --from palingram $NODE $TXFLAG -y
```