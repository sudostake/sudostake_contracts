# SudoMod

SudoMod is central to the SudoStake admin user(s), it also serves as a proxy to minting new vault instances that relay liquidity_comission back to this contract.

&nbsp;

## Upgrade strategy

When a new version of vault contract is available, admin user(s) will call SetVaultCodeId on sudomod_contract_address.
All instances of vault contract created subsequently uses the latest vault_code_id.

This makes the protocol more resilient to forks as vault owners can choose to maintain their old vault instances, or chooose to transfer their assets over from their old vaults, or choose to manage both the new and old vaults simultaneously.
