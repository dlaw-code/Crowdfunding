use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub name: String,
    pub description: String,
    pub funding_goal: u128,
    pub current_funding: u128,
    pub deadline: u64,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
pub const CONTRIBUTORS: Map<&Addr, u128> = Map::new("contributors");
