#[cfg(test)]
mod tests {
    use crate::{
        msg::{ExecuteMsg, InfoResponse, InstantiateMsg, LiquidityRequestOptionMsg, QueryMsg},
        state::{ActiveOption, Config, LiquidityRequestOptionState},
    };
    use cosmwasm_std::{testing::mock_env, Addr, Coin, Decimal, Empty, Uint128, Validator};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor, StakingInfo};

    const USER: &str = "user";
    const STAKING_DENOM: &str = "TOKEN";
    const IBC_DENOM_1: &str = "ibc/usdc_denom";
    const SUPPLY: u128 = 500_000_000u128;
    const VALIDATOR_ONE_ADDRESS: &str = "validator_one";
    const VALIDATOR_TWO_ADDRESS: &str = "validator_two";

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
                        address: VALIDATOR_ONE_ADDRESS.to_string(),
                        commission: Decimal::zero(),
                        max_commission: Decimal::one(),
                        max_change_rate: Decimal::one(),
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
                        address: VALIDATOR_TWO_ADDRESS.to_string(),
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

    fn bank_balance(router: &mut App, addr: &Addr, denom: String) -> Coin {
        router
            .wrap()
            .query_balance(addr.to_string(), denom)
            .unwrap()
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
        let vault_c_addr = instantiate_vault(&mut app);

        // Query for the contract info to assert
        // that all important data was indeed saved
        let info = get_vault_info(&mut app, &vault_c_addr);

        assert_eq!(
            info.config,
            Config {
                owner: Addr::unchecked(USER),
                acc_manager: Addr::unchecked(USER),
            }
        );
    }

    #[test]
    fn test_delegate() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // Send some tokens to vault_c_addr
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        router
            .send_tokens(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
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
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount,
        };
        router
            .execute_contract(wrong_owner, vault_c_addr.clone(), &delegate_msg, &[])
            .unwrap_err();

        // Step 4
        // Test error case ContractError::ValidatorIsInactive {}
        // ------------------------------------------------------------------------------
        let inactive_validator = String::from("validator_inactive");
        let delegate_msg = ExecuteMsg::Delegate {
            validator: inactive_validator,
            amount,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[],
            )
            .unwrap_err();

        // Step 5
        // Test error case ContractError::InsufficientBalance {}
        // ------------------------------------------------------------------------------
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount: amount + Uint128::new(1_000_000),
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[],
            )
            .unwrap_err();

        // Step 6
        // Call delegate method by the vault owner
        // ------------------------------------------------------------------------------
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[],
            )
            .unwrap();

        // Step 7
        // query the vault delegations to assert that the correct amount was delegated
        // ------------------------------------------------------------------------------
        let delegation = router
            .wrap()
            .query_delegation(vault_c_addr.clone(), VALIDATOR_ONE_ADDRESS.to_string())
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

    #[test]
    fn test_redelegate() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // delegate some tokens to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
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

        // Step 3
        // Test error case ContractError::Unauthorized {}
        // ------------------------------------------------------------------------------
        let wrong_owner = Addr::unchecked("WRONG_OWNER");
        let redelegate_msg = ExecuteMsg::Redelegate {
            src_validator: VALIDATOR_ONE_ADDRESS.to_string(),
            dst_validator: VALIDATOR_TWO_ADDRESS.to_string(),
            amount,
        };
        router
            .execute_contract(wrong_owner, vault_c_addr.clone(), &redelegate_msg, &[])
            .unwrap_err();

        // Step 4
        // Test error case ContractError::ValidatorIsInactive {}
        // ------------------------------------------------------------------------------
        let inactive_dst_validator = String::from("validator_inactive");
        let redelegate_msg = ExecuteMsg::Redelegate {
            src_validator: VALIDATOR_ONE_ADDRESS.to_string(),
            dst_validator: inactive_dst_validator,
            amount,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &redelegate_msg,
                &[],
            )
            .unwrap_err();

        // Step 5
        // redelegate tokens to VALIDATOR_TWO_ADDRESS
        // ------------------------------------------------------------------------------
        let redelegate_msg = ExecuteMsg::Redelegate {
            src_validator: VALIDATOR_ONE_ADDRESS.to_string(),
            dst_validator: VALIDATOR_TWO_ADDRESS.to_string(),
            amount,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &redelegate_msg,
                &[],
            )
            .unwrap();

        // Step 6
        // verify that VALIDATOR_TWO_ADDRESS now has the delegations of user
        // ------------------------------------------------------------------------------
        let delegation = router
            .wrap()
            .query_delegation(vault_c_addr, VALIDATOR_TWO_ADDRESS.to_string())
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

    #[test]
    fn test_undelegate() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // Send some tokens to vault_c_addr
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        router
            .send_tokens(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &[Coin {
                    denom: STAKING_DENOM.to_string(),
                    amount,
                }],
            )
            .unwrap();

        // Step 3
        // Call delegate method by the vault owner
        // ------------------------------------------------------------------------------
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[],
            )
            .unwrap();

        // Step 4
        // Test error case ContractError::Unauthorized {}
        // ------------------------------------------------------------------------------
        let wrong_owner = Addr::unchecked("WRONG_OWNER");
        let undelegate_msg = ExecuteMsg::Undelegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount,
        };
        router
            .execute_contract(wrong_owner, vault_c_addr.clone(), &undelegate_msg, &[])
            .unwrap_err();

        // Step 5
        // Test error case ContractError::MaxUndelegateAmountExceeded {}
        // ------------------------------------------------------------------------------
        let undelegate_msg = ExecuteMsg::Undelegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount: amount + Uint128::new(1_000_000),
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &undelegate_msg,
                &[],
            )
            .unwrap_err();

        // Step 6
        // Call undelegate method by the vault owner
        // ------------------------------------------------------------------------------
        let undelegate_msg = ExecuteMsg::Undelegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &undelegate_msg,
                &[],
            )
            .unwrap();

        // Step 7
        // Foward the blockchain ahead and process unbonding queue
        // ------------------------------------------------------------------------------
        router.update_block(|block| block.time = block.time.plus_seconds(10));
        router
            .sudo(cw_multi_test::SudoMsg::Staking(
                cw_multi_test::StakingSudo::ProcessQueue {},
            ))
            .unwrap();

        // Step 8
        // Verify that the contract now has the amount unstaked as balance
        // ------------------------------------------------------------------------------
        let vault_balance = router
            .wrap()
            .query_balance(vault_c_addr.clone(), STAKING_DENOM)
            .unwrap();

        assert_eq!(
            vault_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount
            }
        );
    }

    #[test]
    fn test_open_liquidity_request_fixed_term_loan() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
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

        // Step 3
        // Test error case ContractError::Unauthorized {}
        // due to wrong owner
        // ------------------------------------------------------------------------------
        let wrong_owner = Addr::unchecked("WRONG_OWNER");
        let open_liquidity_request_msg = ExecuteMsg::OpenLiquidityRequest {
            option: crate::msg::LiquidityRequestOptionMsg::FixedTermLoan {
                requested_amount: Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount,
                },
                collateral_amount: amount,
                duration_in_seconds: 60u64,
                can_claim_rewards: None,
                can_cast_vote: None,
            },
        };
        router
            .execute_contract(
                wrong_owner,
                vault_c_addr.clone(),
                &open_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 4
        // Test error case ContractError::InvalidLiquidityRequestOption {}
        // ------------------------------------------------------------------------------
        let open_liquidity_request_msg = ExecuteMsg::OpenLiquidityRequest {
            option: crate::msg::LiquidityRequestOptionMsg::FixedTermLoan {
                requested_amount: Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount: Uint128::zero(),
                },
                collateral_amount: Uint128::zero(),
                duration_in_seconds: 0u64,
                can_claim_rewards: None,
                can_cast_vote: None,
            },
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &open_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 5
        // Create a valid liquidity request
        // ------------------------------------------------------------------------------
        let duration_in_seconds = 60u64;
        let option = LiquidityRequestOptionMsg::FixedTermLoan {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            collateral_amount: amount,
            duration_in_seconds,
            can_claim_rewards: None,
            can_cast_vote: None,
        };

        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::OpenLiquidityRequest {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 6
        // Test error case ContractError::Unauthorized {}
        // due to already existing liquidity request
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &open_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 7
        // Verify that the correct info for the liquidity request was stored
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(
            info.liquidity_request,
            Some(ActiveOption {
                lender: None,
                state: None,
                msg: option
            })
        );
    }

    #[test]
    fn test_open_liquidity_request_fixed_interest_rental() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
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

        // Step 3
        // Test error case ContractError::InvalidLiquidityRequestOption {}
        // ------------------------------------------------------------------------------
        let open_liquidity_request_msg = ExecuteMsg::OpenLiquidityRequest {
            option: crate::msg::LiquidityRequestOptionMsg::FixedInterestRental {
                requested_amount: Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount: Uint128::zero(),
                },
                claimable_tokens: Uint128::zero(),
                can_cast_vote: None,
            },
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &open_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 4
        // Create a valid liquidity request
        // ------------------------------------------------------------------------------
        let option = LiquidityRequestOptionMsg::FixedInterestRental {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            claimable_tokens: amount,
            can_cast_vote: None,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::OpenLiquidityRequest {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 6
        // Test error case ContractError::Unauthorized {}
        // due to already existing liquidity request
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &open_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 7
        // Verify that the correct info for theliquidity requestwas stored
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(
            info.liquidity_request,
            Some(ActiveOption {
                lender: None,
                state: None,
                msg: option,
            })
        );
    }

    #[test]
    fn test_open_liquidity_request_fixed_term_rental() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
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

        // Step 3
        // Test error case ContractError::InvalidLiquidityRequestOption {}
        // ------------------------------------------------------------------------------
        let invalid_liquidity_request_msg = ExecuteMsg::OpenLiquidityRequest {
            option: crate::msg::LiquidityRequestOptionMsg::FixedTermRental {
                requested_amount: Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount: Uint128::zero(),
                },
                duration_in_seconds: 0u64,
                can_cast_vote: None,
            },
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &invalid_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 4
        // Create a valid liquidity request
        // ------------------------------------------------------------------------------
        let valid_liquidity_request_msg = LiquidityRequestOptionMsg::FixedTermRental {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            duration_in_seconds: 60u64,
            can_cast_vote: None,
        };

        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::OpenLiquidityRequest {
                    option: valid_liquidity_request_msg.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 6
        // Test error case ContractError::Unauthorized {}
        // due to already existing liquidity request
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &valid_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 7
        // Verify that the correct info for the liquidity request was stored
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(
            info.liquidity_request,
            Some(ActiveOption {
                lender: None,
                state: None,
                msg: valid_liquidity_request_msg
            })
        );
    }

    #[test]
    fn test_close_pending_liquidity_request() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
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

        // Step 3
        // Test error case ContractError::Unauthorized {}
        // When there is no open liquidity request on the vault
        // ------------------------------------------------------------------------------
        let close_liquidity_request_msg = ExecuteMsg::CloseLiquidityRequest {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &close_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 4
        // Create a valid liquidity request
        // ------------------------------------------------------------------------------
        let option = LiquidityRequestOptionMsg::FixedTermRental {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            duration_in_seconds: 60u64,
            can_cast_vote: None,
        };

        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::OpenLiquidityRequest {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 5
        // unauthorized close pending with wrong owner
        // ------------------------------------------------------------------------------
        let wrong_owner = Addr::unchecked("WRONG_OWNER");
        router
            .execute_contract(
                wrong_owner,
                vault_c_addr.clone(),
                &close_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 6
        // close pending lro correctly
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &close_liquidity_request_msg,
                &[],
            )
            .unwrap();

        // Step 7
        // Verify that the open liquidity request was closed
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(info.liquidity_request, None);
    }

    #[test]
    fn test_accept_liquidity_request_fixed_term_loan() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
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

        // Step 3
        // Test error case ContractError::Unauthorized {}
        // When there is no open liquidity request on the vault
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest { is_lp_group: None };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 4
        // Create a valid FixedTermLoan liquidity request
        // ------------------------------------------------------------------------------
        let duration_in_seconds = 60u64;
        let option = LiquidityRequestOptionMsg::FixedTermLoan {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            collateral_amount: amount,
            duration_in_seconds,
            can_claim_rewards: None,
            can_cast_vote: None,
        };

        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::OpenLiquidityRequest {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 5
        // Test error case ContractError::InvalidInputAmount {}
        // by sending the wrong requested amount
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 6
        // Accept the option
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount,
                }],
            )
            .unwrap();

        // Step 7
        // Verify that the correct info for the liquidity request was updated with the option
        // state
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(
            info.liquidity_request,
            Some(ActiveOption {
                lender: Some(Addr::unchecked(USER)),
                state: Some(crate::state::LiquidityRequestOptionState::FixedTermLoan {
                    requested_amount: Coin {
                        denom: IBC_DENOM_1.to_string(),
                        amount,
                    },
                    collateral_amount: amount,
                    start_time: router.block_info().time,
                    last_claim_time: None,
                    end_time: router.block_info().time.plus_seconds(duration_in_seconds),
                    can_cast_vote: None,
                    is_lp_group: None
                }),
                msg: option
            })
        );

        // Step 8
        // Test error case ContractError::Unauthorized {}
        // Trying to accept an option that already has an active lender
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount,
                }],
            )
            .unwrap_err();
    }

    #[test]
    fn test_accept_liquidity_request_fixed_interest_rental() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
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

        // Step 3
        // Create a valid FixedInterestRental liquidity request
        // ------------------------------------------------------------------------------
        let option = LiquidityRequestOptionMsg::FixedInterestRental {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            claimable_tokens: amount,
            can_cast_vote: None,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::OpenLiquidityRequest {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 4
        // Test error case ContractError::InvalidInputAmount {}
        // by sending the wrong requested amount
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest { is_lp_group: None };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 5
        // Accept the option
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount,
                }],
            )
            .unwrap();

        // Step 6
        // Verify that the correct info for the liquidity request was updated with the option
        // state
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(
            info.liquidity_request,
            Some(ActiveOption {
                lender: Some(Addr::unchecked(USER)),
                state: Some(LiquidityRequestOptionState::FixedInterestRental {
                    requested_amount: Coin {
                        denom: IBC_DENOM_1.to_string(),
                        amount,
                    },
                    claimable_tokens: amount,
                    already_claimed: Uint128::zero(),
                    can_cast_vote: None,
                    is_lp_group: None
                }),
                msg: option
            })
        );

        // Step 7
        // Test error case ContractError::Unauthorized {}
        // Trying to accept an option that already has an active lender
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount,
                }],
            )
            .unwrap_err();
    }

    #[test]
    fn test_accept_liquidity_request_fixed_term_rental() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
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

        // Step 3
        // Create a valid FixedTermRental liquidity request
        // ------------------------------------------------------------------------------
        let duration_in_seconds = 60u64;
        let option = LiquidityRequestOptionMsg::FixedTermRental {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            duration_in_seconds,
            can_cast_vote: None,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::OpenLiquidityRequest {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 4
        // Test error case ContractError::InvalidInputAmount {}
        // by sending the wrong requested amount
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest { is_lp_group: None };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 5
        // Accept the option
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount,
                }],
            )
            .unwrap();

        // Step 6
        // Verify that the correct info for the liquidity request was updated with the option
        // state
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(
            info.liquidity_request,
            Some(ActiveOption {
                lender: Some(Addr::unchecked(USER)),
                state: Some(LiquidityRequestOptionState::FixedTermRental {
                    requested_amount: Coin {
                        denom: IBC_DENOM_1.to_string(),
                        amount,
                    },
                    start_time: router.block_info().time,
                    last_claim_time: router.block_info().time,
                    end_time: router.block_info().time.plus_seconds(duration_in_seconds),
                    can_cast_vote: None,
                    is_lp_group: None
                }),
                msg: option
            })
        );

        // Step 7
        // Test error case ContractError::Unauthorized {}
        // Trying to accept an option that already has an active lender
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount,
                }],
            )
            .unwrap_err();
    }

    #[test]
    fn test_claim_delegator_rewards() {
        // Step 1
        // Instantiate contract instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let vault_c_addr = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
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

        // Step 3
        // Delegate to VALIDATOR_TWO_ADDRESS
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_TWO_ADDRESS.to_string(),
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

        // Step 4
        // Foward the blockchain ahead by one year
        // ------------------------------------------------------------------------------
        router.update_block(|block| block.time = block.time.plus_seconds(60 * 60 * 24 * 365));

        // Step 5
        // try claim rewards from all validators
        // ------------------------------------------------------------------------------
        let claim_rewards_msg = ExecuteMsg::ClaimDelegatorRewards {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &claim_rewards_msg,
                &[],
            )
            .unwrap();

        // Step 6
        // verify by inspecting contract balance
        // ------------------------------------------------------------------------------
        let balance = bank_balance(&mut router, &vault_c_addr, STAKING_DENOM.into());
        assert_eq!(
            balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(2_00_000)
            }
        );
    }
}
