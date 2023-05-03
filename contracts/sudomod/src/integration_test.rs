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

    fn sudomod_contract_template() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    // TODO
    fn _vault_contract_template() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            vault_contract::contract::execute,
            vault_contract::contract::instantiate,
            vault_contract::contract::query,
        ))
    }

    fn instantiate_sudomod(app: &mut App) -> Addr {
        let template_id = app.store_code(sudomod_contract_template());

        let msg = InstantiateMsg {
            staking_denom: STAKING_DENOM.to_string(),
            owner_address: USER.to_string(),
        };

        let template_contract_addr = app
            .instantiate_contract(
                template_id,
                Addr::unchecked(USER),
                &msg,
                &[],
                "sudomod",
                None,
            )
            .unwrap();

        // return addr
        template_contract_addr
    }

    fn get_sudomod_info(app: &mut App, contract_address: &Addr) -> InfoResponse {
        let msg = QueryMsg::Info {};
        let result: InfoResponse = app.wrap().query_wasm_smart(contract_address, &msg).unwrap();

        result
    }

    #[test]
    fn test_instantiate() {
        let mut app = mock_app();
        let sudomod_addr = instantiate_sudomod(&mut app);

        // Query for the contract info to assert that the lp token and other important
        // data was indeed saved
        let info = get_sudomod_info(&mut app, &sudomod_addr);

        assert_eq!(info, InfoResponse {});
    }
}
