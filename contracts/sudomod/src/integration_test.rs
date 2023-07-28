#[cfg(test)]
mod tests {
    use crate::{
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VaultCodeListResponse},
        state::MIN_VAULT_CODE_UPDATE_INTERVAL,
        state::{Config, VaultCodeInfo},
    };
    use cosmwasm_std::{testing::mock_env, Addr, Coin, Decimal, Empty, Uint128, Validator};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor, StakingInfo};

    const USER: &str = "user";
    const STAKING_DENOM: &str = "udenom";
    const IBC_DENOM_1: &str = "ibc/usdc_denom";
    const SUPPLY: u128 = 500_000_000u128;
    const VALIDATOR_ONE_ADDRESS: &str = "validator_one";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, api, storage| {
            let env = mock_env();

            // Set the initial balances for USER
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

            // Setup staking module for the correct mock data.
            router
                .staking
                .setup(
                    storage,
                    StakingInfo {
                        bonded_denom: STAKING_DENOM.to_string(),
                        unbonding_time: 1, // in seconds
                        apr: Decimal::percent(10),
                    },
                )
                .unwrap();

            // Add mock validator
            router
                .staking
                .add_validator(
                    api,
                    storage,
                    &env.block,
                    Validator {
                        address: VALIDATOR_ONE_ADDRESS.to_string(),
                        commission: Decimal::zero(),
                        max_commission: Decimal::one(),
                        max_change_rate: Decimal::one(),
                    },
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
        let code_id = app.store_code(sudomod_contract_template());
        let msg = InstantiateMsg {};

        let contract_addr = app
            .instantiate_contract(code_id, Addr::unchecked(USER), &msg, &[], "sudomod", None)
            .unwrap();

        // return addr
        contract_addr
    }

    fn setup_sudomod(app: &mut App) -> Addr {
        // Create an instance of sudomod with contract_address = contract1,
        // because this is what is hard coded as INSTANTIATOR_ADDR in the vault contract
        // For testing purposes.
        // ------------------------------------------------------------------------------
        instantiate_sudomod(app); // contract0 not used
        let sudomod_c_addr = instantiate_sudomod(app);

        // Return the contract_addr = contract1
        sudomod_c_addr
    }

    #[test]
    fn test_set_vault_code_id() {
        // Step 1
        // Create an instance of sudomod
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
        // Move time forward by MIN_VAULT_CODE_UPDATE_INTERVAL
        // -----------------------------------------------------------------------------
        app.update_block(|block| {
            block.time = block.time.plus_seconds(MIN_VAULT_CODE_UPDATE_INTERVAL)
        });

        // Step 6
        // Call ExecuteMsg::SetVaultCodeId
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
    fn test_set_vault_creation_fee() {
        // Step 1
        // Create an instance of sudomod
        // ------------------------------------------------------------------------------
        let mut app = mock_app();
        let sudomod_c_addr = setup_sudomod(&mut app);

        // Step 2
        // Test error case ContractError::Unauthorized {}
        // by calling with the wrong contract owner
        // -----------------------------------------------------------------------------
        let vault_creation_fee = Coin {
            amount: Uint128::new(10_000_000),
            denom: IBC_DENOM_1.to_string(),
        };
        let wrong_owner = "wrong_owner".to_string();
        let execute_msg = ExecuteMsg::SetVaultCreationFee {
            amount: vault_creation_fee.clone(),
        };
        app.execute_contract(
            Addr::unchecked(wrong_owner),
            sudomod_c_addr.clone(),
            &execute_msg,
            &[],
        )
        .unwrap_err();

        // Step 3
        // Set vault creation fee properly
        // -----------------------------------------------------------------------------
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_c_addr.clone(),
            &execute_msg,
            &[],
        )
        .unwrap();

        // Step 4
        // Query sudomod info to verify the new vault_creation_fee is set
        // ------------------------------------------------------------------------------
        let info = get_contract_info(&mut app, &sudomod_c_addr);
        assert_eq!(info.vault_creation_fee, Some(vault_creation_fee));
    }

    #[test]
    fn test_mint_vault() {
        // Step 1
        // Create an instance of sudomod
        // ------------------------------------------------------------------------------
        let mut app = mock_app();
        let sudomod_c_addr = setup_sudomod(&mut app);

        // Step 2
        // Test error case ContractError::VaultCodeIdNotSet {}
        // by trying to call MintVault before setting vault code id
        // ------------------------------------------------------------------------------
        let mint_vault_msg = ExecuteMsg::MintVault {};
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_c_addr.clone(),
            &mint_vault_msg,
            &[],
        )
        .unwrap_err();

        // Step 3
        // set vault code id
        // ------------------------------------------------------------------------------
        let code_id = app.store_code(vault_contract_template());
        let set_vault_code_id_msg = ExecuteMsg::SetVaultCodeId { code_id };
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_c_addr.clone(),
            &set_vault_code_id_msg,
            &[],
        )
        .unwrap();

        // Step 4
        // Mint a free vault after setting vault code id
        // ------------------------------------------------------------------------------
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_c_addr.clone(),
            &mint_vault_msg,
            &[],
        )
        .unwrap();

        // Step 5
        // Set vault creation fee
        // ------------------------------------------------------------------------------
        let vault_creation_fee = Coin {
            amount: Uint128::new(10_000_000),
            denom: IBC_DENOM_1.to_string(),
        };
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_c_addr.clone(),
            &ExecuteMsg::SetVaultCreationFee {
                amount: vault_creation_fee.clone(),
            },
            &[],
        )
        .unwrap();

        // Step 6
        // Test error case ContractError::IncorrectTokenCreationFee {}
        // by calling MintVault with an incorrect vault_creation_fee
        // ------------------------------------------------------------------------------
        app.execute_contract(
            Addr::unchecked(USER),
            sudomod_c_addr.clone(),
            &mint_vault_msg,
            &[],
        )
        .unwrap_err();

        // Step 7
        // Call MintVault with the correct vault_creation_fee
        // ------------------------------------------------------------------------------
        let res = app
            .execute_contract(
                Addr::unchecked(USER),
                sudomod_c_addr.clone(),
                &mint_vault_msg,
                &[vault_creation_fee.clone()],
            )
            .unwrap();

        // Step 8
        // Get vault_contract_addr from res
        // ------------------------------------------------------------------------------
        let vault_contract_addr = res.events[3].attributes[0].value.clone();

        // Add e2e test to verify that fees accrue at _sudomod_c_addr
        // for accepted liquidity requests on vault_contract_addr
        // ------------------------------------------------------------------------------
        // ------------------------------------------------------------------------------
        //
        // Step 1
        // Delegate to VALIDATOR_ONE_ADDRESS on vault_contract_addr
        // ------------------------------------------------------------------------------
        let delegate_amount = Uint128::new(1_000_000);
        let delegate_msg = vault_contract::msg::ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount: delegate_amount,
        };
        app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(vault_contract_addr.clone()),
            &delegate_msg,
            &[Coin {
                denom: STAKING_DENOM.into(),
                amount: delegate_amount,
            }],
        )
        .unwrap();

        // Step 2
        // Create a valid FixedTermLoan liquidity request on vault_contract_addr
        // ------------------------------------------------------------------------------
        let duration_in_seconds = 60u64;
        let requested_amount = Uint128::new(100_000_000);
        let expected_liquidity_comission = Uint128::new(300_000);
        let option = vault_contract::state::LiquidityRequestMsg::FixedTermLoan {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: requested_amount,
            },
            interest_amount: Uint128::zero(),
            collateral_amount: delegate_amount,
            duration_in_seconds,
        };
        app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(vault_contract_addr.clone()),
            &vault_contract::msg::ExecuteMsg::RequestLiquidity {
                option: option.clone(),
            },
            &[],
        )
        .unwrap();

        // Step 3
        // Accept the open liquidity request on vault_contract_addr
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg =
            vault_contract::msg::ExecuteMsg::AcceptLiquidityRequest {};
        app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(vault_contract_addr),
            &accept_liquidity_request_msg,
            &[Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: requested_amount,
            }],
        )
        .unwrap();

        // Step 4
        // Verify that liquidity_comission from vault_contract_addr accrue at sudomod_c_addr
        // ------------------------------------------------------------------------------
        // ibc_denom_balance = SUPPLY - requested_amount +  vault_creation_fee
        let balance = bank_balance(&mut app, &sudomod_c_addr, IBC_DENOM_1.to_string());
        assert_eq!(
            balance.amount,
            vault_creation_fee.amount + expected_liquidity_comission
        );
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
        let contract_balance = Uint128::new(1_000_000);
        router
            .send_tokens(
                Addr::unchecked(USER),
                sudomod_c_addr.clone(),
                &[Coin {
                    denom: STAKING_DENOM.to_string(),
                    amount: contract_balance,
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
                amount: contract_balance,
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
                amount: contract_balance + Uint128::new(1),
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
