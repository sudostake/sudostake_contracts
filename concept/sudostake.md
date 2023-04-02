# Bonding tokens directly on-chain

*The staking token is the primary capital asset on any POS blockchain. We hope that creating more options for what users can do with their staked tokens will increase the economic activities facilitated by the blockchain.*

Chains created with the Cosmos-SDK usually have an un-bonding period or time for bonded tokens to become available after a withdrawal request from the network, which can range from 7 - 28 days, leading to a couple of issues for delegators.

1. Bonding tokens directly to the network, means the only yield-bearing option available to delegators comes from protocol inflation or fees generated from network activities.

2. It is not possible to transfer bonded tokens to another address without first un-bonding them.

&nbsp;

## SudoStake Liquidity Vaults

Vaults are instances of a smart-contract that manages staked tokens on behalf of its owner, the benefits that come from doing this are as follows;

* Instant transfer of vault ownership to another owner/entity.
* Vault owners can choose to open liquidity request options (more on this in a bit).
* Payment streaming, where vault owner can add a beneficiary who can claim rewards
* Ability to create multiple vaults for managing different asset groups.
* The flexibility of vault-based asset management encourages more assets to be bonded to the network, which increases the overall network security.

&nbsp;

## Liquidity Request Options (LROs)

Once staked assets can be managed through a vault, it creates more opportunities for using this same asset already bonded to the network in other Defi use cases such as;

* Fixed-term rewards claims
* Fixed-interest rewards claims
* Fixed-term Loans
* Hybrid option(Combines FixedTermRewardsClaim && FixedTermLoan)

&nbsp;

## Contracts specification

The protocol defines 3 primary smart contracts for managing these interactions.

&nbsp;

### ACCOUNTS_MANAGER_CONTRACT

The accounts manager contract allow users to create and keep track of all instances of the
VAULT_CONTRACT and LP_GROUP_CONTRACT

---

#### `fn: create_vault`

Creates a new vault by calling the instantiate method of the VAULT_CONTRACT, which returns a contract address, that is then associated with the `msg.sender`.

#### `fn: create_lp_group`

Creates a new liquidity providers group (LP_GROUP) by calling the instantiate method of the LP_GROUP_CONTRACT, which returns a contract address, that is then associated with the `msg.sender`.

&nbsp;

### VAULT_CONTRACT

A vault is a smart contract account that allow users to manage their staked assets, as well as giving them the options to use same assets in Defi.

---

#### `fn: delegate`

Allows the vault owner to stake the assets to a validator.

#### `fn: undelegate`

Allows the vault owner to un-stake the assets from a validator.

#### `fn: redelegate`

Allows the vault owner to redelegate their stake to another validator.

#### `fn: open_LRO`

Allows the vault owner to open a liquidity request option

#### `fn: close_LRO`

Allows the vault owner to close a liquidity request option before the offer is accepted by other market participants.

#### `fn: accept_LRO`

Allows a liquidity provider (which could be an individual or an LP_GROUP) to accept a liquidity request option.

#### `fn: process_LRO_claims`

Allows the vault owner/controller to process LRO claims.

#### `fn: claim_delegator_rewards`

Allows the vault owner to claim delegator rewards when there is no active LRO

#### `fn: withdraw_funds`

Allows the vault owner/controller to withdraw assets held in the vault based on allowance.

#### `fn: transfer`

Allows a vault owner to transfer ownership to another user.

&nbsp;

### LP_GROUP_CONTRACT

A liquidity providers group (or LP_GROUP for short), allows liquidity providers to pool their resources together to fund LROs, where interest generated from accepted LROs is split proportionally amongst invested members of the group upon maturity.

---

#### `fn: join_group`

Allows users to join a liquidity providers group.

#### `fn: subscribe_to_LRO_pool`

Allows group members to subscribe to a LRO funding pool by contributing a portion of the requested liquidity, once the requested amount is filled, the LRO is automatically subscribed to on behalf of the group members that contributed to the  LRO funding pool.

#### `fn: unsubscribe_from_LRO_pool`

Allows group members to unsubscribe from a LRO, by withrawing their contribution from a LRO funding pool before the LRO is accepted.

#### `fn: process_LRO_pool`

Allows any member of an active LRO funding pool, to trigger the underlying vault, to carry out actions such as claim_rewards, begin_liquidation, finalize_contract

#### `fn: process_LRO_pool_hook`

Allows the LP_GROUP to listen to events emitted by the underlying vaults after process_LRO_claims is called on an active vault funded by the group members.

Events emitted:
[claim_rewards, begin_liquidation, finalized_claim]

#### `fn: claim_rewards_from_LRO_pool`

Allows group members who are subscribed to a LRO pool to claim their share of the returns from the pool account after finalized_claim event is emitted by the underlying vault.

#### `fn: leave_group`

Allows a user to leave a liquidity providers group.

&nbsp;

## Protocol fees
0.3% fee charged to vault owners on requested liquidity.

&nbsp;

## Governance

TBA

&nbsp;

## Frontend implementation

* Dashboard for managing vaults
* Dashboard for managing LP_GROUPS
* Marketplace for trading LROs

