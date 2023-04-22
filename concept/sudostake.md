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

* Fixed-term rentals
* Fixed-interest rentals
* Fixed-term Loans


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

