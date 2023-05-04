#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InstantiateMsg};
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

    fn vault_contract_template() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            vault_contract::contract::execute,
            vault_contract::contract::instantiate,
            vault_contract::contract::query,
        ))
    }

    fn instantiate_sudomod(app: &mut App) -> Addr {
        let template_id = app.store_code(sudomod_contract_template());

        let msg = InstantiateMsg {};

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

    fn setup_sudomod(app: &mut App) -> Addr {
        // Step 1
        // We need to create a sudomod instance with contract_address = contract1,
        // because this is what is hard coded as INSTANTIATOR_ADDR in the vault contract
        // For testing purposes.
        //
        // That is why we call instantiate_sudomod twice
        // ------------------------------------------------------------------------------
        instantiate_sudomod(app); // contract0
        let sudomod_addr = instantiate_sudomod(app); // contract1

        // Step 2
        // Set the sudomod_addr as INSTANTIATOR_ADDR on the vault
        // build, store code and get the vault_code_id
        // ------------------------------------------------------------------------------
        let code_id = app.store_code(vault_contract_template());

        // Step 3
        // Call SetVaultCodeId execute message
        // ------------------------------------------------------------------------------
        let execute_msg = ExecuteMsg::SetVaultCodeId { code_id };
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_addr.clone(),
            &execute_msg,
            &[],
        )
        .unwrap();

        // TODO check info to make sure data was saved

        // Return the sudomod contract_addr
        sudomod_addr
    }

    #[test]
    fn test_create_vault() {
        // Step 1
        // Init
        // ------------------------------------------------------------------------------
        let mut app = mock_app();
        let _sudomod_addr = setup_sudomod(&mut app);
    }
}
