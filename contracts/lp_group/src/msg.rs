use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub native_denom: Denom,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Allows users to join a liquidity providers group.
    JoinGroup {},

    /// Allows group members to subscribe to a LRO funding pool
    /// by contributing a portion of the requested liquidity,
    /// once the requested amount is filled, the LRO is automatically
    /// subscribed to on behalf of the group members that contributed
    /// to the  LRO funding pool.
    SubscribeToLROPool {},

    /// Allows group members to unsubscribe from a LRO, by withrawing
    /// their contribution from a LRO funding pool before the LRO is accepted.
    UnsubscribeFromLROPool {},

    /// Allows any member of an active LRO funding pool, to trigger the underlying vault,
    /// to carry out actions such as claim_rewards, begin_liquidation, finalize_contract
    ProcessLROPool {},

    /// Allows the LP_GROUP to listen to events emitted by the underlying vaults
    /// after process_LRO_claims is called on an active vault
    /// funded by the group members.
    ///
    /// Events emitted from vault:
    // [claim_rewards, begin_liquidation, finalized_claim]
    ProcessLROPoolHook {},

    /// Allows group members who are subscribed to a LRO pool to claim their
    /// share of the returns from the pool account after finalized_claim
    /// event is emitted by the underlying vault.
    ClaimRewardsFromLROPool {},

    /// Allows a user to leave liquidity providers group.
    LeaveGroup {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {}
