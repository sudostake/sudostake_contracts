use crate::state::{ActiveOption, Config, LiquidityRequestMsg};
use cosmwasm_std::{Coin, Delegation, Uint128, VoteOption};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Assigned as the owner of the vault instance
    pub owner_address: String,

    /// from_code_id allows us to easily tell the code_id this vault
    /// was instantiated from.
    /// This is useful when we want to check if the vault is outdated
    /// by comparing from_code_id to the latest vault_code_id on sudomod contract
    pub from_code_id: u64,

    // This is the index number of the current vault
    pub index_number: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Allows the vault owner to stake their tokens to a validator.
    Delegate {
        validator: String,
        amount: Uint128,
    },

    /// Allows the vault owner to redelegate their staked tokens to another validator.
    Redelegate {
        src_validator: String,
        dst_validator: String,
        amount: Uint128,
    },

    /// Allows the vault owner to un-stake their tokens from a validator.
    Undelegate {
        validator: String,
        amount: Uint128,
    },

    /// Allows the vault owner to open a liquidity request option
    RequestLiquidity {
        option: LiquidityRequestMsg,
    },

    /// Allows the vault owner to close a liquidity request
    /// before the offer is accepted by lenders.
    ClosePendingLiquidityRequest {},

    /// Allows a lender to accept the pending liquidity request.
    AcceptLiquidityRequest {},

    // Allows the vault owner/lender to claim delegator rewards
    ClaimDelegatorRewards {},

    /// Allows the vault owner to repay the amount borrowed from the lender
    /// before a liquidation event is trigged by the lender
    RepayLoan {},

    /// Allows the vault owner/lender to liquidate collateral
    /// which may include unstaking the outstanding amount owed to the lender.
    /// after all free balance is spent.
    LiquidateCollateral {},

    /// Allows vault owner/lender to cast a simple vote
    Vote {
        proposal_id: u64,
        vote: VoteOption,
    },

    /// Allows owner_address to transfer ownership to another owner's address
    /// Note: To burn this contract account, set to_address = env.contract.address
    TransferOwnership {
        to_address: String,
    },

    /// Allows the vault owner to withdraw funds from the vault.
    /// While liquidation is processing, the lender's withdrawal
    /// is prioritized over the vault's owner.
    WithdrawBalance {
        to_address: Option<String>,
        funds: Coin,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns InfoResponse
    Info {},

    /// Returns StakingInfoResponse
    StakingInfo {},

    /// Returns an array of all active delegations made from this vault
    AllDelegations {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub config: Config,
    pub liquidity_request: Option<ActiveOption>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingInfoResponse {
    pub total_staked: Uint128,
    pub accumulated_rewards: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllDelegationsResponse {
    pub data: Vec<Delegation>,
}
