#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg, TokenSelect};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg, Denom, Expiration};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    const USER: &str = "user";
    const NATIVE_DENOM: &str = "udenom";
    const IBC_DENOM_1: &str = "ibc/denom1";
    const IBC_DENOM_2: &str = "ibc/denom2";
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
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::from(SUPPLY),
                        },
                        Coin {
                            denom: IBC_DENOM_1.to_string(),
                            amount: Uint128::from(SUPPLY),
                        },
                        Coin {
                            denom: IBC_DENOM_2.to_string(),
                            amount: Uint128::from(SUPPLY),
                        },
                    ],
                )
                .unwrap();
        })
    }

    fn contract_cw20() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        ))
    }

    fn contract_template() -> Box<dyn Contract<Empty>> {
        Box::new(
            ContractWrapper::new(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            )
            .with_reply(crate::contract::reply),
        )
    }

    fn get_amm_contract_info(app: &mut App, contract_address: &Addr) -> InfoResponse {
        let msg = QueryMsg::Info {};

        let result: InfoResponse = app.wrap().query_wasm_smart(contract_address, &msg).unwrap();
        result
    }

    fn bank_balance(router: &mut App, addr: &Addr, denom: String) -> Coin {
        router
            .wrap()
            .query_balance(addr.to_string(), denom)
            .unwrap()
    }

    // CreateCW20 create new cw20 with given initial balance belonging to owner
    fn create_cw20_quote_token(
        router: &mut App,
        owner: &Addr,
        name: String,
        symbol: String,
        balance: Uint128,
    ) -> Cw20Contract {
        // set up cw20 contract with some tokens
        let cw20_id = router.store_code(contract_cw20());
        let msg = cw20_base::msg::InstantiateMsg {
            name,
            symbol,
            decimals: 2,
            initial_balances: vec![Cw20Coin {
                address: owner.to_string(),
                amount: balance,
            }],
            mint: None,
            marketing: None,
        };
        let addr = router
            .instantiate_contract(cw20_id, owner.clone(), &msg, &[], "CASH", None)
            .unwrap();
        Cw20Contract(addr)
    }

    fn _instantiate_amm_with_cw20_as_quote(app: &mut App, quote_token_addr: Addr) -> Addr {
        let template_id = app.store_code(contract_template());
        let lp_code_id = app.store_code(contract_cw20());

        let msg = InstantiateMsg {
            native_denom: Denom::Native(NATIVE_DENOM.to_string()),
            base_denom: Denom::Native(NATIVE_DENOM.to_string()),
            quote_denom: Denom::Cw20(quote_token_addr),
            lp_token_code_id: lp_code_id,
        };

        let template_contract_addr = app
            .instantiate_contract(
                template_id,
                Addr::unchecked(USER),
                &msg,
                &[],
                "token_swap",
                None,
            )
            .unwrap();

        // return addr
        template_contract_addr
    }

    fn _instantiate_amm_with_native_as_quote(app: &mut App, quote_token_denom: String) -> Addr {
        let template_id = app.store_code(contract_template());
        let lp_code_id = app.store_code(contract_cw20());

        let msg = InstantiateMsg {
            native_denom: Denom::Native(NATIVE_DENOM.into()),
            base_denom: Denom::Native(NATIVE_DENOM.into()),
            quote_denom: Denom::Native(quote_token_denom),
            lp_token_code_id: lp_code_id,
        };

        let template_contract_addr = app
            .instantiate_contract(
                template_id,
                Addr::unchecked(USER),
                &msg,
                &[],
                "token_swap",
                None,
            )
            .unwrap();

        // return addr
        template_contract_addr
    }

    #[test]
    fn test_instantiate() {
        let mut app = mock_app();

        // cw20 quote token contract
        let quote_token_contract = create_cw20_quote_token(
            &mut app,
            &Addr::unchecked(USER),
            "token".to_string(),
            "CWTOKEN".to_string(),
            Uint128::new(500_000_000),
        );

        let amm_addr = _instantiate_amm_with_cw20_as_quote(&mut app, quote_token_contract.addr());

        // Query for the contract info to assert that the lp token and other important
        // data was indeed saved
        let info = get_amm_contract_info(&mut app, &amm_addr);

        assert_eq!(
            info,
            InfoResponse {
                base_denom: Denom::Native(NATIVE_DENOM.to_string()),
                base_reserve: Uint128::zero(),
                quote_reserve: Uint128::zero(),
                quote_denom: Denom::Cw20(quote_token_contract.addr()),
                lp_token_supply: Uint128::zero(),
                lp_token_address: Addr::unchecked("contract2")
            }
        );
    }

    #[test]
    fn test_add_liquidity_with_cw20_as_quote() {
        // Step 1
        // Setup the mock app
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let owner = Addr::unchecked(USER);

        // cw20 quote token contract
        let quote_token_contract = create_cw20_quote_token(
            &mut router,
            &owner,
            "token".to_string(),
            "CWTOKEN".to_string(),
            Uint128::new(5000),
        );

        // amm contract instance
        let amm_addr =
            _instantiate_amm_with_cw20_as_quote(&mut router, quote_token_contract.addr());

        // make sure that quote_token_contract.addr() != amm_addr
        assert_ne!(quote_token_contract.addr(), amm_addr);

        // Query amm info
        let info = get_amm_contract_info(&mut router, &amm_addr);

        // Setup LP token helper
        let lp_token = Cw20Contract(Addr::unchecked(info.lp_token_address));

        // check quote_token balance for owner
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(5000));

        // Step 2
        // Add liquidity
        // ------------------------------------------------------------------------------

        // increase the spending allowance of the amm_contract on the quote_token_contract
        // on behalf of owner
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: Uint128::new(100u128),
            expires: None,
        };
        let _res = router
            .execute_contract(
                owner.clone(),
                quote_token_contract.addr(),
                &allowance_msg,
                &[],
            )
            .unwrap();

        // Step 3
        // Test error messages
        // ------------------------------------------------------------------------------

        // ContractError::MsgExpirationError {}
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(100),
            expiration: Some(Expiration::AtHeight(0)),
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100),
                }],
            )
            .unwrap_err();

        // ContractError::NonZeroInputAmountExpected {}
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(0),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100),
                }],
            )
            .unwrap_err();

        // ContractError::InsufficientFunds {}
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(100),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(50),
                }],
            )
            .unwrap_err();

        // Step 4
        // Add initial liquidity happy path
        // ------------------------------------------------------------------------------
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(80),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100),
                }],
            )
            .unwrap();

        // check that the owner address on the cw20 quote token contract is decreased by the amount of quote tokens added to the amm
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(4920));

        // check that the amm address on the cw20 quote token contract has the correct amount of quote tokens
        let amm_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, amm_addr.clone())
            .unwrap();
        assert_eq!(amm_balance, Uint128::new(80));

        // check that the lp token contract has the correct lp tokens minted for the owner that added the liquidity
        let lp_balance = lp_token
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(lp_balance, Uint128::new(100));

        // Step 5
        // Top-up liquidity
        // ------------------------------------------------------------------------------

        // Test ContractError::MaxQuoteTokenAmountExceeded {}
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(75),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100),
                }],
            )
            .unwrap_err();

        // increase the spending allowance of the amm_contract on the quote_token_contract
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: Uint128::new(40u128),
            expires: None,
        };
        let _res = router
            .execute_contract(
                owner.clone(),
                quote_token_contract.addr(),
                &allowance_msg,
                &[],
            )
            .unwrap();

        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(50),
            max_quote_token_amount: Uint128::new(40),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(50),
                }],
            )
            .unwrap();

        // check that the owner address on the cw20 quote token contract is decreased by the amount of quote tokens added to the amm
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(4880));

        // check that the amm address on the cw20 quote token contract has the correct amount of quote tokens
        let amm_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, amm_addr.clone())
            .unwrap();
        assert_eq!(amm_balance, Uint128::new(120));

        // check that the lp token contract has the correct lp tokens minted for the owner that added the liquidity
        let lp_balance = lp_token
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(lp_balance, Uint128::new(150));
    }

    #[test]
    fn test_add_liquidity_with_ibc_as_quote() {
        let mut router = mock_app();
        let owner = Addr::unchecked(USER);

        // amm contract instance
        let amm_addr = _instantiate_amm_with_native_as_quote(&mut router, IBC_DENOM_1.into());

        // ContractError::InsufficientFunds {}
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(100),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[
                    Coin {
                        denom: NATIVE_DENOM.into(),
                        amount: Uint128::new(100),
                    },
                    Coin {
                        denom: IBC_DENOM_1.into(),
                        amount: Uint128::new(99),
                    },
                ],
            )
            .unwrap_err();

        // Add liquidity proper and inspect the outputs
        let amount_to_add = Uint128::new(100);
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: amount_to_add,
            max_quote_token_amount: amount_to_add,
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[
                    Coin {
                        denom: NATIVE_DENOM.into(),
                        amount: amount_to_add,
                    },
                    Coin {
                        denom: IBC_DENOM_1.into(),
                        amount: amount_to_add,
                    },
                ],
            )
            .unwrap();

        // Check that the NATIVE_DENOM and IBC_DENOM balance for USER has been reduced
        // by the amount of liquidity added to the pool
        let balance = bank_balance(&mut router, &owner, NATIVE_DENOM.to_string());
        assert_eq!(balance.amount, Uint128::new(SUPPLY) - amount_to_add);

        let balance = bank_balance(&mut router, &owner, IBC_DENOM_1.to_string());
        assert_eq!(balance.amount, Uint128::new(SUPPLY) - amount_to_add);

        // Add liquidity with excess quote tokens sent to the contract
        // and expect the excess to be sent back to the user as change
        let amount_to_add = Uint128::new(100);
        let excess = Uint128::new(50);
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: amount_to_add,
            max_quote_token_amount: amount_to_add + excess,
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[
                    Coin {
                        denom: NATIVE_DENOM.into(),
                        amount: amount_to_add,
                    },
                    Coin {
                        denom: IBC_DENOM_1.into(),
                        amount: amount_to_add + excess,
                    },
                ],
            )
            .unwrap();

        let balance = bank_balance(&mut router, &owner, IBC_DENOM_1.into());
        assert_eq!(
            balance.amount,
            Uint128::new(SUPPLY) - (amount_to_add + amount_to_add)
        );
    }

    #[test]
    fn test_remove_liquidity_with_cw20_as_quote() {
        let mut router = mock_app();
        let owner = Addr::unchecked(USER);

        // cw20 quote token contract
        let quote_token_contract = create_cw20_quote_token(
            &mut router,
            &owner,
            "token".to_string(),
            "CWTOKEN".to_string(),
            Uint128::new(5000),
        );

        // amm contract instance
        let amm_addr =
            _instantiate_amm_with_cw20_as_quote(&mut router, quote_token_contract.addr());

        // make sure that quote_token_contract.addr() != amm_addr
        assert_ne!(quote_token_contract.addr(), amm_addr);

        // Query amm info
        let info = get_amm_contract_info(&mut router, &amm_addr);

        // Setup LP token helper
        let lp_token = Cw20Contract(Addr::unchecked(info.lp_token_address));

        // check quote_token balance for owner
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(5000));

        // increase the spending allowance of the amm_contract on the quote_token_contract
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: Uint128::new(100u128),
            expires: None,
        };
        let _res = router
            .execute_contract(
                owner.clone(),
                quote_token_contract.addr(),
                &allowance_msg,
                &[],
            )
            .unwrap();

        // Add liquidity
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100),
            max_quote_token_amount: Uint128::new(100),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100),
                }],
            )
            .unwrap();

        // check that the owner address on the cw20 quote token contract
        // is decreased by the amount of quote tokens added to the amm
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(4900));

        // check that the amm address on the cw20 quote token contract has the correct amount of quote tokens
        let amm_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, amm_addr.clone())
            .unwrap();
        assert_eq!(amm_balance, Uint128::new(100));

        // check that the lp token contract has the correct lp tokens minted for the owner that added the liquidity
        let lp_balance = lp_token
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(lp_balance, Uint128::new(100));

        // Test All Error Cases
        // ContractError::MsgExpirationError {}
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::RemoveLiquidity {
                    amount: Uint128::new(100),
                    min_base_token_output: Uint128::new(100),
                    min_quote_token_output: Uint128::new(100),
                    expiration: Some(Expiration::AtHeight(0)),
                },
                &[],
            )
            .unwrap_err();

        // ContractError::InsufficientLiquidityError {}
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::RemoveLiquidity {
                    amount: Uint128::new(200),
                    min_base_token_output: Uint128::new(100),
                    min_quote_token_output: Uint128::new(100),
                    expiration: None,
                },
                &[],
            )
            .unwrap_err();

        // ContractError::MinBaseTokenOutputError {}
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::RemoveLiquidity {
                    amount: Uint128::new(100),
                    min_base_token_output: Uint128::new(200),
                    min_quote_token_output: Uint128::new(100),
                    expiration: None,
                },
                &[],
            )
            .unwrap_err();

        // ContractError::MinQuoteTokenOutputError {}
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::RemoveLiquidity {
                    amount: Uint128::new(100),
                    min_base_token_output: Uint128::new(100),
                    min_quote_token_output: Uint128::new(200),
                    expiration: None,
                },
                &[],
            )
            .unwrap_err();

        // Remove some liquidity and ensure balances are updated
        // We need to also grant the amm the right to burn lp tokens of behalf of info.sender
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: Uint128::new(50),
            expires: None,
        };
        router
            .execute_contract(owner.clone(), lp_token.addr(), &allowance_msg, &[])
            .unwrap();

        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::RemoveLiquidity {
                    amount: Uint128::new(50),
                    min_base_token_output: Uint128::new(50),
                    min_quote_token_output: Uint128::new(50),
                    expiration: None,
                },
                &[],
            )
            .unwrap();

        // Check that the owner address on the cw20 quote token contract is increased
        // by the amount of quote tokens removed from the amm
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(4950));

        // check that the amm address on the cw20 quote token contract has the correct amount of quote tokens
        let amm_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, amm_addr.clone())
            .unwrap();
        assert_eq!(amm_balance, Uint128::new(50));

        // check that the lp token contract has the correct lp tokens
        let lp_balance = lp_token
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(lp_balance, Uint128::new(50));

        // Remove all remaining liquidity and ensure balances are updated
        // We need to also grant the amm the right to burn lp tokens of behalf of info.sender
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: Uint128::new(50),
            expires: None,
        };
        router
            .execute_contract(owner.clone(), lp_token.addr(), &allowance_msg, &[])
            .unwrap();

        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::RemoveLiquidity {
                    amount: Uint128::new(50),
                    min_base_token_output: Uint128::new(50),
                    min_quote_token_output: Uint128::new(50),
                    expiration: None,
                },
                &[],
            )
            .unwrap();

        // Check that the owner address on the cw20 quote token contract is increased
        // by the amount of quote tokens removed from the amm
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(5000));

        // check that the amm address on the cw20 quote token contract has the correct amount of quote tokens
        let amm_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, amm_addr.clone())
            .unwrap();
        assert_eq!(amm_balance, Uint128::new(0));

        // check that the lp token contract has the correct lp tokens
        let lp_balance = lp_token
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(lp_balance, Uint128::new(0));
    }

    #[test]
    fn test_remove_liquidity_with_ibc_as_quote() {
        let mut router = mock_app();
        let owner = Addr::unchecked(USER);

        // amm contract instance
        let amm_addr = _instantiate_amm_with_native_as_quote(&mut router, IBC_DENOM_1.into());

        // Query amm info
        let info = get_amm_contract_info(&mut router, &amm_addr);

        // Setup LP token helper
        let lp_token = Cw20Contract(Addr::unchecked(info.lp_token_address));

        // Add liquidity proper
        let amount_to_add = Uint128::new(100);
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: amount_to_add,
            max_quote_token_amount: amount_to_add,
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[
                    Coin {
                        denom: NATIVE_DENOM.into(),
                        amount: amount_to_add,
                    },
                    Coin {
                        denom: IBC_DENOM_1.into(),
                        amount: amount_to_add,
                    },
                ],
            )
            .unwrap();

        // Grant the amm the right to burn lp tokens of behalf of info.sender
        let amount_to_remove = Uint128::new(50);
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: amount_to_remove,
            expires: None,
        };
        router
            .execute_contract(owner.clone(), lp_token.addr(), &allowance_msg, &[])
            .unwrap();

        // Remove some liquidity and inspect the balances
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::RemoveLiquidity {
                    amount: amount_to_remove,
                    min_base_token_output: amount_to_remove,
                    min_quote_token_output: amount_to_remove,
                    expiration: None,
                },
                &[],
            )
            .unwrap();

        let lp_balance = lp_token
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(lp_balance, amount_to_remove);

        let balance = bank_balance(&mut router, &owner, IBC_DENOM_1.into());
        assert_eq!(
            balance.amount,
            Uint128::new(SUPPLY) - amount_to_add + amount_to_remove
        );

        let balance = bank_balance(&mut router, &owner, NATIVE_DENOM.into());
        assert_eq!(
            balance.amount,
            Uint128::new(SUPPLY) - amount_to_add + amount_to_remove
        );
    }

    #[test]
    fn test_swap_with_cw20_as_quote() {
        // Step 1
        // Setup the mock app
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let owner = Addr::unchecked(USER);

        // cw20 quote token contract
        let quote_token_contract = create_cw20_quote_token(
            &mut router,
            &owner,
            "token".to_string(),
            "CWTOKEN".to_string(),
            Uint128::new(200_000),
        );

        // amm contract instance
        let amm_addr =
            _instantiate_amm_with_cw20_as_quote(&mut router, quote_token_contract.addr());

        // make sure that quote_token_contract.addr() != amm_addr
        assert_ne!(quote_token_contract.addr(), amm_addr);

        // Query amm info
        let info = get_amm_contract_info(&mut router, &amm_addr);

        // Setup LP token helper
        let lp_token = Cw20Contract(Addr::unchecked(info.lp_token_address));

        // check quote_token balance for owner
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(200_000));

        // Step 2
        // Add liquidity
        // ------------------------------------------------------------------------------

        // increase the spending allowance of the amm_contract on the quote_token_contract
        // on behalf of owner
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: Uint128::new(100_000u128),
            expires: None,
        };
        let _res = router
            .execute_contract(
                owner.clone(),
                quote_token_contract.addr(),
                &allowance_msg,
                &[],
            )
            .unwrap();

        // Add liquidity proper and ensure balances are updated ===============================>
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100_000),
            max_quote_token_amount: Uint128::new(100_000),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100_000),
                }],
            )
            .unwrap();

        // check that the owner address on the cw20 quote token contract is decreased
        // by the amount of quote tokens added to the amm
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(100_000));

        // check that the amm address on the cw20 quote token contract has the correct amount of quote tokens
        let amm_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, amm_addr.clone())
            .unwrap();
        assert_eq!(amm_balance, Uint128::new(100_000));

        // check that the lp token contract has the correct lp tokens minted for the owner that added the liquidity
        let lp_balance = lp_token
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(lp_balance, Uint128::new(100_000));

        // Step 3
        // Test All Error Cases
        // ------------------------------------------------------------------------------

        // ContractError::MsgExpirationError {}
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::Swap {
                    input_token: TokenSelect::Base,
                    input_amount: Uint128::new(10_000),
                    output_amount: Uint128::new(9063),
                    expiration: Some(Expiration::AtHeight(0)),
                },
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(10_000),
                }],
            )
            .unwrap_err();

        // ContractError::SwapMinError {}
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::Swap {
                    input_token: TokenSelect::Base,
                    input_amount: Uint128::new(10_000),

                    // Expected min_output is 9063 <= calculated_output
                    output_amount: Uint128::new(9064),
                    expiration: None,
                },
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(10_000),
                }],
            )
            .unwrap_err();

        // ContractError::SwapMaxError {}
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::Swap {
                    input_token: TokenSelect::Quote,

                    // Expected max_input of 11144 >= calculated_input
                    input_amount: Uint128::new(11143),
                    output_amount: Uint128::new(10_000),
                    expiration: None,
                },
                &[],
            )
            .unwrap_err();

        // ContractError::InsufficientFunds {}
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::Swap {
                    input_token: TokenSelect::Base,
                    input_amount: Uint128::new(10_000),
                    output_amount: Uint128::new(9063),
                    expiration: None,
                },
                &[],
            )
            .unwrap_err();

        // Step 4
        // Do a swap from base token to quote tokens and verify token balances
        // ------------------------------------------------------------------------------
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::Swap {
                    input_token: TokenSelect::Base,
                    input_amount: Uint128::new(10_000),
                    output_amount: Uint128::new(9063),
                    expiration: None,
                },
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(10_000),
                }],
            )
            .unwrap();

        // check that the amm address on the cw20 quote token contract has the correct amount of quote tokens
        let amm_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, amm_addr.clone())
            .unwrap();
        assert_eq!(amm_balance, Uint128::new(90937));

        // check that the owner address on the cw20 quote token contract is decreased by the amount of quote tokens added to the amm
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(109_063));

        // Step 5
        // Do a reverse swap from quote token to base token
        // ------------------------------------------------------------------------------
        // At this point after the first swap above we now have,
        // quote_reserve = 90937
        // base_reserve = 110_000
        //
        // increase the spending allowance of the amm_contract on the quote_token_contract
        // on behalf of owner

        // Calculate for how much quote tokens allowance do we need in exchange for 10_000 base tokens
        // Where q = Qb / (B - b)
        // q = 90937 * 10000 / (110_000 - 10000)
        // q = 9093.7 + 0.3%
        // q = 9120
        let max_quote_input = Uint128::new(9120u128);
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: max_quote_input,
            expires: None,
        };
        let _res = router
            .execute_contract(
                owner.clone(),
                quote_token_contract.addr(),
                &allowance_msg,
                &[],
            )
            .unwrap();

        // execute swap from quote to base
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::Swap {
                    input_token: TokenSelect::Quote,
                    input_amount: max_quote_input,
                    output_amount: Uint128::new(10_000),
                    expiration: None,
                },
                &[],
            )
            .unwrap();

        // check that the amm address on the cw20 quote token contract has the correct amount of quote tokens
        let amm_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, amm_addr.clone())
            .unwrap();
        assert_eq!(amm_balance, Uint128::new(90937) + max_quote_input);

        // check that the owner address on the cw20 quote token contract is decreased by the amount of quote tokens added to the amm
        let owner_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(owner_balance, Uint128::new(109_063) - max_quote_input);
    }

    #[test]
    fn test_swap_with_ibc_as_quote() {
        // Step 1
        // Setup the mock app
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let owner = Addr::unchecked(USER);

        // amm contract instance
        let amm_addr = _instantiate_amm_with_native_as_quote(&mut router, IBC_DENOM_1.into());

        // Step 2
        // Add liquidity to the amm
        // ------------------------------------------------------------------------------
        let liquidity_added = Uint128::new(100_000);
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: liquidity_added,
            max_quote_token_amount: liquidity_added,
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[
                    Coin {
                        denom: NATIVE_DENOM.into(),
                        amount: liquidity_added,
                    },
                    Coin {
                        denom: IBC_DENOM_1.into(),
                        amount: liquidity_added,
                    },
                ],
            )
            .unwrap();

        // Step 3
        // Do a swap from base token to quote tokens and verify token balances
        // ------------------------------------------------------------------------------
        let quote_output_1 = Uint128::new(9063);
        let base_amount_input = Uint128::new(10_000);
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::Swap {
                    input_token: TokenSelect::Base,
                    input_amount: base_amount_input,
                    output_amount: quote_output_1,
                    expiration: None,
                },
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: base_amount_input,
                }],
            )
            .unwrap();

        // Verify that quote_amount_receive was added to sender IBC_DENOM balance
        let balance = bank_balance(&mut router, &owner, IBC_DENOM_1.into());
        assert_eq!(
            balance.amount,
            Uint128::new(SUPPLY) - liquidity_added + quote_output_1
        );

        // Step 4
        // Do a reverse swap from quote token to base token
        // ------------------------------------------------------------------------------
        // At this point after the first swap above we now have,
        // quote_reserve = 100_000 - 9063 = 90937
        // base_reserve = 100_000 + 10_000 = 110_000
        //
        // Calculate for how much quote tokens input do we need in exchange for 10_000 base tokens
        // Where q = Qb / (B - b)
        // q = 90937 * 10000 / (110_000 - 10000)
        // q = 9093.7 + 0.3%
        // q = 9120
        let max_quote_input_1 = Uint128::new(9120u128);
        let base_output_1 = Uint128::new(10_000);

        // execute swap from quote to base
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::Swap {
                    input_token: TokenSelect::Quote,
                    input_amount: max_quote_input_1,
                    output_amount: base_output_1,
                    expiration: None,
                },
                &[Coin {
                    denom: IBC_DENOM_1.into(),
                    amount: max_quote_input_1,
                }],
            )
            .unwrap();

        // Verify that base_amount_received was added to sender NATIVE_DENOM balance
        let balance = bank_balance(&mut router, &owner, NATIVE_DENOM.into());
        assert_eq!(
            balance.amount,
            Uint128::new(SUPPLY) - liquidity_added - base_amount_input + base_output_1
        );

        // Step 5
        // do a swap from quote to base where we pass in excess quote amount
        // and verify that change is returned to sender
        // ------------------------------------------------------------------------------
        //
        // At this point after the last swap above we now have,
        // quote_reserve = 90937 + 9120 = 100_057
        // base_reserve = 110_000 - 10_000 = 100_000
        //
        // Calculate for how much quote tokens input do we need in exchange for 10_000 base tokens
        // Where q = Qb / (B - b)
        // q = 100_057 * 10_000 / (100_000 - 10_000)
        // q = 11117.4 + 0.3%
        // q = 11150

        let max_quote_input_2 = Uint128::new(11150u128);
        let excess_quote_input = Uint128::new(50);
        let base_output_2 = Uint128::new(10_000);

        // execute swap from quote to base
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::Swap {
                    input_token: TokenSelect::Quote,
                    input_amount: max_quote_input_2 + excess_quote_input,
                    output_amount: base_output_2,
                    expiration: None,
                },
                &[Coin {
                    denom: IBC_DENOM_1.into(),
                    amount: max_quote_input_2 + excess_quote_input,
                }],
            )
            .unwrap();

        // Verify that base_amount_received was added to sender NATIVE_DENOM balance
        let balance = bank_balance(&mut router, &owner, NATIVE_DENOM.into());
        assert_eq!(
            balance.amount,
            Uint128::new(SUPPLY) - liquidity_added - base_amount_input
                + base_output_1
                + base_output_2
        );

        // Verify that change was returned to sender IBC_DENOM balance
        let balance = bank_balance(&mut router, &owner, IBC_DENOM_1.into());
        assert_eq!(
            balance.amount,
            Uint128::new(SUPPLY) - liquidity_added + quote_output_1
                - max_quote_input_1
                - max_quote_input_2
        );
    }

    #[test]
    fn test_swap_and_send_with_cw20_as_quote() {
        // Step 1
        // Setup the mock app
        // ------------------------------------------------------------------------------

        let mut router = mock_app();
        let owner = Addr::unchecked(USER);
        let recipient = Addr::unchecked("recipient");

        // cw20 quote token contract
        let quote_token_contract = create_cw20_quote_token(
            &mut router,
            &owner,
            "token".to_string(),
            "CWTOKEN".to_string(),
            Uint128::new(200_000),
        );

        // amm contract instance
        let amm_addr =
            _instantiate_amm_with_cw20_as_quote(&mut router, quote_token_contract.addr());

        // make sure that quote_token_contract.addr() != amm_addr
        assert_ne!(quote_token_contract.addr(), amm_addr);

        // Step 2
        // Add liquidity to the amm
        // ------------------------------------------------------------------------------

        // increase the spending allowance of the amm_contract on the quote_token_contract
        // on behalf of owner
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: Uint128::new(100_000u128),
            expires: None,
        };
        let _res = router
            .execute_contract(
                owner.clone(),
                quote_token_contract.addr(),
                &allowance_msg,
                &[],
            )
            .unwrap();

        // Add liquidity
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100_000),
            max_quote_token_amount: Uint128::new(100_000),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100_000),
                }],
            )
            .unwrap();

        // Step 3
        // Do a SwapAndSendTo from base token to quote token,
        // where the output goes to recipient
        // ------------------------------------------------------------------------------

        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::SwapAndSendTo {
                    input_token: TokenSelect::Base,
                    input_amount: Uint128::new(10_000),
                    output_amount: Uint128::new(9063),
                    expiration: None,
                    recipient: recipient.clone(),
                },
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(10_000),
                }],
            )
            .unwrap();

        // Verify that recipient got the output tokens
        let recipient_balance = quote_token_contract
            .balance::<_, _, Empty>(&router, recipient.clone())
            .unwrap();
        assert_eq!(recipient_balance, Uint128::new(9063));

        // Step 4
        // Do a SwapAndSendTo from quote token to base token,
        // where the output goes to recipient
        // ------------------------------------------------------------------------------
        // At this point after the first swap above we now have,
        // quote_reserve = 90937
        // base_reserve = 110_000
        //
        // increase the spending allowance of the amm_contract on the quote_token_contract
        // on behalf of owner, as we are going to be inputing a cw20 quote tokens for an
        // exact base output of 10_000 tokens to be sent to the recipient

        // Calculate for how much quote tokens allowance we need in exchange for 10_000 base tokens
        // Where q = Qb / (B - b)
        // q = 90937 * 10000 / (110_000 - 10000)
        // q = 9093.7 + 0.3%
        // q = 9120
        let max_quote_input = Uint128::new(9120u128);
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: amm_addr.to_string(),
            amount: max_quote_input,
            expires: None,
        };
        let _res = router
            .execute_contract(
                owner.clone(),
                quote_token_contract.addr(),
                &allowance_msg,
                &[],
            )
            .unwrap();

        // do swap from quote to base and verify outputs
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::SwapAndSendTo {
                    input_token: TokenSelect::Quote,
                    input_amount: max_quote_input,
                    output_amount: Uint128::new(10_000),
                    expiration: None,
                    recipient: recipient.clone(),
                },
                &[],
            )
            .unwrap();

        // Verify that recipient got the output tokens
        let balance = bank_balance(&mut router, &recipient, NATIVE_DENOM.to_string());
        assert_eq!(balance.amount, Uint128::new(10_000));
    }

    #[test]
    fn test_swap_and_send_with_ibc_as_quote() {
        // Step 1
        // Setup the mock app
        // ------------------------------------------------------------------------------

        let mut router = mock_app();
        let owner = Addr::unchecked(USER);
        let recipient = Addr::unchecked("recipient");

        // amm contract instance
        let amm_addr = _instantiate_amm_with_native_as_quote(&mut router, IBC_DENOM_1.into());

        // Step 2
        // Add liquidity to the amm
        // ------------------------------------------------------------------------------
        let liquidity_added = Uint128::new(100_000);
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: liquidity_added,
            max_quote_token_amount: liquidity_added,
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &add_liquidity_msg,
                &[
                    Coin {
                        denom: NATIVE_DENOM.into(),
                        amount: liquidity_added,
                    },
                    Coin {
                        denom: IBC_DENOM_1.into(),
                        amount: liquidity_added,
                    },
                ],
            )
            .unwrap();

        // Step 3
        // Do a SwapAndSendTo from base token to quote token,
        // where the output goes to recipient
        // ------------------------------------------------------------------------------
        let amount_to_output = Uint128::new(9063);
        router
            .execute_contract(
                owner.clone(),
                amm_addr.clone(),
                &ExecuteMsg::SwapAndSendTo {
                    input_token: TokenSelect::Base,
                    input_amount: Uint128::new(10_000),
                    output_amount: amount_to_output,
                    expiration: None,
                    recipient: recipient.clone(),
                },
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(10_000),
                }],
            )
            .unwrap();

        // Verify that recipient got the output tokens
        let balance = bank_balance(&mut router, &recipient, IBC_DENOM_1.to_string());
        assert_eq!(balance.amount, amount_to_output);
    }

    #[test]
    fn test_pass_through_swap_in_cw20_out_cw20() {
        // Step 1
        // Setup the mock app
        // Create two amm NATIVE_DENOM:CW20 contracts
        // ------------------------------------------------------------------------------

        let mut router = mock_app();
        let owner = Addr::unchecked(USER);

        // cw20 quote token1
        let token1 = create_cw20_quote_token(
            &mut router,
            &owner,
            "tokenone".to_string(),
            "CWTOKENone".to_string(),
            Uint128::new(200_000),
        );

        // cw20 quote token2
        let token2 = create_cw20_quote_token(
            &mut router,
            &owner,
            "tokentwo".to_string(),
            "CWTOKENtwo".to_string(),
            Uint128::new(200_000),
        );

        // amm contract instances
        let native_to_token1_amm = _instantiate_amm_with_cw20_as_quote(&mut router, token1.addr());
        let native_to_token2_amm = _instantiate_amm_with_cw20_as_quote(&mut router, token2.addr());

        // make sure that token1 != token2
        assert_ne!(token1.addr(), token2.addr());
        // make sure that native_to_token1_amm != native_to_token2_amm
        assert_ne!(native_to_token1_amm, native_to_token2_amm);

        // Step 2
        // Add liquidity to both amm pools
        // ------------------------------------------------------------------------------

        // increase the spending allowance of native_to_token1_amm on token1 contract
        // on behalf of owner
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: native_to_token1_amm.to_string(),
            amount: Uint128::new(100_000u128),
            expires: None,
        };
        let _res = router
            .execute_contract(owner.clone(), token1.addr(), &allowance_msg, &[])
            .unwrap();

        // Add liquidity to native_to_token1_amm
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100_000),
            max_quote_token_amount: Uint128::new(100_000),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                native_to_token1_amm.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100_000),
                }],
            )
            .unwrap();

        // increase the spending allowance of native_to_token2_amm on token2 contract
        // on behalf of owner
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: native_to_token2_amm.to_string(),
            amount: Uint128::new(100_000u128),
            expires: None,
        };
        let _res = router
            .execute_contract(owner.clone(), token2.addr(), &allowance_msg, &[])
            .unwrap();

        // Add liquidity to native_to_token2_amm
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: Uint128::new(100_000),
            max_quote_token_amount: Uint128::new(100_000),
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                native_to_token2_amm.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: Uint128::new(100_000),
                }],
            )
            .unwrap();

        // Given the current state of the the amm_s
        // When quote_input_amount  to native_to_token1_amm = 10_000
        // Calculate min_quote_output_amount from native_to_token2_amm when doing a PassThroughSwap
        //
        // To output the intermediate token b from native_to_token1_amm , where B = 100_000 and Q = 100_000 and q = 10_000
        // b = Bq / (Q + q)
        // b = 100_000 * 10_000 / (100_000 + 10_000)
        // b = 9090.9 - fees
        // b = 9090.9 - (0.3% of 9090.9)
        // b = 9090.9 - 27.2727 = 9063
        //
        // Now b becomes the base input to native_to_token2_amm
        // To calculate min_quote_output_amount q, where b = 9063, B = 100_000 and Q = 100_000
        // q = Qb / (B + b)
        // q = 100_000 * 9063 / (100_000 + 9063)
        // q = 8309.87594326 - fees
        // q = 8309.87594326 - (0.3% of 8309.87594326)
        // q = 8309.87594326 - 24.9296278298
        // q = 8285

        // Step 3
        // Test error cases
        // ------------------------------------------------------------------------------
        // ContractError::MsgExpirationError {}
        let min_quote_output_amount = Uint128::new(8285);
        router
            .execute_contract(
                owner.clone(),
                native_to_token1_amm.clone(),
                &ExecuteMsg::PassThroughSwap {
                    quote_input_amount: Uint128::new(10_000),
                    output_amm_address: native_to_token2_amm.clone(),
                    min_quote_output_amount,
                    expiration: Some(Expiration::AtHeight(0)),
                },
                &[],
            )
            .unwrap_err();

        // Step 4
        // do a pass through swap from native_to_token1_amm to native_to_token2_amm
        // ------------------------------------------------------------------------------
        // increase the spending allowance of native_to_token1_amm on token1 contract
        // on behalf of owner
        let token1_input = Uint128::new(10_000);
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: native_to_token1_amm.to_string(),
            amount: token1_input,
            expires: None,
        };
        let _res = router
            .execute_contract(owner.clone(), token1.addr(), &allowance_msg, &[])
            .unwrap();

        // Do pass through swap
        router
            .execute_contract(
                owner.clone(),
                native_to_token1_amm,
                &ExecuteMsg::PassThroughSwap {
                    quote_input_amount: Uint128::new(10_000),
                    output_amm_address: native_to_token2_amm.clone(),
                    min_quote_output_amount,
                    expiration: None,
                },
                &[],
            )
            .unwrap();

        // verify that min_quote_output_amount went to owner's token2 cw20 address
        let owner_token2_balance = token2
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(
            owner_token2_balance,
            Uint128::new(100_000) + min_quote_output_amount
        );
    }

    #[test]
    fn test_pass_through_swap_in_cw20_out_ibc() {
        // Step 1
        // Setup the mock app
        // Create two amm contracts, NATIVE_DENOM:CW20 and NATIVE_DENOM:IBC_DENOM
        // ------------------------------------------------------------------------------

        let mut router = mock_app();
        let owner = Addr::unchecked(USER);

        // cw20 quote token1
        let cw20_quote_token = create_cw20_quote_token(
            &mut router,
            &owner,
            "tokenone".to_string(),
            "CWTOKENone".to_string(),
            Uint128::new(200_000),
        );

        // amm contract instances
        let native_to_cw20_amm =
            _instantiate_amm_with_cw20_as_quote(&mut router, cw20_quote_token.addr());
        let native_to_ibc_amm =
            _instantiate_amm_with_native_as_quote(&mut router, IBC_DENOM_1.into());

        // make sure that native_to_cw20_amm != native_to_ibc_amm
        assert_ne!(native_to_cw20_amm, native_to_ibc_amm);

        // Step 2
        // Add liquidity to both amm pools
        // ------------------------------------------------------------------------------
        // increase the spending allowance of native_to_cw20_amm on cw20_quote_token contract
        // on behalf of owner
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: native_to_cw20_amm.to_string(),
            amount: Uint128::new(100_000u128),
            expires: None,
        };
        let _res = router
            .execute_contract(owner.clone(), cw20_quote_token.addr(), &allowance_msg, &[])
            .unwrap();

        // Add liquidity to native_to_cw20_amm
        let liquidity_added = Uint128::new(100_000);
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: liquidity_added,
            max_quote_token_amount: liquidity_added,
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                native_to_cw20_amm.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: liquidity_added,
                }],
            )
            .unwrap();

        // Add liquidity to native_to_ibc_amm
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: liquidity_added,
            max_quote_token_amount: liquidity_added,
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                native_to_ibc_amm.clone(),
                &add_liquidity_msg,
                &[
                    Coin {
                        denom: NATIVE_DENOM.into(),
                        amount: liquidity_added,
                    },
                    Coin {
                        denom: IBC_DENOM_1.into(),
                        amount: liquidity_added,
                    },
                ],
            )
            .unwrap();

        // Step 3
        // do a pass through swap from native_to_cw20_amm to native_to_ibc_amm
        // ------------------------------------------------------------------------------
        // increase the spending allowance of native_to_cw20_amm on cw20_quote_token contract
        // on behalf of owner
        let quote_input = Uint128::new(10_000);
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: native_to_cw20_amm.to_string(),
            amount: quote_input,
            expires: None,
        };
        let _res = router
            .execute_contract(owner.clone(), cw20_quote_token.addr(), &allowance_msg, &[])
            .unwrap();

        // Do pass through swap
        let min_quote_output_amount = Uint128::new(8285);
        router
            .execute_contract(
                owner.clone(),
                native_to_cw20_amm,
                &ExecuteMsg::PassThroughSwap {
                    quote_input_amount: Uint128::new(10_000),
                    output_amm_address: native_to_ibc_amm.clone(),
                    min_quote_output_amount,
                    expiration: None,
                },
                &[],
            )
            .unwrap();

        // verify that min_quote_output_amount was added to owners IBC_DENOM balance
        let balance = bank_balance(&mut router, &owner, IBC_DENOM_1.to_string());
        assert_eq!(
            balance.amount,
            Uint128::new(SUPPLY) - liquidity_added + min_quote_output_amount
        );
    }

    #[test]
    fn test_pass_through_swap_in_ibc_out_cw20() {
        // Step 1
        // Setup the mock app
        // Create two amm contracts, NATIVE_DENOM:IBC_DENOM and NATIVE_DENOM:CW20
        // ------------------------------------------------------------------------------

        let mut router = mock_app();
        let owner = Addr::unchecked(USER);

        // cw20 quote token1
        let cw20_quote_token = create_cw20_quote_token(
            &mut router,
            &owner,
            "tokenone".to_string(),
            "CWTOKENone".to_string(),
            Uint128::new(200_000),
        );

        // amm contract instances
        let native_to_cw20_amm =
            _instantiate_amm_with_cw20_as_quote(&mut router, cw20_quote_token.addr());
        let native_to_ibc_amm =
            _instantiate_amm_with_native_as_quote(&mut router, IBC_DENOM_1.into());

        // make sure that native_to_cw20_amm != native_to_ibc_amm
        assert_ne!(native_to_cw20_amm, native_to_ibc_amm);

        // Step 2
        // Add liquidity to both amm pools
        // ------------------------------------------------------------------------------

        // increase the spending allowance of native_to_cw20_amm on cw20_quote_token contract
        // on behalf of owner
        let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: native_to_cw20_amm.to_string(),
            amount: Uint128::new(100_000u128),
            expires: None,
        };
        let _res = router
            .execute_contract(owner.clone(), cw20_quote_token.addr(), &allowance_msg, &[])
            .unwrap();

        // Add liquidity to native_to_cw20_amm
        let liquidity_added = Uint128::new(100_000);
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: liquidity_added,
            max_quote_token_amount: liquidity_added,
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                native_to_cw20_amm.clone(),
                &add_liquidity_msg,
                &[Coin {
                    denom: NATIVE_DENOM.into(),
                    amount: liquidity_added,
                }],
            )
            .unwrap();

        // Add liquidity to native_to_ibc_amm
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: liquidity_added,
            max_quote_token_amount: liquidity_added,
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                native_to_ibc_amm.clone(),
                &add_liquidity_msg,
                &[
                    Coin {
                        denom: NATIVE_DENOM.into(),
                        amount: liquidity_added,
                    },
                    Coin {
                        denom: IBC_DENOM_1.into(),
                        amount: liquidity_added,
                    },
                ],
            )
            .unwrap();

        // Step 3
        // do a pass through swap from native_to_ibc_amm to native_to_cw20_amm
        // ------------------------------------------------------------------------------
        // Do pass through swap
        let min_quote_output_amount = Uint128::new(8285);
        router
            .execute_contract(
                owner.clone(),
                native_to_ibc_amm,
                &ExecuteMsg::PassThroughSwap {
                    quote_input_amount: Uint128::new(10_000),
                    output_amm_address: native_to_cw20_amm.clone(),
                    min_quote_output_amount,
                    expiration: None,
                },
                &[Coin {
                    denom: IBC_DENOM_1.into(),
                    amount: Uint128::new(10_000),
                }],
            )
            .unwrap();

        // verify that min_quote_output_amount went to owner's address on cw20_quote_token
        let owner_cw20_quote_token = cw20_quote_token
            .balance::<_, _, Empty>(&router, owner.clone())
            .unwrap();
        assert_eq!(
            owner_cw20_quote_token,
            Uint128::new(100_000) + min_quote_output_amount
        );
    }

    #[test]
    fn test_pass_through_swap_in_ibc_out_ibc() {
        // Step 1
        // Setup the mock app
        // Create two amm contracts, NATIVE_DENOM:IBC_DENOM and NATIVE_DENOM:IBC_DENOM
        // ------------------------------------------------------------------------------
        let mut router = mock_app();
        let owner = Addr::unchecked(USER);

        // amm contract instances
        let native_to_ibc_denom_1_amm =
            _instantiate_amm_with_native_as_quote(&mut router, IBC_DENOM_1.into());
        let native_to_ibc_denom_2_amm =
            _instantiate_amm_with_native_as_quote(&mut router, IBC_DENOM_2.into());

        // make sure that native_to_cw20_amm != native_to_ibc_amm
        assert_ne!(native_to_ibc_denom_1_amm, native_to_ibc_denom_2_amm);

        // Step 2
        // Add liquidity to both amm pools
        // ------------------------------------------------------------------------------
        let liquidity_added = Uint128::new(100_000);

        // Add liquidity to native_to_ibc_denom_1_amm
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: liquidity_added,
            max_quote_token_amount: liquidity_added,
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                native_to_ibc_denom_1_amm.clone(),
                &add_liquidity_msg,
                &[
                    Coin {
                        denom: NATIVE_DENOM.into(),
                        amount: liquidity_added,
                    },
                    Coin {
                        denom: IBC_DENOM_1.into(),
                        amount: liquidity_added,
                    },
                ],
            )
            .unwrap();

        // Add liquidity to native_to_ibc_denom_2_amm
        let add_liquidity_msg = ExecuteMsg::AddLiquidity {
            base_token_amount: liquidity_added,
            max_quote_token_amount: liquidity_added,
            expiration: None,
        };
        router
            .execute_contract(
                owner.clone(),
                native_to_ibc_denom_2_amm.clone(),
                &add_liquidity_msg,
                &[
                    Coin {
                        denom: NATIVE_DENOM.into(),
                        amount: liquidity_added,
                    },
                    Coin {
                        denom: IBC_DENOM_2.into(),
                        amount: liquidity_added,
                    },
                ],
            )
            .unwrap();

        // Do pass through swap
        let min_quote_output_amount = Uint128::new(8285);
        router
            .execute_contract(
                owner.clone(),
                native_to_ibc_denom_1_amm,
                &ExecuteMsg::PassThroughSwap {
                    quote_input_amount: Uint128::new(10_000),
                    output_amm_address: native_to_ibc_denom_2_amm.clone(),
                    min_quote_output_amount,
                    expiration: None,
                },
                &[Coin {
                    denom: IBC_DENOM_1.into(),
                    amount: Uint128::new(10_000),
                }],
            )
            .unwrap();

        // verify that min_quote_output_amount was added to owners IBC_DENOM balance
        let balance = bank_balance(&mut router, &owner, IBC_DENOM_2.to_string());
        assert_eq!(
            balance.amount,
            Uint128::new(SUPPLY) - liquidity_added + min_quote_output_amount
        );
    }
}
