#[cfg(test)]
mod tests {
    use crate::msg::{InfoResponse, InstantiateMsg, QueryMsg};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    const USER: &str = "user";
    const STAKING_DENOM: &str = "udenom";
    const IBC_DENOM_1: &str = "ibc/usdc_denom";
    const SUPPLY: u128 = 500_000_000u128;

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![
                        Coin {
                            denom: STAKING_DENOM.to_string(),
                            amount: Uint128::from(SUPPLY),
                        },
                        Coin {
                            denom: IBC_DENOM_1.to_string(),
                            amount: Uint128::from(SUPPLY),
                        },
                    ],
                )
                .unwrap();
        })
    }

    fn contract_template() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn instantiate_vault(app: &mut App) -> Addr {
        let template_id = app.store_code(contract_template());

        let msg = InstantiateMsg {
            staking_denom: STAKING_DENOM.to_string(),
            owner_address: USER.to_string(),
            account_manager_address: USER.to_string(),
        };

        let template_contract_addr = app
            .instantiate_contract(template_id, Addr::unchecked(USER), &msg, &[], "vault", None)
            .unwrap();

        // return addr
        template_contract_addr
    }

    fn get_vault_info(app: &mut App, contract_address: &Addr) -> InfoResponse {
        let msg = QueryMsg::Info {};
        let result: InfoResponse = app.wrap().query_wasm_smart(contract_address, &msg).unwrap();

        result
    }

    #[test]
    fn test_instantiate() {
        let mut app = mock_app();
        let amm_addr = instantiate_vault(&mut app);

        // Query for the contract info to assert that the lp token and other important
        // data was indeed saved
        let info = get_vault_info(&mut app, &amm_addr);

        assert_eq!(info, InfoResponse {});
    }

    #[test]
    fn test_delegate() {
        // todo
    }
}
