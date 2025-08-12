#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json, Addr, Uint128};
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
            funding_goal: 1000,
            deadline: 1000000,
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_contribute() {
        let mut app = mock_app();
        let code_id = store_code(&mut app);

        let msg = InstantiateMsg {
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            funding_goal: 1000,
            deadline: 1000000,
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        let contract_addr = app.instantiate_contract(code_id, Addr::unchecked("creator"), &msg, &[], "test", None).unwrap();

        let contribute_msg = ExecuteMsg::Contribute {};
        let contributor_info = mock_info("contributor", &coins(500, "earth"));
        let _res = app.execute_contract(contributor_info.sender, contract_addr, &contribute_msg, &coins(500, "earth")).unwrap();

        let query_msg = QueryMsg::GetProjectDetails {};
        let res = app.wrap().query_wasm_smart(contract_addr, &query_msg).unwrap();
        let project_details: GetProjectDetailsResponse = from_json(res).unwrap();
        assert_eq!(500, project_details.current_funding);
    }

    #[test]
    fn test_withdraw() {
        let mut app = mock_app();
        let code_id = store_code(&mut app);

        let msg = InstantiateMsg {
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            funding_goal: 1000,
            deadline: 1000000,
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        let contract_addr = app.instantiate_contract(code_id, Addr::unchecked("creator"), &msg, &[], "test", None).unwrap();

        let contribute_msg = ExecuteMsg::Contribute {};
        let contributor_info = mock_info("contributor", &coins(1000, "earth"));
        let _res = app.execute_contract(contributor_info.sender, contract_addr.clone(), &contribute_msg, &coins(1000, "earth")).unwrap();

        let withdraw_msg = ExecuteMsg::Withdraw { amount: 1000 };
        let owner_info = mock_info("creator", &[]);
        let _res = app.execute_contract(owner_info.sender, contract_addr, &withdraw_msg, &[]).unwrap();

        let query_msg = QueryMsg::GetProjectDetails {};
        let res = app.wrap().query_wasm_smart(contract_addr, &query_msg).unwrap();
        let project_details: GetProjectDetailsResponse = from_json(res).unwrap();
        assert_eq!(0, project_details.current_funding);
    }

    #[test]
    fn test_refund() {
        let mut app = mock_app();
        let code_id = store_code(&mut app);

        let msg = InstantiateMsg {
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            funding_goal: 1000,
            deadline: 1,
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        let contract_addr = app.instantiate_contract(code_id, Addr::unchecked("creator"), &msg, &[], "test", None).unwrap();

        let contribute_msg = ExecuteMsg::Contribute {};
        let contributor_info = mock_info("contributor", &coins(500, "earth"));
        let _res = app.execute_contract(contributor_info.sender, contract_addr.clone(), &contribute_msg, &coins(500, "earth")).unwrap();

        // Simulate passing the deadline
        app.update_block(|block| {
            block.time = 2;
        });

        let refund_msg = ExecuteMsg::Refund {};
        let contributor_info = mock_info("contributor", &[]);
        let _res = app.execute_contract(contributor_info.sender, contract_addr, &refund_msg, &[]).unwrap();

        let query_msg = QueryMsg::GetProjectDetails {};
        let res = app.wrap().query_wasm_smart(contract_addr, &query_msg).unwrap();
        let project_details: GetProjectDetailsResponse = from_json(res).unwrap();
        assert_eq!(0, project_details.current_funding);
    }
}
