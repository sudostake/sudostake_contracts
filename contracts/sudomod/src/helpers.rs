use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, StdResult, Uint128};

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

pub fn get_amount_for_denom(funds: &[Coin], denom_str: String) -> StdResult<Uint128> {
    Ok(funds
        .iter()
        .filter(|c| c.denom == denom_str)
        .map(|c| c.amount)
        .sum())
}
