use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, BankMsg, Binary, BlockInfo, Coin, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw0::parse_reply_instantiate_data;
use cw20::{Cw20ExecuteMsg, Denom, Expiration, MinterResponse};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg, TokenSelect};
use crate::state::{
    SwapPrice, Token, TokenAmount, BASE_TOKEN, LP_TOKEN, NATIVE_DENOM, QUOTE_TOKEN,
};

// Version info for migration info
pub const CONTRACT_NAME: &str = "huahuaswap";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTANTIATE_LP_TOKEN_REPLY_ID: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Store the contract name and version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Check that native denom is of Native token type
    match msg.native_denom {
        Denom::Native(_) => {}
        _ => {
            return Err(ContractError::InvalidNativeDenom {});
        }
    }

    // Make sure the base_denom == native_denom
    if msg.native_denom.clone().ne(&msg.base_denom.clone()) {
        return Err(ContractError::NativeTokenNotProvidedInPair {});
    }

    // Check that the quote denom != base denom
    if msg.base_denom.clone().eq(&msg.quote_denom.clone()) {
        return Err(ContractError::InvalidQuoteDenom {});
    }

    // Save the native denom
    NATIVE_DENOM.save(deps.storage, &msg.native_denom)?;

    // Save base token
    BASE_TOKEN.save(
        deps.storage,
        &Token {
            reserve: Uint128::zero(),
            denom: msg.base_denom.clone(),
        },
    )?;

    // Save quote token
    QUOTE_TOKEN.save(
        deps.storage,
        &Token {
            denom: msg.quote_denom.clone(),
            reserve: Uint128::zero(),
        },
    )?;

    // Add submessage for creating the LP token for this pool
    let sub_msg = SubMsg {
        gas_limit: None,
        id: INSTANTIATE_LP_TOKEN_REPLY_ID,
        reply_on: ReplyOn::Success,
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.lp_token_code_id,
            msg: to_binary(&cw20_base::msg::InstantiateMsg {
                name: "HuahuaSwap LP Token".into(),
                symbol: "hhslpt".into(),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.into(),
                    cap: None,
                }),
                marketing: None,
            })?,
            funds: vec![],
            label: format!("hhslp_{:?}_{:?}", msg.base_denom, msg.quote_denom),
        }
        .into(),
    };

    // Build response
    let res = Response::new()
        .add_attribute("method", "instantiate")
        .add_submessage(sub_msg);

    // return response
    Ok(res)
}

/**
 * Handle reply for contract instantiation
 * Get the contract address and save as LP_TOKEN
 *
 * @return the token_contract_addr as an attribute on success
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        INSTANTIATE_LP_TOKEN_REPLY_ID => handle_instantiate_reply(deps, msg),
        id => Err(StdError::generic_err(format!("Unknown reply id: {}", id))),
    }
}

fn handle_instantiate_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    let res = parse_reply_instantiate_data(msg);
    let data = match res {
        Ok(d) => d,
        Err(_) => {
            return Err(StdError::generic_err("Error parsing data"));
        }
    };

    // Validate contract address
    let cw20_addr = deps.api.addr_validate(&data.contract_address)?;

    // Save lp token
    LP_TOKEN.save(deps.storage, &cw20_addr)?;

    Ok(Response::new().add_attribute("token_contract_addr", data.contract_address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddLiquidity {
            base_token_amount,
            max_quote_token_amount,
            expiration,
        } => execute_add_liquidity(
            deps,
            &info,
            env,
            base_token_amount,
            max_quote_token_amount,
            expiration,
        ),
        ExecuteMsg::RemoveLiquidity {
            amount,
            min_base_token_output,
            min_quote_token_output,
            expiration,
        } => execute_remove_liquidity(
            deps,
            info,
            env,
            amount,
            min_base_token_output,
            min_quote_token_output,
            expiration,
        ),
        ExecuteMsg::Swap {
            input_token,
            input_amount,
            output_amount,
            expiration,
        } => execute_swap(
            env,
            deps,
            &info,
            input_amount,
            input_token,
            output_amount,
            &info.sender,
            expiration,
        ),
        ExecuteMsg::SwapAndSendTo {
            input_token,
            input_amount,
            output_amount,
            recipient,
            expiration,
        } => execute_swap(
            env,
            deps,
            &info,
            input_amount,
            input_token,
            output_amount,
            &recipient,
            expiration,
        ),
        ExecuteMsg::PassThroughSwap {
            output_amm_address,
            quote_input_amount,
            min_quote_output_amount,
            expiration,
        } => execute_pass_through_swap(
            deps,
            info,
            env,
            quote_input_amount,
            output_amm_address,
            min_quote_output_amount,
            expiration,
        ),
    }
}

fn check_expiration(
    expiration: &Option<Expiration>,
    block: &BlockInfo,
) -> Result<(), ContractError> {
    if let Some(e) = expiration {
        if e.is_expired(block) {
            return Err(ContractError::MsgExpirationError {});
        }
    }

    Ok(())
}

fn get_amount_for_denom(coins: &[Coin], denom: &str) -> Coin {
    let amount: Uint128 = coins
        .iter()
        .filter(|c| c.denom == denom)
        .map(|c| c.amount)
        .sum();

    Coin {
        amount,
        denom: denom.to_string(),
    }
}

fn validate_exact_native_amount(
    coins: &[Coin],
    given_amount: Uint128,
    denom_str: &str,
) -> Result<(), ContractError> {
    let actual = get_amount_for_denom(coins, denom_str);

    if actual.amount != given_amount {
        return Err(ContractError::IncorrectAmountProvided {
            provided: actual.amount,
            required: given_amount,
        });
    }

    Ok(())
}

fn get_lp_token_supply(deps: Deps, lp_token_addr: &Addr) -> StdResult<Uint128> {
    let resp: cw20::TokenInfoResponse = deps
        .querier
        .query_wasm_smart(lp_token_addr, &cw20_base::msg::QueryMsg::TokenInfo {})?;
    Ok(resp.total_supply)
}

fn get_token_balance(deps: Deps, contract: &Addr, addr: &Addr) -> StdResult<Uint128> {
    let resp: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        contract,
        &cw20_base::msg::QueryMsg::Balance {
            address: addr.to_string(),
        },
    )?;
    Ok(resp.balance)
}

pub fn get_lp_token_amount_to_mint(
    base_token_amount: Uint128,
    liquidity_supply: Uint128,
    base_reserve: Uint128,
) -> Result<Uint128, ContractError> {
    if liquidity_supply == Uint128::zero() {
        Ok(base_token_amount)
    } else {
        Ok(base_token_amount
            .checked_mul(liquidity_supply)
            .map_err(StdError::overflow)?
            .checked_div(base_reserve)
            .map_err(StdError::divide_by_zero)?)
    }
}

pub fn get_required_quote_token_amount(
    base_token_amount: Uint128,
    quote_token_amount: Uint128,
    liquidity_supply: Uint128,
    quote_reserve: Uint128,
    base_reserve: Uint128,
) -> Result<Uint128, StdError> {
    if liquidity_supply == Uint128::zero() {
        Ok(quote_token_amount)
    } else {
        Ok(base_token_amount
            .checked_mul(quote_reserve)
            .map_err(StdError::overflow)?
            .checked_div(base_reserve)
            .map_err(StdError::divide_by_zero)?)
    }
}

fn get_cw20_transfer_from_msg(
    owner: &Addr,
    recipient: &Addr,
    token_addr: &Addr,
    token_amount: Uint128,
) -> StdResult<CosmosMsg> {
    // create transfer cw20 msg
    let transfer_cw20_msg = Cw20ExecuteMsg::TransferFrom {
        owner: owner.into(),
        recipient: recipient.into(),
        amount: token_amount,
    };
    let exec_cw20_transfer = WasmMsg::Execute {
        contract_addr: token_addr.into(),
        msg: to_binary(&transfer_cw20_msg)?,
        funds: vec![],
    };
    let cw20_transfer_cosmos_msg: CosmosMsg = exec_cw20_transfer.into();
    Ok(cw20_transfer_cosmos_msg)
}

fn mint_lp_tokens(
    recipient: &Addr,
    liquidity_amount: Uint128,
    lp_token_address: &Addr,
) -> StdResult<CosmosMsg> {
    let mint_msg = cw20_base::msg::ExecuteMsg::Mint {
        recipient: recipient.into(),
        amount: liquidity_amount,
    };
    Ok(WasmMsg::Execute {
        contract_addr: lp_token_address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    }
    .into())
}

fn get_bank_transfer_to_msg(recipient: &Addr, denom: &str, native_amount: Uint128) -> CosmosMsg {
    let transfer_bank_msg = BankMsg::Send {
        to_address: recipient.into(),
        amount: vec![Coin {
            denom: denom.into(),
            amount: native_amount,
        }],
    };

    let transfer_bank_cosmos_msg: CosmosMsg = transfer_bank_msg.into();
    transfer_bank_cosmos_msg
}

fn get_cw20_transfer_to_msg(
    recipient: &Addr,
    token_addr: &Addr,
    token_amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(WasmMsg::Execute {
        contract_addr: token_addr.into(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.into(),
            amount: token_amount,
        })?,
        funds: vec![],
    }
    .into())
}

fn get_burn_msg(contract: &Addr, owner: &Addr, amount: Uint128) -> StdResult<CosmosMsg> {
    let msg = cw20_base::msg::ExecuteMsg::BurnFrom {
        owner: owner.to_string(),
        amount,
    };
    Ok(WasmMsg::Execute {
        contract_addr: contract.to_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    }
    .into())
}

pub fn execute_add_liquidity(
    deps: DepsMut,
    info: &MessageInfo,
    env: Env,
    base_token_amount: Uint128,
    max_quote_token_amount: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    check_expiration(&expiration, &env.block)?;

    // Check that non zero amounts are passed for both tokens
    if base_token_amount.is_zero() || max_quote_token_amount.is_zero() {
        return Err(ContractError::NonZeroInputAmountExpected {});
    }

    // load the token reserves
    let base = BASE_TOKEN.load(deps.storage)?;
    let quote = QUOTE_TOKEN.load(deps.storage)?;
    let lp_token_addr = LP_TOKEN.load(deps.storage)?;

    // Validate the input for the base_token_amount to know if the user
    // sent the exact amount and denom in the contract call
    if let Denom::Native(denom) = base.denom {
        validate_exact_native_amount(&info.funds, base_token_amount, &denom)?;
    }

    // If the quote token is Native, validate the input for max_quote_token_amount
    // to know if the user sent the exact amount and denom in the contract call
    if let Denom::Native(denom) = quote.denom.clone() {
        validate_exact_native_amount(&info.funds, max_quote_token_amount, &denom)?;
    }

    // Calculate how much lp tokens to mint
    let lp_token_supply = get_lp_token_supply(deps.as_ref(), &lp_token_addr)?;
    let liquidity_amount =
        get_lp_token_amount_to_mint(base_token_amount, lp_token_supply, base.reserve)?;

    // Calculate the required_quote_token_amount
    let required_quote_token_amount = get_required_quote_token_amount(
        base_token_amount,
        max_quote_token_amount,
        lp_token_supply,
        quote.reserve,
        base.reserve,
    )?;

    // Validate that max_quote_token_amount <= required_quote_token_amount
    if required_quote_token_amount > max_quote_token_amount {
        return Err(ContractError::MaxQuoteTokenAmountExceeded {
            max_quote_token_amount,
            required_quote_token_amount,
        });
    }

    // Generate SDK message for token transfers and LP tokens mint
    let mut sdk_msgs = vec![];

    match quote.denom {
        Denom::Cw20(addr) => {
            sdk_msgs.push(get_cw20_transfer_from_msg(
                &info.sender,
                &env.contract.address,
                &addr,
                required_quote_token_amount,
            )?);
        }

        Denom::Native(denom) => {
            // If the quote token is Native and required_quote_token_amount < max_quote_token_amount
            // we return the difference to info.sender
            if required_quote_token_amount < max_quote_token_amount {
                let change = max_quote_token_amount - required_quote_token_amount;

                sdk_msgs.push(get_bank_transfer_to_msg(&info.sender, &denom, change));
            }
        }
    }

    // Update token reserves
    BASE_TOKEN.update(deps.storage, |mut base| -> Result<_, ContractError> {
        base.reserve += base_token_amount;
        Ok(base)
    })?;
    QUOTE_TOKEN.update(deps.storage, |mut quote| -> Result<_, ContractError> {
        quote.reserve += required_quote_token_amount;
        Ok(quote)
    })?;

    // Mint LP tokens
    sdk_msgs.push(mint_lp_tokens(
        &info.sender,
        liquidity_amount,
        &lp_token_addr,
    )?);

    // respond
    Ok(Response::new().add_messages(sdk_msgs).add_attributes(vec![
        attr("base_token_amount", base_token_amount),
        attr("required_quote_token_amount", required_quote_token_amount),
        attr("liquidity_received", liquidity_amount),
    ]))
}

pub fn execute_remove_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    lp_amount: Uint128,
    min_base_token_output: Uint128,
    min_quote_token_output: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    check_expiration(&expiration, &env.block)?;

    // load the token reserves
    let base = BASE_TOKEN.load(deps.storage)?;
    let quote = QUOTE_TOKEN.load(deps.storage)?;
    let lp_token_addr = LP_TOKEN.load(deps.storage)?;
    let lp_token_supply = get_lp_token_supply(deps.as_ref(), &lp_token_addr)?;
    let user_lp_balance = get_token_balance(deps.as_ref(), &lp_token_addr, &info.sender)?;

    // Check if lp amount to withdraw is valid
    if lp_amount > user_lp_balance {
        return Err(ContractError::InsufficientLiquidityError {
            requested: lp_amount,
            available: user_lp_balance,
        });
    }

    // Calculate the base token amount to withdraw from the pool
    let base_amount_to_output = lp_amount
        .checked_mul(base.reserve)
        .map_err(StdError::overflow)?
        .checked_div(lp_token_supply)
        .map_err(StdError::divide_by_zero)?;

    if base_amount_to_output < min_base_token_output {
        return Err(ContractError::MinBaseTokenOutputError {
            requested: min_base_token_output,
            available: base_amount_to_output,
        });
    }

    // Calculate the quote token amount to withdraw from the pool
    let quote_amount_to_output = lp_amount
        .checked_mul(quote.reserve)
        .map_err(StdError::overflow)?
        .checked_div(lp_token_supply)
        .map_err(StdError::divide_by_zero)?;

    if quote_amount_to_output < min_quote_token_output {
        return Err(ContractError::MinQuoteTokenOutputError {
            requested: min_quote_token_output,
            available: quote_amount_to_output,
        });
    }

    // Generate SDK messages for token transfers and LP tokens burn
    let mut sdk_msgs = vec![];

    // Construct the messages to send the output tokens to info.sender
    match base.denom {
        Denom::Native(denom) => {
            sdk_msgs.push(get_bank_transfer_to_msg(
                &info.sender,
                &denom,
                base_amount_to_output,
            ));
        }
        _ => {
            // This branch is never called because
            // we already enforced only Native as base token
        }
    }

    match quote.denom {
        Denom::Cw20(addr) => {
            sdk_msgs.push(get_cw20_transfer_to_msg(
                &info.sender,
                &addr,
                quote_amount_to_output,
            )?);
        }

        Denom::Native(denom) => {
            // Send the native token to info.sender
            sdk_msgs.push(get_bank_transfer_to_msg(
                &info.sender,
                &denom,
                quote_amount_to_output,
            ));
        }
    }

    // Update token reserves
    BASE_TOKEN.update(deps.storage, |mut base| -> Result<_, ContractError> {
        base.reserve -= base_amount_to_output;
        Ok(base)
    })?;
    QUOTE_TOKEN.update(deps.storage, |mut quote| -> Result<_, ContractError> {
        quote.reserve -= quote_amount_to_output;
        Ok(quote)
    })?;

    // Construct message to burn lp_amount
    sdk_msgs.push(get_burn_msg(&lp_token_addr, &info.sender, lp_amount)?);

    // respond
    Ok(Response::new().add_messages(sdk_msgs).add_attributes(vec![
        attr("liquidity_burned", lp_amount),
        attr("base_token_returned", base_amount_to_output),
        attr("quote_token_returned", quote_amount_to_output),
    ]))
}

/*
 * When swapping from base token to quote token, we use fn exactInputVariableOutput {}
 * Where the input_amount is the exact amount of base tokens to be swapped for a variable amount
 * of the quote tokens such that calculated_quote_output >= output_amount
 * Here, output_amount represents min_quote_output_amount.
 *
 *
 * When swapping from quote token to base token, we use fn exactOutputVariableInput {}
 * Where the input_amount is the max limit of quote token to be inputed in exchange for an exact amount
 * of base token such that calculated_quote_input <= input_amount
 * Here, input_amount represents the max_quote_input_amount.
 *
 *
 * What this means is that the swap_fee is always charged to the quote token.
 */
pub fn execute_swap(
    _env: Env,
    deps: DepsMut,
    info: &MessageInfo,
    input_amount: Uint128,
    input_token: TokenSelect,
    output_amount: Uint128,
    recipient: &Addr,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    check_expiration(&expiration, &_env.block)?;

    // here we load the token reserves
    let base = BASE_TOKEN.load(deps.storage)?;
    let quote = QUOTE_TOKEN.load(deps.storage)?;

    // Here we get the swap_prices which is the amount of input and output tokens required
    let swap_price = match input_token {
        TokenSelect::Base => exact_input_variable_output(
            input_amount,
            output_amount,
            base.reserve,
            quote.reserve,
            base.denom.clone(),
            quote.denom.clone(),
        )?,

        TokenSelect::Quote => exact_output_variable_input(
            output_amount,
            input_amount,
            base.reserve,
            quote.reserve,
            base.denom.clone(),
            quote.denom.clone(),
        )?,
    };

    // Create SDK messages holder
    let mut sdk_msgs = vec![];

    // Update reserves and sdk messages for the input token
    let native_denom = get_native_denom_str(&deps)?;

    match swap_price.input.denom.clone() {
        Denom::Native(input_denom) => {
            match input_denom == native_denom {
                true => {
                    validate_exact_native_amount(
                        &info.funds,
                        swap_price.input.amount,
                        &input_denom,
                    )?;

                    BASE_TOKEN.update(deps.storage, |mut base| -> Result<_, ContractError> {
                        base.reserve += swap_price.input.amount;
                        Ok(base)
                    })?;
                }
                false => {
                    validate_exact_native_amount(&info.funds, input_amount, &input_denom)?;

                    // Return change if input_amount > swap_price.input.amount
                    if input_amount > swap_price.input.amount {
                        let change = input_amount - swap_price.input.amount;

                        sdk_msgs.push(get_bank_transfer_to_msg(&info.sender, &input_denom, change));
                    }

                    QUOTE_TOKEN.update(deps.storage, |mut quote| -> Result<_, ContractError> {
                        quote.reserve += swap_price.input.amount;
                        Ok(quote)
                    })?;
                }
            }
        }

        Denom::Cw20(addr) => {
            sdk_msgs.push(get_cw20_transfer_from_msg(
                &info.sender,
                &_env.contract.address,
                &addr,
                swap_price.input.amount,
            )?);

            QUOTE_TOKEN.update(deps.storage, |mut quote| -> Result<_, ContractError> {
                quote.reserve += swap_price.input.amount;
                Ok(quote)
            })?;
        }
    }

    // Update reserves and sdk messages for the output token
    match swap_price.output.denom.clone() {
        Denom::Native(output_denom) => {
            sdk_msgs.push(get_bank_transfer_to_msg(
                recipient,
                &output_denom,
                swap_price.output.amount,
            ));

            match native_denom == output_denom {
                true => {
                    BASE_TOKEN.update(deps.storage, |mut base| -> Result<_, ContractError> {
                        base.reserve -= swap_price.output.amount;
                        Ok(base)
                    })?;
                }
                false => {
                    QUOTE_TOKEN.update(deps.storage, |mut quote| -> Result<_, ContractError> {
                        quote.reserve -= swap_price.output.amount;
                        Ok(quote)
                    })?;
                }
            }
        }

        Denom::Cw20(addr) => {
            sdk_msgs.push(get_cw20_transfer_to_msg(
                recipient,
                &addr,
                swap_price.output.amount,
            )?);

            QUOTE_TOKEN.update(deps.storage, |mut quote| -> Result<_, ContractError> {
                quote.reserve -= swap_price.output.amount;
                Ok(quote)
            })?;
        }
    }

    // Respond
    Ok(Response::new().add_messages(sdk_msgs).add_attributes(vec![
        attr("input_amount", swap_price.input.amount),
        attr("input_denom", format!("{:?}", swap_price.input.denom)),
        attr("output_amount", swap_price.output.amount),
        attr("output_denom", format!("{:?}", swap_price.output.denom)),
    ]))
}

/**
 * To output q and input b, we use
 * (B + b) * (Q - q) = k, where k = B * Q
 *
 * Differentiate for variable output q
 *
 * (Q - q) = k / (B + b)
 * q = Q - (BQ / (B + b))
 * q * (B + b)  =  Q *  (B + b) - BQ
 * q * (B + b) = QB + Qb - BQ
 * q = Qb / (B + b)
 */
pub fn exact_input_variable_output(
    exact_input_amount: Uint128,
    min_output_amount: Uint128,
    base_reserve: Uint128,
    quote_reserve: Uint128,
    base_denom: Denom,
    quote_denom: Denom,
) -> Result<SwapPrice, ContractError> {
    let numerator = quote_reserve
        .checked_mul(exact_input_amount)
        .map_err(StdError::overflow)?;

    let denominator = base_reserve
        .checked_add(exact_input_amount)
        .map_err(StdError::overflow)?;

    let calculated_quote_output = numerator
        .checked_div(denominator)
        .map_err(StdError::divide_by_zero)?;

    // Deduct swap_fee from the calculated_quote_output
    let swap_fee = get_swap_fee(calculated_quote_output)?;
    let calculated_quote_output = calculated_quote_output
        .checked_sub(swap_fee)
        .map_err(StdError::overflow)?;

    // make sure calculated_quote_output >= min_output_amount
    if calculated_quote_output < min_output_amount {
        return Err(ContractError::SwapMinError {
            min: min_output_amount,
            available: calculated_quote_output,
        });
    }

    Ok(SwapPrice {
        input: TokenAmount {
            amount: exact_input_amount,
            denom: base_denom,
        },
        output: TokenAmount {
            amount: calculated_quote_output,
            denom: quote_denom,
        },
    })
}

// Here we hardcode the swap fees as 3/1000 or 0.3% of amount
fn get_swap_fee(amount: Uint128) -> StdResult<Uint128> {
    amount
        .checked_mul(Uint128::from(3u128))
        .map_err(StdError::overflow)?
        .checked_div(Uint128::from(1000u128))
        .map_err(StdError::divide_by_zero)
}

/**
 * To output b and input q, we use
 * (B - b) * (Q + q) = k, where k = B * Q
 *
 * Differentiate for variable input q
 *
 * (Q + q) = k / (B - b)
 * q = -Q + ( k / (B - b))
 * q * (B - b) = -Q *  (B - b) + BQ
 * q * (B - b) = -QB + Qb + BQ
 * q = Qb / (B - b)
 */
pub fn exact_output_variable_input(
    exact_output_amount: Uint128,
    max_input_amount: Uint128,
    base_reserve: Uint128,
    quote_reserve: Uint128,
    base_denom: Denom,
    quote_denom: Denom,
) -> Result<SwapPrice, ContractError> {
    let numerator = quote_reserve
        .checked_mul(exact_output_amount)
        .map_err(StdError::overflow)?;

    let denominator = base_reserve
        .checked_sub(exact_output_amount)
        .map_err(StdError::overflow)?;

    let calculated_quote_input = numerator
        .checked_div(denominator)
        .map_err(StdError::divide_by_zero)?;

    // Add swap_fee to the calculated_quote_input
    let swap_fee = get_swap_fee(calculated_quote_input)?;
    let calculated_quote_input = calculated_quote_input
        .checked_add(swap_fee)
        .map_err(StdError::overflow)?;

    // make sure calculated_quote_input <= max_input_amount
    if calculated_quote_input > max_input_amount {
        return Err(ContractError::SwapMaxError {
            max: max_input_amount,
            required: calculated_quote_input,
        });
    }

    Ok(SwapPrice {
        input: TokenAmount {
            amount: calculated_quote_input,
            denom: quote_denom,
        },
        output: TokenAmount {
            amount: exact_output_amount,
            denom: base_denom,
        },
    })
}

fn get_native_denom_str(deps: &DepsMut) -> Result<String, ContractError> {
    let native_denom_enum = NATIVE_DENOM.load(deps.storage)?;

    Ok(match native_denom_enum.clone() {
        Denom::Native(native_denom) => native_denom,
        Denom::Cw20(_) => {
            // this will never be called because we already ensured
            // that the native_denom_enum is always native
            return Err(ContractError::NoneError {});
        }
    })
}

pub fn execute_pass_through_swap(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    quote_input_amount: Uint128,
    output_amm_address: Addr,
    min_quote_output_amount: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    check_expiration(&expiration, &_env.block)?;

    // here we load the token reserves
    let base = BASE_TOKEN.load(deps.storage)?;
    let quote = QUOTE_TOKEN.load(deps.storage)?;

    /*
     * To output b and input q, we use
     * (B - b) * (Q + q) = k, where k = B * Q
     *
     * Differentiate for variable output b
     *
     * (Q + q) = k / (B - b)
     * (Q + q) = B * Q / (B - b)
     * (B - b) =  B * Q / (Q + q)
     * b = B - (B * Q) / (Q + q)
     *
     * Multiplying both sides by  (Q + q)
     * b (Q + q) = B (Q + q) - (B * Q)
     * b (Q + q) = BQ + Bq - BQ
     *
     * b = Bq / (Q + q)
     *
     * Note: because we are outputing a variable amount of base token from this first swap,
     * The swap fees is deducted from the output
     */
    let numerator = base
        .reserve
        .checked_mul(quote_input_amount)
        .map_err(StdError::overflow)?;

    let denominator = quote
        .reserve
        .checked_add(quote_input_amount)
        .map_err(StdError::overflow)?;

    let calculated_base_output = numerator
        .checked_div(denominator)
        .map_err(StdError::divide_by_zero)?;

    // Deduct swap_fee from the calculated_base_output
    let swap_fee = get_swap_fee(calculated_base_output)?;
    let calculated_base_output = calculated_base_output
        .checked_sub(swap_fee)
        .map_err(StdError::overflow)?;

    // Update reserves
    BASE_TOKEN.update(deps.storage, |mut base| -> Result<_, ContractError> {
        base.reserve -= calculated_base_output;
        Ok(base)
    })?;

    QUOTE_TOKEN.update(deps.storage, |mut quote| -> Result<_, ContractError> {
        quote.reserve += quote_input_amount;
        Ok(quote)
    })?;

    // Create SDK messages holder
    let mut sdk_msgs = vec![];

    match quote.denom.clone() {
        // Add message to transfer quote_input_amount to this amm address
        Denom::Cw20(addr) => {
            sdk_msgs.push(get_cw20_transfer_from_msg(
                &info.sender,
                &_env.contract.address,
                &addr,
                quote_input_amount,
            )?);
        }

        // verify that the correct quote tokens was sent in the contract call
        Denom::Native(denom) => {
            validate_exact_native_amount(&info.funds, quote_input_amount, &denom)?;
        }
    }

    // Add the message to do a SwapAndSendTo from the output_amm_address
    // where output goes to info.sender
    sdk_msgs.push(
        WasmMsg::Execute {
            contract_addr: output_amm_address.into(),
            msg: to_binary(&ExecuteMsg::SwapAndSendTo {
                input_token: TokenSelect::Base,
                input_amount: calculated_base_output,
                output_amount: min_quote_output_amount,
                recipient: info.sender,
                expiration,
            })?,
            funds: vec![Coin {
                denom: get_native_denom_str(&deps)?,
                amount: calculated_base_output,
            }],
        }
        .into(),
    );

    Ok(Response::new().add_messages(sdk_msgs).add_attributes(vec![
        attr("input_token_amount", quote_input_amount),
        attr("native_transferred", calculated_base_output),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
    }
}

pub fn query_info(deps: Deps) -> StdResult<InfoResponse> {
    let base = BASE_TOKEN.load(deps.storage)?;
    let quote = QUOTE_TOKEN.load(deps.storage)?;
    let lp_token_address = LP_TOKEN.load(deps.storage)?;

    Ok(InfoResponse {
        base_reserve: base.reserve,
        base_denom: base.denom,
        quote_reserve: quote.reserve,
        quote_denom: quote.denom,
        lp_token_supply: get_lp_token_supply(deps, &lp_token_address)?,
        lp_token_address: lp_token_address,
    })
}
