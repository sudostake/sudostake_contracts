#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg};
    use cosmwasm_std::{testing::mock_env, Addr, Coin, Decimal, Empty, Uint128, Validator};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor, StakingInfo};

    const USER: &str = "user";
    const STAKING_DENOM: &str = "TOKEN";
    const IBC_DENOM_1: &str = "ibc/usdc_denom";
    const SUPPLY: u128 = 500_000_000u128;
    const VALIDATOR_ADDRESS: &str = "validator";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, api, storage| {
            let env = mock_env();

            // Set the initial balances
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
                        address: VALIDATOR_ADDRESS.to_string(),
                        commission: Decimal::zero(),
                        max_commission: Decimal::one(),
                        max_change_rate: Decimal::one(),
                    },
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
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // set balance for wrong_owner so we can try to call the delegate method
        // on a vault owned by USER
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        let wrong_owner = Addr::unchecked("WRONG_OWNER");
        router
            .send_tokens(
                Addr::unchecked(USER),
                wrong_owner.clone(),
                &[Coin {
                    denom: STAKING_DENOM.to_string(),
                    amount,
                }],
            )
            .unwrap();

        // Step 3
        // Test error case ContractError::Unauthorized {}
        // ------------------------------------------------------------------------------
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ADDRESS.to_string(),
            amount,
        };
        router
            .execute_contract(
                wrong_owner,
                vault_c_addr.clone(),
                &delegate_msg,
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount,
                }],
            )
            .unwrap_err();

        // Step 4
        // Test error case ContractError::IncorrectCoinInfoProvided {}
        // ------------------------------------------------------------------------------
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ADDRESS.to_string(),
            amount,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount: Uint128::new(1_000),
                }],
            )
            .unwrap_err();

        // Step 5
        // Call delegate method by the vault owner
        // ------------------------------------------------------------------------------
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ADDRESS.to_string(),
            amount,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount,
                }],
            )
            .unwrap();

        // Step 6
        // query the vault info to assert that the correct amount was delegated
        // ------------------------------------------------------------------------------
        let delegation = router
            .wrap()
            .query_delegation(vault_c_addr, VALIDATOR_ADDRESS.to_string())
            .unwrap()
            .unwrap();

        assert_eq!(
            delegation.amount,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount
            }
        );
    }
}
