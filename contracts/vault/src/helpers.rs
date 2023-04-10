use crate::{
    state::{ActiveOption, LiquidityRequestOptionState, OPEN_LIQUIDITY_REQUEST},
    ContractError,
};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, StdError, StdResult, Uint128, WasmMsg,
};

pub fn verify_validator_is_active(deps: &DepsMut, validator: &str) -> Result<(), ContractError> {
    let res = deps.querier.query_validator(validator)?;
    if res.is_none() {
        return Err(ContractError::ValidatorIsInactive {
            validator: validator.to_string(),
        });
    }

    Ok(())
}

pub fn get_available_staking_balace(
    env: &Env,
    deps: &DepsMut,
    denom_str: String,
) -> Result<Coin, ContractError> {
    // find the coin with non-zero balance that matches the denom
    let contract_balances = deps
        .querier
        .query_all_balances(env.contract.address.clone())?;

    let coin = contract_balances
        .iter()
        .find(|coin| coin.denom == denom_str);

    Ok(match coin {
        Some(coin) => coin.clone(),
        None => Coin {
            amount: Uint128::zero(),
            denom: denom_str,
        },
    })
}

pub fn query_total_delegations(deps: &DepsMut, env: &Env) -> StdResult<Uint128> {
    let total = deps
        .querier
        .query_all_delegations(env.contract.address.clone())?
        .iter()
        .map(|d| d.amount.amount)
        .sum();

    Ok(total)
}

pub fn validate_amount_to_delegate(
    env: &Env,
    deps: &DepsMut,
    amount_to_delegate: Uint128,
    denom_str: String,
) -> Result<(), ContractError> {
    let balance = get_available_staking_balace(env, deps, denom_str.clone())?;
    if balance.amount < amount_to_delegate {
        return Err(ContractError::InsufficientBalance {
            available: balance,
            required: Coin {
                denom: denom_str,
                amount: amount_to_delegate,
            },
        });
    }

    Ok(())
}

pub fn get_bank_transfer_to_msg(recipient: &Addr, denom: &str, amount: Uint128) -> CosmosMsg {
    let transfer_bank_msg = BankMsg::Send {
        to_address: recipient.into(),
        amount: vec![Coin {
            denom: denom.into(),
            amount,
        }],
    };

    let transfer_bank_cosmos_msg: CosmosMsg = transfer_bank_msg.into();
    transfer_bank_cosmos_msg
}

pub fn has_open_liquidity_request(deps: &DepsMut) -> StdResult<bool> {
    let data = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?;
    Ok(data.is_some())
}

pub fn get_amount_for_denom(funds: &[Coin], denom_str: String) -> StdResult<Uint128> {
    let amount: Uint128 = funds
        .iter()
        .filter(|c| c.denom == denom_str)
        .map(|c| c.amount)
        .sum();

    Ok(amount)
}

pub fn process_lender_claims(
    deps: DepsMut,
    env: &Env,
    liquidity_request: ActiveOption,
    total_rewards_claimed: Uint128,
) -> Result<Option<CosmosMsg>, ContractError> {
    // Get native denom str
    let denom_str = deps.querier.query_bonded_denom()?;

    // Process liquidity_request variants
    match liquidity_request.state.unwrap() {
        LiquidityRequestOptionState::FixedInterestRental {
            requested_amount,
            can_cast_vote,
            claimable_tokens,
            already_claimed,
            is_lp_group,
        } => {
            // Calculate amount_to_send_to_lender from total_rewards_claimed
            let outstanding_amount = claimable_tokens - already_claimed;
            let amount_to_send_to_lender = if outstanding_amount > total_rewards_claimed {
                total_rewards_claimed
            } else {
                outstanding_amount
            };
            let updated_already_claimed = already_claimed + amount_to_send_to_lender;

            // Update the liquidity request state
            OPEN_LIQUIDITY_REQUEST.update(deps.storage, |data| -> Result<_, ContractError> {
                if updated_already_claimed.eq(&claimable_tokens) {
                    Ok(None)
                } else {
                    let mut option = data.unwrap();
                    option.state = Some(LiquidityRequestOptionState::FixedInterestRental {
                        requested_amount,
                        claimable_tokens,
                        already_claimed: updated_already_claimed,
                        can_cast_vote,
                        is_lp_group,
                    });

                    Ok(Some(option))
                }
            })?;

            // Return cosmos_msg to transfer funds to the lender
            return Ok(Some(if is_lp_group.is_some() {
                WasmMsg::Execute {
                    contract_addr: liquidity_request.lender.unwrap().to_string(),
                    msg: to_binary(&shared_types::ProcessPoolHook {
                        vault_address: env.contract.address.to_string(),
                        event: if updated_already_claimed.eq(&claimable_tokens) {
                            shared_types::VaultEvents::FinalizedClaim {}
                        } else {
                            shared_types::VaultEvents::ClaimedRewards {}
                        },
                    })?,
                    funds: vec![Coin {
                        denom: denom_str,
                        amount: amount_to_send_to_lender,
                    }],
                }
                .into()
            } else {
                get_bank_transfer_to_msg(
                    &liquidity_request.lender.unwrap(),
                    &denom_str,
                    amount_to_send_to_lender,
                )
            }));
        }

        LiquidityRequestOptionState::FixedTermRental {
            requested_amount,
            can_cast_vote,
            start_time,
            last_claim_time,
            end_time,
            is_lp_group,
        } => {
            let current_time = env.block.time;
            let amount_to_send_to_lender = if current_time < end_time {
                total_rewards_claimed
            } else {
                let duration_eligible_for_rewards = end_time.seconds() - last_claim_time.seconds();
                let total_duration_since_last_claim =
                    current_time.seconds() - last_claim_time.seconds();

                // calculate the portion of the total_rewards_claimed to go to lender
                total_rewards_claimed
                    .checked_multiply_ratio(
                        duration_eligible_for_rewards,
                        total_duration_since_last_claim,
                    )
                    .map_err(|_| StdError::GenericErr {
                        msg: "error calculating amount_to_send_to_lender".to_string(),
                    })?
            };

            // Update the liquidity request state
            OPEN_LIQUIDITY_REQUEST.update(deps.storage, |data| -> Result<_, ContractError> {
                if current_time < end_time {
                    let mut option = data.unwrap();
                    option.state = Some(LiquidityRequestOptionState::FixedTermRental {
                        requested_amount,
                        can_cast_vote,
                        start_time,
                        last_claim_time: current_time,
                        end_time,
                        is_lp_group,
                    });

                    Ok(Some(option))
                } else {
                    Ok(None)
                }
            })?;

            // Return cosmos_msg to transfer funds to the lender
            return Ok(Some(if is_lp_group.is_some() {
                WasmMsg::Execute {
                    contract_addr: liquidity_request.lender.unwrap().to_string(),
                    msg: to_binary(&shared_types::ProcessPoolHook {
                        vault_address: env.contract.address.to_string(),
                        event: if current_time < end_time {
                            shared_types::VaultEvents::ClaimedRewards {}
                        } else {
                            shared_types::VaultEvents::FinalizedClaim {}
                        },
                    })?,
                    funds: vec![Coin {
                        denom: denom_str,
                        amount: amount_to_send_to_lender,
                    }],
                }
                .into()
            } else {
                get_bank_transfer_to_msg(
                    &liquidity_request.lender.unwrap(),
                    &denom_str,
                    amount_to_send_to_lender,
                )
            }));
        }

        _default => Ok(None),
    }
}
