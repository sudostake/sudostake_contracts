#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
    use cosmwasm_std::{
        to_binary, Addr, Attribute, Empty, MemoryStorage, OwnedDeps, Reply, ReplyOn, SubMsg,
        SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
    };
    use cw20::{Denom, MinterResponse};

    use crate::contract::{
        exact_input_variable_output, exact_output_variable_input, get_lp_token_amount_to_mint,
        get_required_quote_token_amount, instantiate, reply,
    };
    use crate::msg::InstantiateMsg;
    use crate::state::{SwapPrice, TokenAmount, LP_TOKEN};
    use crate::ContractError;

    struct InstantiationResponse {
        deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier<Empty>, Empty>,
    }

    #[test]
    fn init_error_invalid_native_denom() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let caller = String::from("cosmos2contract");

        let msg = InstantiateMsg {
            native_denom: Denom::Cw20(Addr::unchecked("non_native")),
            base_denom: Denom::Cw20(Addr::unchecked("non_native")),
            quote_denom: Denom::Cw20(Addr::unchecked("quote")),
            lp_token_code_id: 1234u64,
        };

        // Inspect response
        let info = mock_info(&caller, &[]);
        let _err = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap_err();
        match _err {
            ContractError::InvalidNativeDenom {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn init_error_native_token_not_provided_in_pair() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let caller = String::from("cosmos2contract");

        let msg = InstantiateMsg {
            native_denom: Denom::Native(String::from("native")),
            base_denom: Denom::Native(String::from("native_but_wrong_value")),
            quote_denom: Denom::Native(String::from("ibc/token")),
            lp_token_code_id: 1234u64,
        };

        // Inspect response
        let info = mock_info(&caller, &[]);
        let _err = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap_err();
        match _err {
            ContractError::NativeTokenNotProvidedInPair {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn init_error_invalid_quote_denom() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let caller = String::from("cosmos2contract");

        let msg = InstantiateMsg {
            native_denom: Denom::Native(String::from("native")),
            base_denom: Denom::Native(String::from("native")),
            quote_denom: Denom::Native(String::from("native")),
            lp_token_code_id: 1234u64,
        };

        // Inspect response
        let info = mock_info(&caller, &[]);
        let _err = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap_err();
        match _err {
            ContractError::InvalidQuoteDenom {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    // This function instantiate the contract and returns reusable components
    fn proper_initialization() -> InstantiationResponse {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let caller = String::from("cosmos2contract");

        let msg = InstantiateMsg {
            native_denom: Denom::Native(String::from("native")),
            base_denom: Denom::Native(String::from("native")),
            quote_denom: Denom::Cw20(Addr::unchecked("quote")),
            lp_token_code_id: 1234u64,
        };

        // Inspect response
        let info = mock_info(&caller, &[]);
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

        assert_eq!(
            _res.attributes[0],
            Attribute {
                key: String::from("method"),
                value: String::from("instantiate")
            }
        );
        assert_eq!(
            _res.messages[0],
            SubMsg {
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Success,
                msg: WasmMsg::Instantiate {
                    msg: to_binary(&lp_token_info(caller.clone())).unwrap(),
                    code_id: msg.lp_token_code_id,
                    funds: vec![],
                    label: format!("hhslp_{:?}_{:?}", msg.base_denom, msg.quote_denom),
                    admin: None,
                }
                .into()
            }
        );

        // return reusable data
        InstantiationResponse { deps }
    }

    fn lp_token_info(minter: String) -> cw20_base::msg::InstantiateMsg {
        cw20_base::msg::InstantiateMsg {
            name: "HuahuaSwap LP Token".into(),
            symbol: "hhslpt".into(),
            decimals: 6,
            initial_balances: vec![],
            mint: Some(MinterResponse { minter, cap: None }),
            marketing: None,
        }
    }

    #[test]
    fn test_correct_instantiation_reply() {
        let mut _instance = proper_initialization();

        // Test the submsg after cw_20_token is stored
        let contract_addr = String::from("pair0000");
        let reply_msg = Reply {
            id: 0,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                // ? derive this line from contract_addr
                data: Some(vec![10, 8, 112, 97, 105, 114, 48, 48, 48, 48].into()),
            }),
        };

        // execute reply message
        let _res = reply(_instance.deps.as_mut(), mock_env(), reply_msg).unwrap();
        assert_eq!(
            _res.attributes[0],
            Attribute {
                key: String::from("token_contract_addr"),
                value: contract_addr.clone()
            }
        );

        // query the contract state to see if the lp contract address was saved
        let lp_token_address = LP_TOKEN.load(&_instance.deps.storage).unwrap();
        assert_eq!(lp_token_address, Addr::unchecked(contract_addr));
    }

    // we cannot test the execute methods using the standard execute method calls
    // because we are calling into another contract
    // in that case, we just test the standard functions used in the contract
    #[test]
    fn test_get_lp_token_amount_to_mint() {
        let liquidity =
            get_lp_token_amount_to_mint(Uint128::new(100), Uint128::zero(), Uint128::zero())
                .unwrap();
        assert_eq!(liquidity, Uint128::new(100));

        let liquidity =
            get_lp_token_amount_to_mint(Uint128::new(100), Uint128::new(50), Uint128::new(25))
                .unwrap();
        assert_eq!(liquidity, Uint128::new(200));
    }

    #[test]
    fn test_get_required_quote_token_amount() {
        let liquidity = get_required_quote_token_amount(
            Uint128::new(100),
            Uint128::new(50),
            Uint128::zero(),
            Uint128::zero(),
            Uint128::zero(),
        )
        .unwrap();
        assert_eq!(liquidity, Uint128::new(50));

        let liquidity = get_required_quote_token_amount(
            Uint128::new(200),
            Uint128::new(50),
            Uint128::new(100),
            Uint128::new(100),
            Uint128::new(100),
        )
        .unwrap();
        assert_eq!(liquidity, Uint128::new(200));
    }

    // Where q = Qb / (B + b)
    #[test]
    fn test_exact_input_variable_output() {
        let exact_input_amount = Uint128::new(10_000);
        let base_reserve = Uint128::new(100_000);
        let quote_reserve = Uint128::new(100_000);
        let base_denom = Denom::Native("base".to_string());
        let quote_denom = Denom::Cw20(Addr::unchecked("quote"));

        // q = 100000 * 10000 / (100000 + 10000)
        // q = 9090 - 0.3%
        // q = 9063

        // Expect an error because calculated_output < min_output_amount
        exact_input_variable_output(
            exact_input_amount,
            Uint128::new(9064),
            base_reserve,
            quote_reserve,
            base_denom.clone(),
            quote_denom.clone(),
        )
        .unwrap_err();

        // We call the function again with min_output_amount <= calculated_output
        let res = exact_input_variable_output(
            exact_input_amount,
            Uint128::new(9063),
            base_reserve,
            quote_reserve,
            base_denom.clone(),
            quote_denom.clone(),
        )
        .unwrap();

        assert_eq!(
            res,
            SwapPrice {
                input: TokenAmount {
                    amount: Uint128::new(10000),
                    denom: base_denom
                },
                output: TokenAmount {
                    amount: Uint128::new(9063),
                    denom: quote_denom
                }
            }
        );
    }

    // Where q = Qb / (B - b)
    #[test]
    fn test_exact_output_variable_input() {
        let exact_output_amount = Uint128::new(10000);
        let base_reserve = Uint128::new(100_000);
        let quote_reserve = Uint128::new(100_000);
        let base_denom = Denom::Native("base".to_string());
        let quote_denom = Denom::Cw20(Addr::unchecked("quote"));

        // q = 100000 * 10000 / (100000 - 10000)
        // q = 11111 + 0.3%
        // q = 11144

        // Expect an error because calculated_input > max_input_amount is
        exact_output_variable_input(
            exact_output_amount,
            Uint128::new(11142),
            base_reserve,
            quote_reserve,
            base_denom.clone(),
            quote_denom.clone(),
        )
        .unwrap_err();

        // We call the function again with max_input_amount >= calculated_input
        let res = exact_output_variable_input(
            exact_output_amount,
            Uint128::new(11144),
            base_reserve,
            quote_reserve,
            base_denom.clone(),
            quote_denom.clone(),
        )
        .unwrap();

        assert_eq!(
            res,
            SwapPrice {
                output: TokenAmount {
                    amount: Uint128::new(10000),
                    denom: base_denom
                },
                input: TokenAmount {
                    amount: Uint128::new(11144),
                    denom: quote_denom
                }
            }
        );
    }
}
