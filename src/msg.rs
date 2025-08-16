use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};  // Added Addr import

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub description: String,
    pub funding_goal: Uint128,
    pub deadline: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Contribute {},
    Withdraw { amount: Uint128 },
    Refund {},
    ExtendDeadline { new_deadline: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetProjectDetailsResponse)]
    GetProjectDetails {},
    #[returns(GetContributorsResponse)]
    GetContributors {},
}

#[cw_serde]
pub struct GetProjectDetailsResponse {
    pub name: String,
    pub description: String,
    pub funding_goal: Uint128,
    pub current_funding: Uint128,
    pub deadline: u64,
    pub owner: Addr,
}

#[cw_serde]
pub struct GetContributorsResponse {
    pub contributors: Vec<(Addr, Uint128)>,
}