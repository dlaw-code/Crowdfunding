#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cosmwasm_std::Timestamp;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, GetProjectDetailsResponse, GetContributorsResponse};
use crate::state::{State, STATE, CONTRIBUTORS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:crowdfunding";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Clone `msg.name` before moving it into the `State` struct
    let name = msg.name.clone();
    let state = State {
        name: name, // Use the cloned value
        description: msg.description,
        funding_goal: msg.funding_goal,
        current_funding: 0,
        deadline: msg.deadline,
        owner: info.sender.clone(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    // Now you can still use `msg.name` here
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("project_name", msg.name))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Contribute {} => execute::contribute(deps, env, info),
        ExecuteMsg::Withdraw { amount } => execute::withdraw(deps, env, info, amount),
        ExecuteMsg::Refund {} => execute::refund(deps, env, info),
    }
}

pub mod execute {
    use super::*;

    // Inside the `execute::contribute` function
pub fn contribute(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // First, update the state
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if env.block.time > Timestamp::from_nanos(state.deadline) {
            return Err(ContractError::DeadlineExceeded {});
        }

        let contribution = info.funds.iter()
            .find(|coin| coin.denom == "earth")
            .map(|coin| coin.amount.u128())
            .unwrap_or(0);

        state.current_funding += contribution;
        Ok(state)
    })?;

    // Then, update the contributors
    CONTRIBUTORS.update(deps.storage, &info.sender, |amount| -> StdResult<_> {
        let contribution = info.funds.iter()
            .find(|coin| coin.denom == "earth")
            .map(|coin| coin.amount.u128())
            .unwrap_or(0);

        Ok(amount.unwrap_or_default() + contribution)
    })?;

    Ok(Response::new().add_attribute("action", "contribute"))
}


// Inside the `execute::withdraw` function
pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo, amount: u128) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }

        if env.block.time < Timestamp::from_nanos(state.deadline) || state.current_funding < state.funding_goal {
            return Err(ContractError::WithdrawalNotAllowed {});
        }

        state.current_funding -= amount;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("action", "withdraw"))
}



    // Inside the `execute::refund` function
pub fn refund(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // First, check and update the state
    STATE.update(deps.storage, |state| -> Result<_, ContractError> {
    if env.block.time < Timestamp::from_nanos(state.deadline) {
        return Err(ContractError::RefundNotAllowed {});
    }

    if state.current_funding >= state.funding_goal {
        return Err(ContractError::RefundNotAllowed {});
    }

    Ok(state)
})?;

    // Then, load the contributor's amount and update the state again
    let amount = CONTRIBUTORS.load(deps.storage, &info.sender)?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.current_funding -= amount;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("action", "refund"))
}

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProjectDetails {} => to_json_binary(&query::project_details(deps)?),
        QueryMsg::GetContributors {} => to_json_binary(&query::contributors(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn project_details(deps: Deps) -> StdResult<GetProjectDetailsResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetProjectDetailsResponse {
            name: state.name,
            description: state.description,
            funding_goal: state.funding_goal,
            current_funding: state.current_funding,
            deadline: state.deadline,
            owner: state.owner,
        })
    }

    pub fn contributors(deps: Deps) -> StdResult<GetContributorsResponse> {
        let contributors = CONTRIBUTORS
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .map(|item| {
                let (addr, amount) = item?;
                Ok((addr, amount))
            })
            .collect::<StdResult<Vec<_>>>()?;
        Ok(GetContributorsResponse { contributors })
    }
}
