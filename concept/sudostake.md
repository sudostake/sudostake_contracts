# Motivation

Blockstream has a mining derivatives marketplace where users can buy the rights to hashing power to mine Bitcoins over a certain period defined in the contract.

When a user buys a [BMN(Blockstream Mining Note)](https://blockstream.com/finance/bmn/), Bitcoin hashing power (measured in TH/s) gets allocated to the user, and mining rewards streams to an escrow account that releases the funds to the BMN holder after the expiration date defined in the contract.

Meanwhile, BMN can still be traded in a secondary market for users who need liquidity before the expiration date stated on the mining contract.

&nbsp;

### Benefits to miners

* BMN allows miners to generate liquidity in exchange for hashing power they control, thereby bringing liquidity to the otherwise illiquid but cashflow-rich mining business.
* BMN lowers the barrier to entry into the mining business by allowing investors to buy BMN representing mining hash rate.

&nbsp;

## Managing staked assets directly on-chain

Cosmos-SDK base blockchains usually have an un-bonding period(or time it takes for bonded tokens to become available after a withdrawal request to a validator) ranging from 7 - 28 days, which leads to a couple of issues for delegators.

* Limited Defi use cases (because bonding directly to the network means the only viable yield-bearing option is staking rewards).
* Currently, it's not possible to transfer bonded assets to another address without first un-bonding them.

&nbsp;

## Vault-based alternative to liquid-staking

Vaults are instances of a smart-contract that manages staked assets on behalf of its owner, the benefits from doing this are as follows;

* Vaults can be transfered instantly to another owner/entity
* Vaults containing assets can be used as collateral to borrow USDC in a p2p marketplace
* A vault owner can offer staking deals in exchange for upfront liquidity
* Users can create multiple vaults with different presets for managing different asset groups.
* The flexibility vault-based asset management provides, encourages more assets to be bonded to the network, which increases the over-all security of the network.

&nbsp;

## Contracts specification

The protocol defines two primary smart contracts for managing these interactions.

&nbsp;

### VAULTS_MANAGER_CONTRACT

<details>
<summary>State: vaults_list</summary>

<pre>
[
    {
        vault_id,
        vault_c_addres,
        owner_adddress
    },
    ...
]
</pre>
</details>

#### `fn: mint`

Create a new vault by calling the instantiate method of the VAULT_CONTRACT, which returns a contract address, that is then associated with the `msg.sender`.

#### `fn: transfer`

Allow a vault owner to transfer ownership to another user.

#### `fn: list_for_sale`

A vault owner can list their vault for sale for a fixed price.

&nbsp;

### VAULT_CONTRACT

<details>
<summary>State: vault_preferences</summary>

<pre>
{
    // Only vaults_manager_contract can update the owner's address
    vault_manager_address,
    owner:{
        address,
        actions_scope: [delegate, undelegate, redelegate, claim_rewards, withdraw_funds]
        is_active,
    }
    controller: {
        address,
        actions_scope: [claim_rewards, withdraw_funds],
        expiration_date
        percentage_stake_to_undelegate_at_liquidation
        is_active
    }
}
</pre>
</details>

#### `fn: delegate`

Allow the vault owner to stake the assets to a validator.

#### `fn: undelegate`

Allow the vault owner to un-stake the assets.

#### `fn: redelegate`

Allow the vault owner to redelegate their stake to another validator.

#### `fn: withdraw_funds`

Allow the vault owner/controller to withdraw assets held in the vault based on allowance.

#### `fn: claim_rewards`

Allow the vault owner/controller to claim staking rewards.

#### `fn: change_vault_owner`

Allow the associated vaults_manager_contract to change a vault's owner.

#### `fn: create_deal`

Allow the vault owner to create a deal that other users can see

#### `fn: accept_deal`

Allow a user to accept a deal created by the vault owner

&nbsp;

## Governance

TBA

&nbsp;

## Market Places

Anyone can create frontends/marketplaces that allow users to interact with vaults on supported blockchains.
