use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

// contract info
pub const CONTRACT_NAME: &str = "vault_contract";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Store the contract name and version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // make sure the staking_denom is a non-empty string
    if msg.staking_denom.is_empty() {
        return Err(ContractError::InvalidStakingDenom {});
    }

    // Validate the owner_address
    let address = deps.api.addr_validate(&msg.owner_address)?;

    // Save contract state
    CONFIG.save(
        deps.storage,
        &Config {
            staking_denom: msg.staking_denom,
            owner: address,
        },
    )?;

    // todo test shared types
    shared_types::ProcessPoolHook {
        vault_id: 8u16,
        event: shared_types::VaultEvents::BeginLiquidation,
    };

    // response
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // return response
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
    }
}

pub fn query_info(_deps: Deps) -> StdResult<InfoResponse> {
    Ok(InfoResponse {})
}
