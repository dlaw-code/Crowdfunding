#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{
        ExecuteMsg, GetContributorsResponse, GetProjectDetailsResponse, InstantiateMsg, QueryMsg
    };
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{coins, Addr, Coin, Timestamp, Uint128};
    use cw_multi_test::{App, ContractWrapper, Executor};

    fn mock_app() -> App {
        App::default()
    }

    fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            funding_goal: Uint128::from(1000u128),
            deadline: 1000000,
        };
        let creator = Addr::unchecked("creator");
        let info = message_info(&creator, &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

   #[test]
fn test_contribute() {
    let mut app = mock_app();
    let code_id = store_code(&mut app);

    // Set initial block time
    app.update_block(|block| {
        block.time = Timestamp::from_seconds(0);
        block.height += 1;
    });

    // Initialize contract with deadline far in the future
    let msg = InstantiateMsg {
        name: "Test Project".to_string(),
        description: "A test project".to_string(),
        funding_goal: Uint128::new(1000),
        deadline: 1000000, // Far future deadline
    };
    let creator = Addr::unchecked("creator");
    let contract_addr = app.instantiate_contract(code_id, creator, &msg, &[], "test", None)
        .expect("Contract instantiation failed");

    // Create contributor and mint them some tokens
    let contributor = Addr::unchecked("contributor1");
    app.init_modules(|router, _, storage| {
        router.bank.init_balance(
            storage,
            &contributor,
            vec![Coin {
                denom: "earth".to_string(),
                amount: Uint128::new(1000),
            }],
        )
        .unwrap();
    });

    // Make contribution
    let res = app.execute_contract(
        contributor.clone(),
        contract_addr.clone(),
        &ExecuteMsg::Contribute {},
        &[Coin {
            denom: "earth".to_string(),
            amount: Uint128::new(500),
        }],
    );

    // Verify no errors
    res.unwrap();

    // Verify updated state
    let updated_state: GetProjectDetailsResponse = app.wrap()
        .query_wasm_smart(&contract_addr, &QueryMsg::GetProjectDetails {})
        .unwrap();
    assert_eq!(updated_state.current_funding, Uint128::new(500));

    // Verify contributor balance
    let contributors: GetContributorsResponse = app.wrap()
        .query_wasm_smart(&contract_addr, &QueryMsg::GetContributors {})
        .unwrap();
    assert_eq!(contributors.contributors, vec![(contributor, Uint128::new(500))]);
}
    #[test]
    fn test_withdraw() {
        let mut app = mock_app();
        let code_id = store_code(&mut app);

        let msg = InstantiateMsg {
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            funding_goal: Uint128::from(1000u128),
            deadline: 1000000,
        };
        let creator = Addr::unchecked("creator");
        let contract_addr = app
            .instantiate_contract(code_id, creator.clone(), &msg, &[], "test", None)
            .unwrap();

        // First contribute
        let contributor = Addr::unchecked("contributor");
        let _ = app.execute_contract(
            contributor,
            contract_addr.clone(),
            &ExecuteMsg::Contribute {},
            &coins(1000, "earth"),
        );

        // Then withdraw
        let _ = app.execute_contract(
            creator,
            contract_addr.clone(),
            &ExecuteMsg::Withdraw { amount: Uint128::from(1000u128) },
            &[],
        );

        let details: GetProjectDetailsResponse = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::GetProjectDetails {})
            .unwrap();
        assert_eq!(Uint128::zero(), details.current_funding);
    }

    #[test]
    fn test_refund() {
        let mut app = mock_app();
        let code_id = store_code(&mut app);

        let msg = InstantiateMsg {
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            funding_goal: Uint128::from(1000u128),
            deadline: 1, // Set deadline in past
        };
        let creator = Addr::unchecked("creator");
        let contract_addr = app
            .instantiate_contract(code_id, creator, &msg, &[], "test", None)
            .unwrap();

        // Contribute first
        let contributor = Addr::unchecked("contributor");
        let _ = app.execute_contract(
            contributor.clone(),
            contract_addr.clone(),
            &ExecuteMsg::Contribute {},
            &coins(500, "earth"),
        );

        // Advance time past deadline
        app.update_block(|block| {
            block.time = Timestamp::from_seconds(2);
        });

        // Get refund
        let _ = app.execute_contract(
            contributor,
            contract_addr.clone(),
            &ExecuteMsg::Refund {},
            &[],
        );

        let details: GetProjectDetailsResponse = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::GetProjectDetails {})
            .unwrap();
        assert_eq!(Uint128::zero(), details.current_funding);
    }
}