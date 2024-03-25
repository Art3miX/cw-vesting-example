use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const VESTING_CODE_ID: Item<u64> = Item::new("vesting_code_id");
pub const VESTING_CONTRACTS: Map<String, Addr> = Map::new("vesting_contracts");

pub const INIT_FOR: Item<String> = Item::new("init_for");
