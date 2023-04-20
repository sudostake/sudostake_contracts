use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub staking_denom: String,
    pub owner_address: String,
    pub max_group_members: u16,
    pub membership_fee: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Allows users who pay membership_fee to be added to the group.
    JoinGroup { user_address: String },

    /// Allows group members to subscribe to a LRO funding pool
    /// by contributing a portion of the requested liquidity,
    /// once the requested amount is filled, the LRO is automatically
    /// subscribed to on behalf of the group members that contributed
    /// to the  LRO funding pool.
    ///
    /// If the msg.sender is the first to subscribe to the group, he has to
    /// contribute at least 10% of the requested liquidity
    SubscribeToPool { vault_id: u16, amount: Coin },

    /// Allows group members to unsubscribe from a LRO, by withrawing
    /// their contribution from a LRO funding pool before the LRO is accepted.
    UnsubscribeFromPool { vault_id: u16 },

    /// Allows group members who are subscribed to a LRO pool to claim their
    /// share of the returns from the pool account after finalized_claim
    /// event is emitted by the underlying vault.
    ClaimRewardsFromPool { vault_id: u16 },

    /// Allows a user to leave liquidity providers group.
    LeaveGroup {},

    /// Allows the group admin to remove a group member, when they are
    /// currently not part of any LRO_pool
    RemoveGroupMember { user_address: String },

    /// Allows the group admin to transfer group ownership to another owner.
    Transfer { to_address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns information about the current state of the vault
    Info {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {}
