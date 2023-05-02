use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    // the first custom coin that can be deposited and withdrawn
    pub accepted_denoms: [String; 2],
}

impl Config {
    /// returns true if the address is the registered owner
    pub fn is_owner(&self, addr: impl AsRef<str>) -> bool {
        let addr = addr.as_ref();
        self.owner == addr
    }
}

pub const CONFIG: Item<Config> = Item::new("config");

// this keeps track of the contract balance of the two custom coins after deposits and withdrawals
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolBalance {
    pub amount: u128,
}

// string here is the denom
pub const POOLBALANCE: Map<String, PoolBalance> = Map::new("poolbalance");
