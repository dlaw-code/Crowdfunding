use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub name: String,
    pub description: String,
    pub funding_goal: Uint128,
    pub current_funding: Uint128,
    pub deadline: u64,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
pub const CONTRIBUTORS: Map<&Addr, Uint128> = Map::new("contributors");