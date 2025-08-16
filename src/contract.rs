#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, StdError, Uint128
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetContributorsResponse, GetProjectDetailsResponse, InstantiateMsg, QueryMsg,
};
use crate::state::{State, CONTRIBUTORS, STATE};

const CONTRACT_NAME: &str = "crates.io:crowdfunding";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        name: msg.name.clone(),
        description: msg.description,
        funding_goal: msg.funding_goal,
        current_funding: Uint128::zero(), // Explicitly initialize to zero
        deadline: msg.deadline,
        owner: info.sender.clone(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

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
        ExecuteMsg::ExtendDeadline { new_deadline } => execute::extend_deadline(deps, env, info, new_deadline),
    }
}
pub mod execute {
    use super::*;

    pub fn contribute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        // Explicitly validate and extract the contribution
        let contribution = match info.funds.as_slice() {
            [coin] if coin.denom == "earth" => coin.amount,
            _ => return Err(ContractError::InvalidFunds {}),
        };

        // Validate positive contribution
        if contribution.is_zero() {
            return Err(ContractError::EmptyContribution {});
        }

        // Update project funding - ONLY ADDITION
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            // Check deadline
            if env.block.time.seconds() > state.deadline {
                return Err(ContractError::DeadlineExceeded {});
            }

            // Handle the Result from checked_add properly
            state.current_funding = state.current_funding.checked_add(contribution)
                .map_err(|_| ContractError::Overflow {})?;
            Ok(state)
        })?;

        // Update contributor balance - ONLY ADDITION
        CONTRIBUTORS.update(deps.storage, &info.sender, |balance| {
            let current = balance.unwrap_or(Uint128::zero());
            current.checked_add(contribution)
                .map_err(|e| StdError::overflow(e))
        })?;

        Ok(Response::new()
            .add_attribute("action", "contribute")
            .add_attribute("sender", info.sender)
            .add_attribute("amount", contribution))
    }
    pub fn withdraw(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if info.sender != state.owner {
                return Err(ContractError::Unauthorized {});
            }

            if env.block.time.seconds() < state.deadline 
                || state.current_funding < state.funding_goal
            {
                return Err(ContractError::WithdrawalNotAllowed {});
            }

            state.current_funding = state.current_funding.checked_sub(amount)
                .map_err(|_| ContractError::Overflow {})?;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "withdraw"))
    }

    pub fn refund(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |state| -> Result<_, ContractError> {
            if env.block.time.seconds() < state.deadline {
                return Err(ContractError::RefundNotAllowed {});
            }

            if state.current_funding >= state.funding_goal {
                return Err(ContractError::RefundNotAllowed {});
            }

            Ok(state)
        })?;

        let amount = CONTRIBUTORS.load(deps.storage, &info.sender)?;
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.current_funding = state.current_funding.checked_sub(amount)
                .map_err(|_| ContractError::Overflow {})?;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "refund"))
    }

    pub fn extend_deadline(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        new_deadline: u64,
    ) -> Result<Response, ContractError> {
        let mut state = STATE.load(deps.storage)?;
        
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        
        if new_deadline <= env.block.time.seconds() {
            return Err(ContractError::InvalidDeadline {});
        }
        
        state.deadline = new_deadline;
        STATE.save(deps.storage, &state)?;
        
        Ok(Response::new()
            .add_attribute("action", "extend_deadline")
            .add_attribute("new_deadline", new_deadline.to_string()))
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