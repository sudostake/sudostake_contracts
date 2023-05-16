#[cfg(test)]
mod tests {
    use crate::{
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VaultCodeListResponse},
        state::{Config, VaultCodeInfo},
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

    fn get_vault_code_id_list(app: &mut App, contract_address: &Addr) -> VaultCodeListResponse {
        let msg = QueryMsg::QueryVaultCodeList {
            start_after: None,
            limit: None,
        };
        app.wrap().query_wasm_smart(contract_address, &msg).unwrap()
    }

    fn bank_balance(router: &mut App, addr: &Addr, denom: String) -> Coin {
        router
            .wrap()
            .query_balance(addr.to_string(), denom)
            .unwrap()
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
        // We need to create a sudomod instance with contract_address = contract1,
        // because this is what is hard coded as INSTANTIATOR_ADDR in the vault contract
        // For testing purposes.
        //
        // That is why we call instantiate_sudomod twice
        // ------------------------------------------------------------------------------
        instantiate_sudomod(app); // contract0
        let sudomod_c_addr = instantiate_sudomod(app); // contract1

        // Return the sudomod contract_addr
        sudomod_c_addr
    }

    #[test]
    fn test_set_vault_code_id() {
        // Step 1
        // Init
        // ------------------------------------------------------------------------------
        let mut app = mock_app();
        let sudomod_c_addr = setup_sudomod(&mut app);

        // Step 2
        // Test error case ContractError::Unauthorized {}
        // by calling with the wrong contract owner
        // -----------------------------------------------------------------------------
        let wrong_owner = "wrong_owner".to_string();
        let code_id = app.store_code(vault_contract_template());
        let execute_msg = ExecuteMsg::SetVaultCodeId { code_id };
        app.execute_contract(
            Addr::unchecked(wrong_owner),
            sudomod_c_addr.clone(),
            &execute_msg,
            &[],
        )
        .unwrap_err();

        // Step 3
        // Set vault code id properly
        // -----------------------------------------------------------------------------
        let code_id = app.store_code(vault_contract_template());
        let execute_msg = ExecuteMsg::SetVaultCodeId { code_id };
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_c_addr.clone(),
            &execute_msg,
            &[],
        )
        .unwrap();

        // Step 4
        // Test error case ContractError::MinVaultCodeUpdateIntervalNotReached {}
        // by calling before the MIN_VAULT_CODE_UPDATE_INTERVAL is reached
        // -----------------------------------------------------------------------------
        let code_id = app.store_code(vault_contract_template());
        let execute_msg = ExecuteMsg::SetVaultCodeId { code_id };
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_c_addr.clone(),
            &execute_msg,
            &[],
        )
        .unwrap_err();

        // Step 5
        // Move the time forward such that MIN_VAULT_CODE_UPDATE_INTERVAL is exceeded
        // -----------------------------------------------------------------------------
        let min_update_interval: u64 = 60 * 60 * 24 * 30;
        app.update_block(|block| block.time = block.time.plus_seconds(min_update_interval));

        // Step 6
        // Set vault code id properly
        // -----------------------------------------------------------------------------
        let code_id = app.store_code(vault_contract_template());
        let execute_msg = ExecuteMsg::SetVaultCodeId { code_id };
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_c_addr.clone(),
            &execute_msg,
            &[],
        )
        .unwrap();

        // Step 7
        // QueryVaultCodeList to ensure data was stored correctly
        // -----------------------------------------------------------------------------
        let vault_code_list = get_vault_code_id_list(&mut app, &sudomod_c_addr);
        assert_eq!(
            vault_code_list,
            VaultCodeListResponse {
                entries: vec![
                    VaultCodeInfo { id: 1, code_id: 4 },
                    VaultCodeInfo { id: 2, code_id: 6 }
                ]
            }
        );
    }

    #[test]
    fn test_mint_vault() {
        // Step 1
        // Init
        // ------------------------------------------------------------------------------
        let mut app = mock_app();
        let _sudomod_c_addr = setup_sudomod(&mut app);

        // todo
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

    #[test]
    fn test_withdraw_balance() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let sudomod_c_addr = setup_sudomod(&mut router);

        // Step 2
        // Send some tokens to sudomod_c_addr
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        router
            .send_tokens(
                Addr::unchecked(USER),
                sudomod_c_addr.clone(),
                &[Coin {
                    denom: STAKING_DENOM.to_string(),
                    amount,
                }],
            )
            .unwrap();

        // Step 3
        // Test error case ContractError::Unauthorized {}
        // ------------------------------------------------------------------------------
        let wrong_owner = Addr::unchecked("WRONG_OWNER");
        let withdraw_balance_msg = ExecuteMsg::WithdrawBalance {
            to_address: None,
            funds: Coin {
                denom: STAKING_DENOM.to_string(),
                amount: amount,
            },
        };
        router
            .execute_contract(
                wrong_owner,
                sudomod_c_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap_err();

        // Step 4
        // Test error case ContractError::InsufficientBalance {}
        // when trying to withdraw more than the available contract balance
        // ------------------------------------------------------------------------------
        let withdraw_balance_msg = ExecuteMsg::WithdrawBalance {
            to_address: None,
            funds: Coin {
                denom: STAKING_DENOM.to_string(),
                amount: amount + amount,
            },
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                sudomod_c_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap_err();

        // Step 5
        // Withdraw half of the contract balance without providing an optional recipient
        // ------------------------------------------------------------------------------
        let half = Uint128::new(500_000);
        let withdraw_balance_msg = ExecuteMsg::WithdrawBalance {
            to_address: None,
            funds: Coin {
                denom: STAKING_DENOM.to_string(),
                amount: half,
            },
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                sudomod_c_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap();

        // Step 6
        // Verify caller's balance
        // ------------------------------------------------------------------------------
        let balance = bank_balance(
            &mut router,
            &Addr::unchecked(USER),
            STAKING_DENOM.to_string(),
        );
        assert_eq!(balance.amount, Uint128::new(SUPPLY) - half);

        // Step 7
        // Withdraw the remaining half of the contract balance
        // by providing an optional recipient
        // ------------------------------------------------------------------------------
        let recipient = Addr::unchecked("recipient");
        let withdraw_balance_msg = ExecuteMsg::WithdrawBalance {
            to_address: Some(recipient.to_string()),
            funds: Coin {
                denom: STAKING_DENOM.to_string(),
                amount: half,
            },
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                sudomod_c_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap();

        // Step 8
        // Verify recipient's balance
        // ------------------------------------------------------------------------------
        let balance = bank_balance(&mut router, &recipient, STAKING_DENOM.to_string());
        assert_eq!(balance.amount, half);

        // Step 9
        // Verify that the contract_addr balance is zero
        // ------------------------------------------------------------------------------
        let balance = bank_balance(&mut router, &sudomod_c_addr, STAKING_DENOM.to_string());
        assert_eq!(balance.amount, Uint128::zero());
    }
}
