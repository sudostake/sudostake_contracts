#[cfg(test)]
mod tests {
    use crate::{
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
        state::Config,
    };
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

    fn get_contract_info(app: &mut App, contract_address: &Addr) -> Config {
        let msg = QueryMsg::Info {};
        app.wrap().query_wasm_smart(contract_address, &msg).unwrap()
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
        let sudomod_c_addr = instantiate_sudomod(app); // contract1

        // Step 2
        // Set the sudomod_c_addr as INSTANTIATOR_ADDR on the vault
        // build, store code and get the vault_code_id
        // ------------------------------------------------------------------------------
        let code_id = app.store_code(vault_contract_template());

        // Step 3
        // Call SetVaultCodeId execute message
        // ------------------------------------------------------------------------------
        let execute_msg = ExecuteMsg::SetVaultCodeId { code_id };
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_c_addr.clone(),
            &execute_msg,
            &[],
        )
        .unwrap();

        // Step 4
        // Call SetVaultCreationFee execute message
        // ------------------------------------------------------------------------------
        let amount = Coin {
            amount: Uint128::new(10_000_000),
            denom: IBC_DENOM_1.to_string(),
        };
        let execute_msg = ExecuteMsg::SetVaultCreationFee {
            amount: amount.clone(),
        };
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_c_addr.clone(),
            &execute_msg,
            &[],
        )
        .unwrap();

        // Step 5
        // Check info to make sure data was saved correctly
        // ------------------------------------------------------------------------------
        let info = get_contract_info(app, &sudomod_c_addr);
        assert_eq!(
            info,
            Config {
                owner: Addr::unchecked(USER),
                vault_code_id: Some(code_id),
                vault_creation_fee: Some(amount),
            }
        );

        // Return the sudomod contract_addr
        sudomod_c_addr
    }

    #[test]
    fn test_mint_vault() {
        // Step 1
        // Init
        // ------------------------------------------------------------------------------
        let mut app = mock_app();
        let _sudomod_c_addr = setup_sudomod(&mut app);
    }

    #[test]
    fn test_transfer_ownership() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let sudomod_c_addr = setup_sudomod(&mut router);

        // Step 2
        // Test error case  ContractError::Unauthorized {}
        // When transfer_ownership is called by a user who is not the current
        // sudomod owner
        // ------------------------------------------------------------------------------
        let new_owner = "new_owner".to_string();
        let transfer_ownership_msg = ExecuteMsg::TransferOwnership {
            to_address: new_owner.clone(),
        };
        router
            .execute_contract(
                Addr::unchecked("fake_owner"),
                sudomod_c_addr.clone(),
                &transfer_ownership_msg,
                &[],
            )
            .unwrap_err();

        // Step 3
        // set the new vault owner by the current vault owner
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                sudomod_c_addr.clone(),
                &transfer_ownership_msg,
                &[],
            )
            .unwrap();

        // Step 4
        // Query the vault info to verify the new owner
        // ------------------------------------------------------------------------------
        let info = get_contract_info(&mut router, &sudomod_c_addr);
        assert_eq!(info.owner, Addr::unchecked(new_owner));
    }
}
