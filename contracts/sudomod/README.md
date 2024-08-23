# SudoMod

SudoMod is central to the SudoStake admin user(s), it also serves as a proxy for minting new vault instances that relay liquidity_comission back to this contract.

&nbsp;

## Upgrade strategy

When a new version of the vault contract is available, the admin user(s) will call SetVaultCodeId on sudomod_contract_address.
All instances of vault contract created subsequently use the latest vault_code_id.

This makes the protocol more resilient to forks as vault owners can choose to maintain their old vault instances, transfer their assets over from their old vaults, or manage both the new and old vaults simultaneously.
