use crate::{
    state::{ActiveOption, LiquidityRequestMsg, LiquidityRequestState, OPEN_LIQUIDITY_REQUEST},
    ContractError,
};
use cosmwasm_std::{
    Addr, BankMsg, Coin, CosmosMsg, Delegation, Deps, DepsMut, DistributionMsg, Env, StakingMsg,
    StdError, StdResult, Uint128,
};

pub fn ensure_validator_is_active(deps: &DepsMut, validator: &str) -> Result<(), ContractError> {
    if deps.querier.query_validator(validator)?.is_none() {
        return Err(ContractError::ValidatorIsInactive {
            validator: validator.to_string(),
        });
    }

    Ok(())
}

pub fn ensure_lender_can_redelegate(deps: &DepsMut, src_validator: &str) -> Result<(), ContractError> {
    if deps.querier.query_validator(src_validator)?.is_some() {
        return Err(ContractError::LenderCannotRedelegateFromActiveValidator {
            validator: src_validator.to_string(),
        });
    }

    Ok(())
}

fn get_amount_for_denom(funds: &[Coin], denom_str: String) -> StdResult<Uint128> {
    Ok(funds
        .iter()
        .filter(|c| c.denom == denom_str)
        .map(|c| c.amount)
        .sum())
}

pub fn get_balace_for_demon(
    deps: &DepsMut,
    env: &Env,
    denom_str: String,
) -> Result<Coin, ContractError> {
    let contract_balances = deps
        .querier
        .query_all_balances(env.contract.address.clone())?;
    let amount = get_amount_for_denom(&contract_balances, denom_str.clone())?;

    Ok(Coin {
        amount,
        denom: denom_str,
    })
}

pub fn validate_exact_input_amount(
    coins: &[Coin],
    given_amount: Uint128,
    denom_str: String,
) -> Result<(), ContractError> {
    let actual_amount = get_amount_for_denom(coins, denom_str)?;
    if actual_amount != given_amount {
        return Err(ContractError::InvalidInputAmount {
            required: given_amount,
            received: actual_amount,
        });
    }

    Ok(())
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

pub fn accumulated_rewards(
    deps: &DepsMut,
    env: &Env,
    selected_validators: Option<Vec<String>>,
) -> StdResult<(Uint128, Vec<DistributionMsg>)> {
    let mut distribute_msgs = vec![];
    let mut total_rewards_claimed = Uint128::new(0);

    // Calculate total_rewards_claimed and build distribute_msgs
    deps.querier
        .query_all_delegations(env.contract.address.clone())?
        .iter()
        .filter(|d| {
            if let Some(validators) = &selected_validators {
                return validators.contains(&d.validator);
            }

            true
        })
        .for_each(|d| {
            distribute_msgs.push(DistributionMsg::WithdrawDelegatorReward {
                validator: d.validator.clone(),
            });

            // Update total_rewards_claimed
            deps.querier
                .query_delegation(env.contract.address.clone(), d.validator.clone())
                .unwrap()
                .unwrap()
                .accumulated_rewards
                .iter()
                .for_each(|c| total_rewards_claimed += c.amount);
        });

    Ok((total_rewards_claimed, distribute_msgs))
}

pub fn query_staking_info(deps: &Deps, env: &Env) -> StdResult<(Uint128, Uint128)> {
    let mut total_staked = Uint128::new(0);
    let mut accumulated_rewards = Uint128::new(0);

    // Calculate total_staked and accumulated_rewards
    deps.querier
        .query_all_delegations(env.contract.address.clone())?
        .iter()
        .for_each(|d| {
            total_staked += d.amount.amount;
            deps.querier
                .query_delegation(env.contract.address.clone(), d.validator.clone())
                .unwrap()
                .unwrap()
                .accumulated_rewards
                .iter()
                .for_each(|c| accumulated_rewards += c.amount);
        });

    Ok((total_staked, accumulated_rewards))
}

pub fn query_all_delegations(deps: &Deps, env: &Env) -> StdResult<Vec<Delegation>> {
    let mut data: Vec<Delegation> = vec![];
    deps.querier
        .query_all_delegations(env.contract.address.clone())?
        .iter()
        .for_each(|d| {
            data.push(d.clone());
        });

    Ok(data)
}

pub fn validate_amount_to_delegate(
    env: &Env,
    deps: &DepsMut,
    amount_to_delegate: Uint128,
    denom_str: String,
) -> Result<(), ContractError> {
    let balance = get_balace_for_demon(deps, env, denom_str.clone())?;
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
    BankMsg::Send {
        to_address: recipient.into(),
        amount: vec![Coin {
            denom: denom.into(),
            amount,
        }],
    }
    .into()
}

pub fn has_open_liquidity_request(deps: &DepsMut) -> StdResult<bool> {
    Ok(OPEN_LIQUIDITY_REQUEST.load(deps.storage)?.is_some())
}

pub fn get_liquidity_comission(amount: Uint128) -> StdResult<Uint128> {
    // Here we hardcode the liquidity_comission as 3/1000 or 0.3% of amount
    amount
        .checked_mul(Uint128::from(3u128))
        .map_err(StdError::overflow)?
        .checked_div(Uint128::from(1000u128))
        .map_err(StdError::divide_by_zero)
}

pub fn process_lender_claims(
    deps: DepsMut,
    env: &Env,
    liquidity_request_state: LiquidityRequestState,
    lender: Addr,
    total_rewards_claimed: Uint128,
) -> Result<Option<CosmosMsg>, ContractError> {
    let denom_str = deps.querier.query_bonded_denom()?;
    match liquidity_request_state {
        LiquidityRequestState::FixedInterestRental {
            requested_amount,
            can_cast_vote,
            claimable_tokens,
            already_claimed,
        } => {
            // Calculate the portion of total_rewards_claimed to send to lender
            let outstanding_amount = claimable_tokens - already_claimed;
            let amount_to_send_to_lender = if outstanding_amount > total_rewards_claimed {
                total_rewards_claimed
            } else {
                outstanding_amount
            };

            // Update the liquidity request state
            let updated_already_claimed = already_claimed + amount_to_send_to_lender;
            OPEN_LIQUIDITY_REQUEST.update(deps.storage, |data| -> Result<_, ContractError> {
                if updated_already_claimed.lt(&claimable_tokens) {
                    let mut option = data.unwrap();
                    option.state = Some(LiquidityRequestState::FixedInterestRental {
                        requested_amount,
                        claimable_tokens,
                        already_claimed: updated_already_claimed,
                        can_cast_vote,
                    });

                    Ok(Some(option))
                } else {
                    Ok(None)
                }
            })?;

            // Return cosmos_msg to transfer funds to the lender
            Ok(Some(get_bank_transfer_to_msg(
                &lender,
                &denom_str,
                amount_to_send_to_lender,
            )))
        }

        LiquidityRequestState::FixedTermRental {
            requested_amount,
            can_cast_vote,
            start_time,
            last_claim_time,
            end_time,
        } => {
            // Calculate the portion of total_rewards_claimed to send to lender
            let current_time = env.block.time;
            let amount_to_send_to_lender = if current_time.le(&end_time) {
                total_rewards_claimed
            } else {
                let duration_eligible_for_rewards = end_time.seconds() - last_claim_time.seconds();
                let duration_since_last_claim = current_time.seconds() - last_claim_time.seconds();

                // calculate the portion of the total_rewards_claimed to go to lender
                total_rewards_claimed
                    .checked_multiply_ratio(
                        duration_eligible_for_rewards,
                        duration_since_last_claim,
                    )
                    .map_err(|_| StdError::GenericErr {
                        msg: "error calculating amount_to_send_to_lender".to_string(),
                    })?
            };

            // Update the liquidity request state
            OPEN_LIQUIDITY_REQUEST.update(deps.storage, |data| -> Result<_, ContractError> {
                if current_time < end_time {
                    let mut option = data.unwrap();
                    option.state = Some(LiquidityRequestState::FixedTermRental {
                        requested_amount,
                        can_cast_vote,
                        start_time,
                        last_claim_time: current_time,
                        end_time,
                    });

                    Ok(Some(option))
                } else {
                    Ok(None)
                }
            })?;

            // Return cosmos_msg to transfer funds to the lender
            Ok(Some(get_bank_transfer_to_msg(
                &lender,
                &denom_str,
                amount_to_send_to_lender,
            )))
        }

        // FixedTermLoan does not currently allow sharing delegator rewards with lender
        _default => Ok(None),
    }
}

pub fn unbond_tokens_from_validators(
    deps: &DepsMut,
    env: &Env,
    max_amount: Uint128,
) -> Result<Vec<StakingMsg>, ContractError> {
    let mut accumulated_unbonding_amount = Uint128::zero();
    let mut unstaking_msgs = vec![];

    // Query all delegations, and unbond enough amount to payback max_amount if available
    deps.querier
        .query_all_delegations(env.contract.address.clone())?
        .iter()
        .for_each(|d| {
            if accumulated_unbonding_amount < max_amount {
                let outstanding_amount = max_amount - accumulated_unbonding_amount;
                let amount_to_unstake = if outstanding_amount < d.amount.amount {
                    outstanding_amount
                } else {
                    d.amount.amount
                };

                // add unbonding msg
                unstaking_msgs.push(StakingMsg::Undelegate {
                    validator: d.validator.clone(),
                    amount: Coin {
                        denom: d.amount.denom.clone(),
                        amount: amount_to_unstake,
                    },
                });

                // update amount unbonded so far
                accumulated_unbonding_amount += amount_to_unstake;
            }
        });

    // Respond
    Ok(unstaking_msgs)
}

pub fn current_lender_can_cast_vote(deps: &DepsMut, env: &Env) -> Result<bool, ContractError> {
    let mut lender_can_cast_vote = false;

    if let Some(ActiveOption {
        msg: _,
        lender: _,
        state: Some(liquidity_request_state),
    }) = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?
    {
        match liquidity_request_state {
            LiquidityRequestState::FixedInterestRental {
                requested_amount: _,
                can_cast_vote,
                claimable_tokens: _,
                already_claimed: _,
            } => {
                if can_cast_vote {
                    lender_can_cast_vote = can_cast_vote;
                }
            }

            LiquidityRequestState::FixedTermRental {
                requested_amount: _,
                can_cast_vote,
                start_time: _,
                last_claim_time: _,
                end_time,
            } => {
                if can_cast_vote && end_time < env.block.time {
                    lender_can_cast_vote = can_cast_vote;
                }
            }

            // FixedTermLoan does not currently allow sharing voting rights with lender
            _default => {}
        }
    }

    Ok(lender_can_cast_vote)
}

pub fn outstanding_fixed_term_loan_debt(
    deps: &DepsMut,
    env: &Env,
) -> Result<Uint128, ContractError> {
    let mut outstanding_debt = Uint128::zero();

    if let Some(ActiveOption {
        msg: _,
        lender: _,
        state:
            Some(LiquidityRequestState::FixedTermLoan {
                requested_amount: _,
                interest_amount: _,
                collateral_amount,
                start_time: _,
                end_time,
                already_claimed,
                last_liquidation_date: _,
                processing_liquidation: _,
            }),
    }) = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?
    {
        if env.block.time >= end_time {
            outstanding_debt = collateral_amount - already_claimed;
        }
    }

    Ok(outstanding_debt)
}

pub fn can_delegate_with_active_liquidity_request(
    deps: &DepsMut,
    env: &Env,
) -> Result<(), ContractError> {
    if let Some(ActiveOption {
        msg: _,
        lender: _,
        state:
            Some(LiquidityRequestState::FixedTermLoan {
                requested_amount: _,
                interest_amount: _,
                collateral_amount,
                start_time: _,
                end_time,
                already_claimed,
                last_liquidation_date: _,
                processing_liquidation: _,
            }),
    }) = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?
    {
        if env.block.time >= end_time {
            return Err(ContractError::ClearOutstandingDebt {
                amount: Coin {
                    amount: collateral_amount - already_claimed,
                    denom: deps.querier.query_bonded_denom()?,
                },
            });
        }
    }

    Ok(())
}

pub fn map_liquidity_request_state(
    deps: &DepsMut,
    env: &Env,
) -> Result<(LiquidityRequestState, Coin), ContractError> {
    Ok(
        match OPEN_LIQUIDITY_REQUEST.load(deps.storage)?.unwrap().msg {
            LiquidityRequestMsg::FixedInterestRental {
                requested_amount,
                claimable_tokens,
                can_cast_vote,
            } => (
                LiquidityRequestState::FixedInterestRental {
                    requested_amount: requested_amount.clone(),
                    claimable_tokens,
                    already_claimed: Uint128::zero(),
                    can_cast_vote,
                },
                requested_amount,
            ),

            LiquidityRequestMsg::FixedTermRental {
                requested_amount,
                duration_in_seconds,
                can_cast_vote,
            } => (
                LiquidityRequestState::FixedTermRental {
                    requested_amount: requested_amount.clone(),
                    start_time: env.block.time,
                    end_time: env.block.time.plus_seconds(duration_in_seconds),
                    last_claim_time: env.block.time,
                    can_cast_vote,
                },
                requested_amount,
            ),

            LiquidityRequestMsg::FixedTermLoan {
                requested_amount,
                interest_amount,
                duration_in_seconds,
                collateral_amount,
            } => (
                LiquidityRequestState::FixedTermLoan {
                    requested_amount: requested_amount.clone(),
                    interest_amount,
                    collateral_amount,
                    start_time: env.block.time,
                    end_time: env.block.time.plus_seconds(duration_in_seconds),
                    last_liquidation_date: None,
                    already_claimed: Uint128::zero(),
                    processing_liquidation: false,
                },
                requested_amount,
            ),
        },
    )
}
