#[cfg(test)]
mod tests {
    use crate::{
        msg::{
            AllDelegationsResponse, ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg,
            StakingInfoResponse,
        },
        state::{
            ActiveOption, Config, LiquidityRequestMsg, LiquidityRequestState, INSTANTIATOR_ADDR,
        },
    };
    use cosmwasm_std::{
        testing::mock_env, Addr, Coin, Decimal, Delegation, Empty, Uint128, Validator,
    };
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor, StakingInfo};

    const USER: &str = "user";
    const LENDER: &str = "lender";
    const STAKING_DENOM: &str = "TOKEN";
    const IBC_DENOM_1: &str = "ibc/usdc_denom";
    const SUPPLY: u128 = 500_000_000u128;
    const VALIDATOR_ONE_ADDRESS: &str = "validator_one";
    const VALIDATOR_TWO_ADDRESS: &str = "validator_two";

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

            // Set the initial balances for LENDER
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(LENDER),
                    vec![
                        Coin {
                            denom: STAKING_DENOM.to_string(),
                            amount: Uint128::zero(),
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

    fn instantiate_vault(app: &mut App) -> (Addr, u64) {
        let code_id = app.store_code(contract_template());
        let template_contract_addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(INSTANTIATOR_ADDR),
                &InstantiateMsg {
                    owner_address: USER.to_string(),
                    from_code_id: code_id,
                    index_number: 1u64,
                },
                &[],
                "vault",
                None,
            )
            .unwrap();

        // return addr
        (template_contract_addr, code_id)
    }

    fn get_vault_info(app: &mut App, contract_address: &Addr) -> InfoResponse {
        let msg = QueryMsg::Info {};
        let result: InfoResponse = app.wrap().query_wasm_smart(contract_address, &msg).unwrap();

        result
    }

    fn get_vault_staking_info(app: &mut App, contract_address: &Addr) -> StakingInfoResponse {
        let msg = QueryMsg::StakingInfo {};
        let result: StakingInfoResponse =
            app.wrap().query_wasm_smart(contract_address, &msg).unwrap();

        result
    }
    fn get_all_delegations(app: &mut App, contract_address: &Addr) -> AllDelegationsResponse {
        let msg = QueryMsg::AllDelegations {};
        let result: AllDelegationsResponse =
            app.wrap().query_wasm_smart(contract_address, &msg).unwrap();

        result
    }

    #[test]
    fn test_instantiate() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, from_code_id) = instantiate_vault(&mut router);

        // Step 2
        // Query for the contract info to assert
        // that all important data was indeed saved
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(
            info.config,
            Config {
                owner: Addr::unchecked(USER),
                from_code_id: from_code_id,
                index_number: 1u64,
            }
        );
    }

    #[test]
    fn test_delegate() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _) = instantiate_vault(&mut router);

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
        // when trying to call delegate method when info.sender != owner
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
        // when trying to delegate to a validator that is not in the active set
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
        // when we try to delegate more than the amount held in the vault
        // ------------------------------------------------------------------------------
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount: amount + Uint128::new(1),
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
        // Call delegate method correctly
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
    fn test_delegate_with_fixed_term_loan() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _) = instantiate_vault(&mut router);

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
        // Call delegate method correctly
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
        // Open FixedTermLoan request
        // ------------------------------------------------------------------------------
        let requested_amount = Uint128::new(300_000);
        let interest_amount = Uint128::new(30_000);
        let one_year_duration = 60 * 60 * 24 * 365; // 1 year;
        let option = LiquidityRequestMsg::FixedTermLoan {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: requested_amount,
            },
            interest_amount,
            collateral_amount: requested_amount,
            duration_in_seconds: one_year_duration,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 5
        // Call delegate method correctly with open FixedTermLoan
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[Coin {
                    denom: STAKING_DENOM.to_string(),
                    amount,
                }],
            )
            .unwrap();

        // Step 6
        // Accept FixedTermLoan
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount: requested_amount,
                }],
            )
            .unwrap();

        // Step 7
        // Call delegate method correctly with active FixedTermLoan
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[Coin {
                    denom: STAKING_DENOM.to_string(),
                    amount,
                }],
            )
            .unwrap();

        // Step 8
        // Move time foward to expire FixedTermLoan
        // ------------------------------------------------------------------------------
        router.update_block(|block| block.time = block.time.plus_seconds(one_year_duration));

        // Step 9
        // Test error case ContractError::ClearOutstandingDebt {}
        // when calling delegate method after the fixed term loan expires
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[Coin {
                    denom: STAKING_DENOM.to_string(),
                    amount,
                }],
            )
            .unwrap_err();

        // Step 10
        // Repay FixedTermLoan to close option by sending the interest_amount to the contract
        //
        // We also include the 0.3% liquidity_comission that was deducted and sent to
        // INSTANTIATOR_ADDR when the option was accepted
        // ------------------------------------------------------------------------------
        let repay_loan_msg = ExecuteMsg::RepayLoan {};
        let liquidity_comission = Uint128::new(900);
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &repay_loan_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount: interest_amount + liquidity_comission,
                }],
            )
            .unwrap();

        // Step 11
        // Verify that the option on the vault is closed,
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(info.liquidity_request, None);

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
                amount: amount + amount + amount
            }
        );
    }

    #[test]
    fn test_delegate_with_active_rental_option() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _) = instantiate_vault(&mut router);

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
        // Call delegate method correctly
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::Delegate {
                    validator: VALIDATOR_ONE_ADDRESS.to_string(),
                    amount,
                },
                &[],
            )
            .unwrap();

        // Step 4
        // Open FixedTermRental liquidity request
        // ------------------------------------------------------------------------------
        let one_year_duration = 60 * 60 * 24 * 365;
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: LiquidityRequestMsg::FixedTermRental {
                        requested_amount: Coin {
                            denom: IBC_DENOM_1.to_string(),
                            amount,
                        },
                        duration_in_seconds: one_year_duration,
                        can_cast_vote: false,
                    },
                },
                &[],
            )
            .unwrap();

        // Step 5
        // Accept FixedTermRental
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest {};
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
        // Move the chain foward to accumulate rewards
        // ------------------------------------------------------------------------------
        let expected_claimed_rewards = Uint128::new(100_000);
        router.update_block(|block| block.time = block.time.plus_seconds(one_year_duration));

        // Step 7
        // Call delegate method correctly
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::Delegate {
                    validator: VALIDATOR_ONE_ADDRESS.to_string(),
                    amount,
                },
                &[Coin {
                    denom: STAKING_DENOM.to_string(),
                    amount,
                }],
            )
            .unwrap();

        // Step 8
        // Ensure that the claimed accumulated staking rewards went to the lender
        // ------------------------------------------------------------------------------
        let lender_balance =
            bank_balance(&mut router, &Addr::unchecked(USER), STAKING_DENOM.into());
        assert_eq!(
            lender_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(SUPPLY) - (amount + amount) + expected_claimed_rewards
            }
        );

        // Step 9
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
                amount: amount + amount
            }
        );
    }

    #[test]
    fn test_redelegate() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

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
        // when trying to call delegate method when info.sender != owner
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
        // Redelegate tokens to VALIDATOR_TWO_ADDRESS
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
    fn test_redelegate_with_active_rental_option() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

        // Step 2
        // delegate some tokens to VALIDATOR_ONE_ADDRESS and VALIDATOR_TWO_ADDRESS
        // ------------------------------------------------------------------------------
        let amount = Uint128::new(1_000_000);
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::Delegate {
                    validator: VALIDATOR_ONE_ADDRESS.to_string(),
                    amount,
                },
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount,
                }],
            )
            .unwrap();

        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::Delegate {
                    validator: VALIDATOR_TWO_ADDRESS.to_string(),
                    amount,
                },
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount,
                }],
            )
            .unwrap();

        // Step 3
        // Open a rental option on the vault
        // ------------------------------------------------------------------------------
        let one_year_duration = 60 * 60 * 24 * 365;
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: LiquidityRequestMsg::FixedTermRental {
                        requested_amount: Coin {
                            denom: IBC_DENOM_1.to_string(),
                            amount,
                        },
                        duration_in_seconds: one_year_duration + one_year_duration,
                        can_cast_vote: false,
                    },
                },
                &[],
            )
            .unwrap();

        // Step 4
        // Accept liquidity request option by LENDER
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest {};
        router
            .execute_contract(
                Addr::unchecked(LENDER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount,
                }],
            )
            .unwrap();

        // Step 5
        // Test error case ContractError::LenderCannotRedelegateFromActiveValidator {}
        // when we try to redelegate from an active validator by LENDER
        let redelegate_msg = ExecuteMsg::Redelegate {
            src_validator: VALIDATOR_ONE_ADDRESS.to_string(),
            dst_validator: VALIDATOR_TWO_ADDRESS.to_string(),
            amount,
        };
        router
            .execute_contract(
                Addr::unchecked(LENDER),
                vault_c_addr.clone(),
                &redelegate_msg,
                &[],
            )
            .unwrap_err();

        // Step 6
        // Move the chain foward to accumulate rewards
        // ------------------------------------------------------------------------------
        let expected_claimed_rewards = Uint128::new(100_000);
        router.update_block(|block| block.time = block.time.plus_seconds(one_year_duration));

        // Step 7
        // Redelegate tokens to VALIDATOR_TWO_ADDRESS as vault owner
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &redelegate_msg,
                &[],
            )
            .unwrap();

        // Step 8
        // Ensure that the claimed accumulated staking rewards from VALIDATOR_ONE_ADDRESS
        // and VALIDATOR_TWO_ADDRESS went to LENDER
        // ------------------------------------------------------------------------------
        let lender_balance =
            bank_balance(&mut router, &Addr::unchecked(LENDER), STAKING_DENOM.into());
        assert_eq!(
            lender_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: expected_claimed_rewards + expected_claimed_rewards
            }
        );

        // Step 9
        // verify that VALIDATOR_TWO_ADDRESS now has all the delegations of user
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
                amount: amount + amount
            }
        );
    }

    #[test]
    fn test_undelegate() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

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
        // Verify that the correct amount was staked
        // ------------------------------------------------------------------------------
        let all_delegations = get_all_delegations(&mut router, &vault_c_addr);
        assert_eq!(
            all_delegations,
            AllDelegationsResponse {
                data: vec![Delegation {
                    delegator: vault_c_addr.clone(),
                    validator: VALIDATOR_ONE_ADDRESS.to_string(),
                    amount: Coin {
                        denom: STAKING_DENOM.to_string(),
                        amount
                    }
                }]
            }
        );

        // Step 5
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

        // Step 6
        // Test error case ContractError::MaxUndelegateAmountExceeded {}
        // ------------------------------------------------------------------------------
        let undelegate_msg = ExecuteMsg::Undelegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount: amount + Uint128::new(1),
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &undelegate_msg,
                &[],
            )
            .unwrap_err();

        // Step 7
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

        // Step 8
        // Foward the blockchain ahead and process unbonding queue
        // ------------------------------------------------------------------------------
        router.update_block(|block| block.time = block.time.plus_seconds(10));
        router
            .sudo(cw_multi_test::SudoMsg::Staking(
                cw_multi_test::StakingSudo::ProcessQueue {},
            ))
            .unwrap();

        // Step 9
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

        // Step 10
        // Verify that all delegations is empty
        // ------------------------------------------------------------------------------
        let all_delegations = get_all_delegations(&mut router, &vault_c_addr);
        assert_eq!(all_delegations, AllDelegationsResponse { data: vec![] });
    }

    #[test]
    fn test_request_liquidity_fixed_term_loan() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

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
        // when info.sender != owner
        // ------------------------------------------------------------------------------
        let wrong_owner = Addr::unchecked("WRONG_OWNER");
        router
            .execute_contract(
                wrong_owner,
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: LiquidityRequestMsg::FixedTermLoan {
                        requested_amount: Coin {
                            denom: IBC_DENOM_1.to_string(),
                            amount,
                        },
                        interest_amount: Uint128::zero(),
                        collateral_amount: amount,
                        duration_in_seconds: 60u64,
                    },
                },
                &[],
            )
            .unwrap_err();

        // Step 4
        // Test error case ContractError::InvalidLiquidityRequestOption {}
        // When we send in zero value for interest_amount, duration_in_seconds or collateral_amount
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: LiquidityRequestMsg::FixedTermLoan {
                        requested_amount: Coin {
                            denom: IBC_DENOM_1.to_string(),
                            amount,
                        },
                        interest_amount: Uint128::zero(),
                        duration_in_seconds: 0u64,
                        collateral_amount: Uint128::zero(),
                    },
                },
                &[],
            )
            .unwrap_err();

        // Step 5
        // Create a valid liquidity request
        // ------------------------------------------------------------------------------
        let duration_in_seconds = 60u64;
        let liquidity_request = LiquidityRequestMsg::FixedTermLoan {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            interest_amount: Uint128::zero(),
            collateral_amount: amount,
            duration_in_seconds,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: liquidity_request.clone(),
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
                &liquidity_request,
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
                msg: liquidity_request
            })
        );
    }

    #[test]
    fn test_request_liquidity_fixed_interest_rental() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

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
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: LiquidityRequestMsg::FixedInterestRental {
                        requested_amount: Coin {
                            denom: IBC_DENOM_1.to_string(),
                            amount: Uint128::zero(),
                        },
                        claimable_tokens: Uint128::zero(),
                        can_cast_vote: false,
                    },
                },
                &[],
            )
            .unwrap_err();

        // Step 4
        // Create a valid liquidity request
        // ------------------------------------------------------------------------------
        let liquidity_request = LiquidityRequestMsg::FixedInterestRental {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            claimable_tokens: amount,
            can_cast_vote: false,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: liquidity_request.clone(),
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
                &liquidity_request,
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
                msg: liquidity_request,
            })
        );
    }

    #[test]
    fn test_request_liquidity_fixed_term_rental() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

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
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: LiquidityRequestMsg::FixedTermRental {
                        requested_amount: Coin {
                            denom: IBC_DENOM_1.to_string(),
                            amount: Uint128::zero(),
                        },
                        duration_in_seconds: 0u64,
                        can_cast_vote: false,
                    },
                },
                &[],
            )
            .unwrap_err();

        // Step 4
        // Create a valid liquidity request
        // ------------------------------------------------------------------------------
        let valid_liquidity_request_msg = LiquidityRequestMsg::FixedTermRental {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            duration_in_seconds: 60u64,
            can_cast_vote: false,
        };

        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
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
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

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
        let close_liquidity_request_msg = ExecuteMsg::ClosePendingLiquidityRequest {};
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
        let requested_amount = Coin {
            denom: IBC_DENOM_1.to_string(),
            amount,
        };
        let option = LiquidityRequestMsg::FixedTermRental {
            requested_amount: requested_amount.clone(),
            duration_in_seconds: 60u64,
            can_cast_vote: false,
        };

        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
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
        // close pending liquidity request correctly
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

        // Step 8
        // Open another liquidity request
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 9
        // Accept the liquidity request
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::AcceptLiquidityRequest {},
                &[requested_amount],
            )
            .unwrap();

        // Step 3
        // Test error case ContractError::LiquidityRequestIsActive {}
        // When owner tries to call ClosePendingLiquidityRequest when the options
        // has already been accepted.
        // ------------------------------------------------------------------------------
        let close_liquidity_request_msg = ExecuteMsg::ClosePendingLiquidityRequest {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &close_liquidity_request_msg,
                &[],
            )
            .unwrap_err();
    }

    #[test]
    fn test_accept_liquidity_request_fixed_term_loan() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

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
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest {};
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
        let option = LiquidityRequestMsg::FixedTermLoan {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            interest_amount: Uint128::zero(),
            collateral_amount: amount,
            duration_in_seconds,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
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
        // Accept the open liquidity request
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
        // Verify that the correct info for the liquidity request was updated
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(
            info.liquidity_request,
            Some(ActiveOption {
                lender: Some(Addr::unchecked(USER)),
                state: Some(crate::state::LiquidityRequestState::FixedTermLoan {
                    requested_amount: Coin {
                        denom: IBC_DENOM_1.to_string(),
                        amount,
                    },
                    interest_amount: Uint128::zero(),
                    collateral_amount: amount,
                    start_time: router.block_info().time,
                    end_time: router.block_info().time.plus_seconds(duration_in_seconds),
                    last_liquidation_date: None,
                    already_claimed: Uint128::zero(),
                    processing_liquidation: false
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

        // Step 9
        // Verify that liquidity_request_commission was paid to INSTANTIATOR_ADDR
        // The liquidity_request_commission is calculated as 0.3% of requested amount
        // ------------------------------------------------------------------------------
        let liquidity_request_commission = Uint128::new(3_000);
        let instantiator_balance = bank_balance(
            &mut router,
            &Addr::unchecked(INSTANTIATOR_ADDR),
            IBC_DENOM_1.into(),
        );
        assert_eq!(
            instantiator_balance,
            Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: liquidity_request_commission
            }
        );
    }

    #[test]
    fn test_accept_liquidity_request_fixed_interest_rental() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

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
        let option = LiquidityRequestMsg::FixedInterestRental {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            claimable_tokens: amount,
            can_cast_vote: false,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 4
        // Test error case ContractError::InvalidInputAmount {}
        // by sending the wrong requested amount
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest {};
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
                state: Some(LiquidityRequestState::FixedInterestRental {
                    requested_amount: Coin {
                        denom: IBC_DENOM_1.to_string(),
                        amount,
                    },
                    claimable_tokens: amount,
                    already_claimed: Uint128::zero(),
                    can_cast_vote: false,
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

        // Step 8
        // Verify that liquidity_request_commission was paid to INSTANTIATOR_ADDR
        // The liquidity_request_commission is calculated as 0.3% of requested amount
        // ------------------------------------------------------------------------------
        let liquidity_request_commission = Uint128::new(3_000);
        let instantiator_balance = bank_balance(
            &mut router,
            &Addr::unchecked(INSTANTIATOR_ADDR),
            IBC_DENOM_1.into(),
        );
        assert_eq!(
            instantiator_balance,
            Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: liquidity_request_commission
            }
        );
    }

    #[test]
    fn test_accept_liquidity_request_fixed_term_rental() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

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
        let option = LiquidityRequestMsg::FixedTermRental {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount,
            },
            duration_in_seconds,
            can_cast_vote: false,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 4
        // Test error case ContractError::InvalidInputAmount {}
        // by sending the wrong requested amount
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[],
            )
            .unwrap_err();

        // Step 5
        // Foward the blockchain ahead by one year
        // ------------------------------------------------------------------------------
        router.update_block(|block| block.time = block.time.plus_seconds(60 * 60 * 24 * 365));

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
        // Verify that the vault contains the balance of the accumulated_rewards claimed
        // ------------------------------------------------------------------------------
        let balance = bank_balance(&mut router, &vault_c_addr, STAKING_DENOM.into());
        assert_eq!(
            balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(100_000)
            }
        );

        // Step 8
        // Verify that the correct info for the liquidity request was updated with the option
        // state
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(
            info.liquidity_request,
            Some(ActiveOption {
                lender: Some(Addr::unchecked(USER)),
                state: Some(LiquidityRequestState::FixedTermRental {
                    requested_amount: Coin {
                        denom: IBC_DENOM_1.to_string(),
                        amount,
                    },
                    start_time: router.block_info().time,
                    last_claim_time: router.block_info().time,
                    end_time: router.block_info().time.plus_seconds(duration_in_seconds),
                    can_cast_vote: false,
                }),
                msg: option
            })
        );

        // Step 9
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

        // Step 10
        // Verify that liquidity_request_commission was paid to INSTANTIATOR_ADDR
        // The liquidity_request_commission is calculated as 0.3% of requested amount
        // ------------------------------------------------------------------------------
        let liquidity_request_commission = Uint128::new(3_000);
        let instantiator_balance = bank_balance(
            &mut router,
            &Addr::unchecked(INSTANTIATOR_ADDR),
            IBC_DENOM_1.into(),
        );
        assert_eq!(
            instantiator_balance,
            Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: liquidity_request_commission
            }
        );
    }

    #[test]
    fn test_claim_delegator_rewards_no_liquidity_request() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

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
        // query_staking_info and verify data
        // ------------------------------------------------------------------------------
        let staking_info = get_vault_staking_info(&mut router, &vault_c_addr);
        assert_eq!(
            staking_info,
            StakingInfoResponse {
                total_staked: Uint128::new(2000000),
                accumulated_rewards: Uint128::new(200000)
            }
        );

        // Step 6
        // Claim rewards from all validators
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

        // Step 7
        // Verify by inspecting contract balance
        // ------------------------------------------------------------------------------
        let balance = bank_balance(&mut router, &vault_c_addr, STAKING_DENOM.into());
        assert_eq!(
            balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: staking_info.accumulated_rewards
            }
        );
    }

    #[test]
    fn test_claim_delegator_rewards_with_fixed_interest_rental() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let delegated_amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount: delegated_amount,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount: delegated_amount,
                }],
            )
            .unwrap();

        // Step 3
        // Create a valid FixedInterestRental liquidity request
        // ------------------------------------------------------------------------------
        let requested_amount = Uint128::new(350_000);
        let option = LiquidityRequestMsg::FixedInterestRental {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: requested_amount,
            },
            claimable_tokens: requested_amount,
            can_cast_vote: false,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 4
        // Accept the option
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount: requested_amount,
                }],
            )
            .unwrap();

        // Step 5
        // Fast foward the chain to a future date, where
        // the rewards has not fully covered the claimable_tokens
        // ------------------------------------------------------------------------------
        let expected_claims_after_two_years = Uint128::new(200_000);
        router.update_block(|block| block.time = block.time.plus_seconds(60 * 60 * 24 * 365 * 2));

        // Step 6
        // Claim rewards from the validator
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

        // Step 7
        // Verify to ensure vault balance is zero and lender's balance contains
        // expected_claims_after_two_years
        // ------------------------------------------------------------------------------
        let vault_balance = bank_balance(&mut router, &vault_c_addr, STAKING_DENOM.into());
        let lender_balance =
            bank_balance(&mut router, &Addr::unchecked(USER), STAKING_DENOM.into());
        assert_eq!(
            vault_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(0)
            }
        );
        assert_eq!(
            lender_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(SUPPLY) - delegated_amount + expected_claims_after_two_years
            }
        );

        // Step 8
        // Verify that the correct info for the liquidity request
        // was updated in the option state
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(
            info.liquidity_request,
            Some(ActiveOption {
                lender: Some(Addr::unchecked(USER)),
                state: Some(LiquidityRequestState::FixedInterestRental {
                    requested_amount: Coin {
                        denom: IBC_DENOM_1.to_string(),
                        amount: requested_amount,
                    },
                    claimable_tokens: requested_amount,
                    already_claimed: expected_claims_after_two_years,
                    can_cast_vote: false,
                }),
                msg: option
            })
        );

        // Step 9
        // Fast foward the chain to a future date, where
        // the rewards has fully covered the claimable_tokens
        // ------------------------------------------------------------------------------
        router.update_block(|block| block.time = block.time.plus_seconds(60 * 60 * 24 * 365 * 2));

        // Step 10
        // try claim rewards from the validator
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

        // Step 11
        // Verify to ensure vault balance is
        // (2 * expected_claims_after_two_years) - requsted_amount,
        // and the lender's balance now has the requested_amount
        // ------------------------------------------------------------------------------
        let vault_balance = bank_balance(&mut router, &vault_c_addr, STAKING_DENOM.into());
        let lender_balance =
            bank_balance(&mut router, &Addr::unchecked(USER), STAKING_DENOM.into());
        assert_eq!(
            vault_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: (expected_claims_after_two_years + expected_claims_after_two_years)
                    - requested_amount
            }
        );
        assert_eq!(
            lender_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(SUPPLY) - delegated_amount + requested_amount
            }
        );

        // Step 12
        // Verify that the option has been finalized
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(info.liquidity_request, None);
    }

    #[test]
    fn test_claim_delegator_rewards_with_fixed_term_rental() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let delegated_amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount: delegated_amount,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount: delegated_amount,
                }],
            )
            .unwrap();

        // Step 3
        // Create a valid FixedTermRental liquidity request
        // ------------------------------------------------------------------------------
        let requested_amount = Uint128::new(350_000);
        let duration_in_seconds = 60 * 60 * 24 * 365 * 3;
        let expected_rewards_after_duration = Uint128::new(300_000);
        let option = LiquidityRequestMsg::FixedTermRental {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: Uint128::new(350_000),
            },
            duration_in_seconds,
            can_cast_vote: false,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 4
        // Accept the option
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount: requested_amount,
                }],
            )
            .unwrap();

        // Record start time
        let start_time = router.block_info().time;

        // Step 5
        // fast foward the chain to a future date, before the end_timme
        // ------------------------------------------------------------------------------
        let expected_claims_after_two_years = Uint128::new(200_000);
        let two_years = 60 * 60 * 24 * 365 * 2;
        router.update_block(|block| block.time = block.time.plus_seconds(two_years));

        // Step 6
        // try claim rewards from the validator
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

        // Record last claim time
        let last_claim_time = router.block_info().time;

        // Step 7
        // verify to ensure vault balance is zero and lender's balance contains
        // expected_claims_after_two_years
        // ------------------------------------------------------------------------------
        let vault_balance = bank_balance(&mut router, &vault_c_addr, STAKING_DENOM.into());
        let lender_balance =
            bank_balance(&mut router, &Addr::unchecked(USER), STAKING_DENOM.into());
        assert_eq!(
            vault_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(0)
            }
        );
        assert_eq!(
            lender_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(SUPPLY) - delegated_amount + expected_claims_after_two_years
            }
        );

        // Step 8
        // Verify that the correct info for the liquidity request
        // was updated in the option state
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(
            info.liquidity_request,
            Some(ActiveOption {
                lender: Some(Addr::unchecked(USER)),
                state: Some(LiquidityRequestState::FixedTermRental {
                    requested_amount: Coin {
                        denom: IBC_DENOM_1.to_string(),
                        amount: requested_amount,
                    },
                    start_time,
                    last_claim_time,
                    end_time: start_time.plus_seconds(duration_in_seconds),
                    can_cast_vote: false,
                }),
                msg: option
            })
        );

        // Step 9
        // fast foward the chain to a future date greater than the end_time
        // ------------------------------------------------------------------------------
        router.update_block(|block| block.time = block.time.plus_seconds(two_years));

        // Step 10
        // try claim rewards from the validator
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

        // Step 11
        // verify to ensure vault balance is
        // (2 * expected_claims_after_two_years) - expected_rewards_after_duration,
        // and the lender's balance now has the expected_rewards_after_duration
        // ------------------------------------------------------------------------------
        let vault_balance = bank_balance(&mut router, &vault_c_addr, STAKING_DENOM.into());
        let lender_balance =
            bank_balance(&mut router, &Addr::unchecked(USER), STAKING_DENOM.into());
        assert_eq!(
            vault_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: (expected_claims_after_two_years + expected_claims_after_two_years)
                    - expected_rewards_after_duration
            }
        );
        assert_eq!(
            lender_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(SUPPLY) - delegated_amount + expected_rewards_after_duration
            }
        );

        // Step 12
        // Verify that the option has been finalized
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(info.liquidity_request, None);
    }

    #[test]
    fn test_repay_loan() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS
        // ------------------------------------------------------------------------------
        let delegated_amount = Uint128::new(1_000_000);
        let delegate_msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ONE_ADDRESS.to_string(),
            amount: delegated_amount,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &delegate_msg,
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount: delegated_amount,
                }],
            )
            .unwrap();

        // Step 3
        // Try to call repay loan with ContractError::Unauthorized {}
        // because there is no active liquidity request
        // ------------------------------------------------------------------------------
        let repay_loan_msg = ExecuteMsg::RepayLoan {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &repay_loan_msg,
                &[],
            )
            .unwrap_err();

        // Step 4
        // Create a valid FixedTermLoan liquidity request
        // ------------------------------------------------------------------------------
        let requested_amount = Uint128::new(300_000);
        let interest_amount = Uint128::new(30_000);
        let duration_in_seconds = 60u64;
        let option = LiquidityRequestMsg::FixedTermLoan {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: requested_amount,
            },
            interest_amount,
            collateral_amount: requested_amount,
            duration_in_seconds,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 5
        // Try to call repay loan with ContractError::Unauthorized {}
        // because there is a liquidity request that has not been accepted yet
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &repay_loan_msg,
                &[],
            )
            .unwrap_err();

        // Step 6
        // Accept the open liquidity request
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount: requested_amount,
                }],
            )
            .unwrap();

        // Step 7
        // Try to call repay loan with ContractError::InsufficientBalance {}
        // because at this point, the vault contract still has the requested_amount
        // sent to it by the lender when they accept the liquidity request
        // but does not have the extra interest_amount required
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &repay_loan_msg,
                &[],
            )
            .unwrap_err();

        // Step 8
        // Try to repay the loan correctly by sending the interest_amount to the contract
        //
        // We also include the 0.3% liquidity_comission that was deducted from the requested_amount
        // and sent to INSTANTIATOR_ADDR when the option was accepted
        // ------------------------------------------------------------------------------
        let liquidity_comission = Uint128::new(900);
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &repay_loan_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount: interest_amount + liquidity_comission,
                }],
            )
            .unwrap();

        // Step 9
        // Verify that the option on the vault is closed,
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(info.liquidity_request, None);

        // Step 10
        // Verify that the lender got the amount paid. In this case, given the lender
        // is still USER, then his balance for IBC_DENOM_1 should be equal SUPPLY  - liquidity_comission
        //
        // Also verify that the vault balance for IBC_DENOM_1 is zero as the balance
        // has been used to clear the debt
        // ------------------------------------------------------------------------------
        let vault_balance = bank_balance(&mut router, &vault_c_addr, IBC_DENOM_1.into());
        assert_eq!(
            vault_balance,
            Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: Uint128::new(0)
            }
        );

        let lender_balance = bank_balance(&mut router, &Addr::unchecked(USER), IBC_DENOM_1.into());
        assert_eq!(
            lender_balance,
            Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: Uint128::new(SUPPLY) - liquidity_comission
            }
        );
    }

    #[test]
    fn test_liquidate_collateral() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

        // Step 2
        // Delegate to VALIDATOR_ONE_ADDRESS & VALIDATOR_TWO_ADDRESS
        // ------------------------------------------------------------------------------
        let delegated_amount = Uint128::new(1_000_000);
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::Delegate {
                    validator: VALIDATOR_ONE_ADDRESS.to_string(),
                    amount: delegated_amount,
                },
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount: delegated_amount,
                }],
            )
            .unwrap();
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::Delegate {
                    validator: VALIDATOR_TWO_ADDRESS.to_string(),
                    amount: delegated_amount,
                },
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount: delegated_amount,
                }],
            )
            .unwrap();

        // Step 3
        // Error liquidating fixed term loan when no option is available
        // ------------------------------------------------------------------------------
        let liquidation_msg = ExecuteMsg::LiquidateCollateral {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &liquidation_msg,
                &[],
            )
            .unwrap_err();

        // Step 4
        // Create a valid FixedTermLoan liquidity request
        // ------------------------------------------------------------------------------
        let requested_amount = Uint128::new(300_000);
        let interest_amount = Uint128::new(30_000);
        let one_year_duration = 60 * 60 * 24 * 365; // 1 year;
        let option = LiquidityRequestMsg::FixedTermLoan {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: requested_amount,
            },
            interest_amount,
            collateral_amount: requested_amount,
            duration_in_seconds: one_year_duration,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 5
        // Test error case ContractError::Unauthorized {}
        // because there is a liquidity request that has not been accepted yet
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &liquidation_msg,
                &[],
            )
            .unwrap_err();

        // Step 6
        // Accept the open liquidity request
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount: requested_amount,
                }],
            )
            .unwrap();

        // Step 7
        // Try to call liquidate collateral with ContractError::Unauthorized {}
        // because there is a fixed term loans that have not yet expired
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &liquidation_msg,
                &[],
            )
            .unwrap_err();

        // Step 8
        // Fast foward the time so the option expires without repayment
        // ------------------------------------------------------------------------------
        router.update_block(|block| block.time = block.time.plus_seconds(one_year_duration));

        // Step 9
        // Try to call liquidate collateral with ContractError::Unauthorized {}
        // when a wrong_user is calling this method
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked("WRONG_OWNER"),
                vault_c_addr.clone(),
                &liquidation_msg,
                &[],
            )
            .unwrap_err();

        // Step 10
        // Begin liquidation of collateral, partial liquidation
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &liquidation_msg,
                &[],
            )
            .unwrap();

        // Step 11
        // Verify that accumulated staking rewards and
        // available staking_demon vault balance was send to lender
        // ------------------------------------------------------------------------------
        let expected_claims_after_one_years = Uint128::new(200_000);
        let lender_balance =
            bank_balance(&mut router, &Addr::unchecked(USER), STAKING_DENOM.into());
        assert_eq!(
            lender_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(SUPPLY) - delegated_amount - delegated_amount
                    + expected_claims_after_one_years
            }
        );

        // Step 12
        // Try to repay loan with ContractError::Unauthorized {}
        // when liquidation is processing
        // ------------------------------------------------------------------------------
        let repay_loan_msg = ExecuteMsg::RepayLoan {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &repay_loan_msg,
                &[],
            )
            .unwrap_err();

        // Step 13
        // Foward the blockchain ahead and process unbonding queue
        // ------------------------------------------------------------------------------
        router.update_block(|block| block.time = block.time.plus_seconds(10));
        router
            .sudo(cw_multi_test::SudoMsg::Staking(
                cw_multi_test::StakingSudo::ProcessQueue {},
            ))
            .unwrap();

        // Step 14
        // Verify that vault balance contains the expected_outstanding_balance
        // ------------------------------------------------------------------------------
        let expected_outstanding_balance = Uint128::new(100_000);
        let vault_balance = bank_balance(&mut router, &vault_c_addr, STAKING_DENOM.into());
        assert_eq!(
            vault_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: expected_outstanding_balance
            }
        );

        // Step 15
        // Complete liquidation of collateral
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &liquidation_msg,
                &[],
            )
            .unwrap();

        // Step 16
        // Verify that outstanding balance was send to lender
        // ------------------------------------------------------------------------------
        let lender_balance =
            bank_balance(&mut router, &Addr::unchecked(USER), STAKING_DENOM.into());
        assert_eq!(
            lender_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(SUPPLY) - delegated_amount - delegated_amount
                    + expected_claims_after_one_years
                    + expected_outstanding_balance
            }
        );

        // Step 17
        // Verify that the option has been finalized
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(info.liquidity_request, None);
    }

    #[test]
    fn test_withdraw_balance() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

        // Step 2
        // Delegate 600_000 STAKING_DENOM to VALIDATOR_ONE_ADDRESS
        // Leave 1_000_000 STAKING_DENOM as balance
        // ------------------------------------------------------------------------------
        let total_sent_to_vult = Uint128::new(1_600_000);
        let delegated_amount = Uint128::new(600_000);
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::Delegate {
                    validator: VALIDATOR_ONE_ADDRESS.to_string(),
                    amount: delegated_amount,
                },
                &[Coin {
                    denom: STAKING_DENOM.into(),
                    amount: total_sent_to_vult,
                }],
            )
            .unwrap();

        // Step 3
        // Withdraw 100_000 STAKING_DENOM leaving 900_000 balance
        // ------------------------------------------------------------------------------
        let withdraw_balance_msg = ExecuteMsg::WithdrawBalance {
            to_address: None,
            funds: Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(100_000),
            },
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap();

        // Step 4
        // Create a fixed term loan liquidity using 600_000 STAKING_DENOM as collateral
        // ------------------------------------------------------------------------------
        let requested_amount = Uint128::new(300_000);
        let collateral_amount = Uint128::new(600_000);
        let one_year_duration = 60 * 60 * 24 * 365; // 1 year;
        let option = LiquidityRequestMsg::FixedTermLoan {
            requested_amount: Coin {
                denom: IBC_DENOM_1.to_string(),
                amount: requested_amount,
            },
            interest_amount: Uint128::new(30_000),
            collateral_amount,
            duration_in_seconds: one_year_duration,
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &ExecuteMsg::RequestLiquidity {
                    option: option.clone(),
                },
                &[],
            )
            .unwrap();

        // Step 5
        // Withdraw 100_000 STAKING_DENOM leaving 800_000 balance
        // ------------------------------------------------------------------------------
        let withdraw_balance_msg = ExecuteMsg::WithdrawBalance {
            to_address: None,
            funds: Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(100_000),
            },
        };
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap();

        // Step 6
        // Accept the fixed term loan
        // ------------------------------------------------------------------------------
        let accept_liquidity_request_msg = ExecuteMsg::AcceptLiquidityRequest {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &accept_liquidity_request_msg,
                &[Coin {
                    denom: IBC_DENOM_1.to_string(),
                    amount: requested_amount,
                }],
            )
            .unwrap();

        // Step 7
        // Withdraw 100_000 STAKING_DENOM leaving 700_000 balance
        // Withdrawal still possible because the fixed term loan has not expired yet
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap();

        // Step 8
        // Expire the fixed term loan without repayment
        // ------------------------------------------------------------------------------
        router.update_block(|block| block.time = block.time.plus_seconds(one_year_duration));

        // Step 8
        // Try to withdraw some of the balance with ContractError::ClearOutstandingDebt {}
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap_err();

        // Step 9
        // Liquidate collateral to clear the debt
        // ------------------------------------------------------------------------------
        let liquidation_msg = ExecuteMsg::LiquidateCollateral {};
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &liquidation_msg,
                &[],
            )
            .unwrap();

        // Step 10
        // Withdraw the remaining 100_000 STAKING_DENOM from the vault
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                Addr::unchecked(USER),
                vault_c_addr.clone(),
                &withdraw_balance_msg,
                &[],
            )
            .unwrap();

        // Step 11
        // Verify that the vault balance is 60_000, which is the accumulated
        // staking rewards claimed during liquidation
        // ------------------------------------------------------------------------------
        let vault_balance = bank_balance(&mut router, &vault_c_addr, STAKING_DENOM.into());
        assert_eq!(
            vault_balance,
            Coin {
                denom: STAKING_DENOM.to_string(),
                amount: Uint128::new(60_000)
            }
        );
    }

    #[test]
    fn test_transfer_ownership() {
        // Step 1
        // Get vault instance
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let (vault_c_addr, _from_code_id) = instantiate_vault(&mut router);

        // Step 2
        // Test error case  ContractError::Unauthorized {}
        // When transfer_ownership is called by a user who is not the current
        // vault owner
        // ------------------------------------------------------------------------------
        let new_owner = "new_owner".to_string();
        let transfer_ownership_msg = ExecuteMsg::TransferOwnership {
            to_address: new_owner.clone(),
        };
        router
            .execute_contract(
                Addr::unchecked("fake_owner"),
                vault_c_addr.clone(),
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
                vault_c_addr.clone(),
                &transfer_ownership_msg,
                &[],
            )
            .unwrap();

        // Step 4
        // Query the vault info to verify the new owner
        // ------------------------------------------------------------------------------
        let info = get_vault_info(&mut router, &vault_c_addr);
        assert_eq!(info.config.owner, Addr::unchecked(new_owner));
    }
}
