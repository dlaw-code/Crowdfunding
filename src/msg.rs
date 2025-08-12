use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub description: String,
    pub funding_goal: u128,
    pub deadline: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Contribute {},
    Withdraw { amount: u128 },
    Refund {},
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
    pub funding_goal: u128,
    pub current_funding: u128,
    pub deadline: u64,
    pub owner: Addr,
}

#[cw_serde]
pub struct GetContributorsResponse {
    pub contributors: Vec<(Addr, u128)>,
}
