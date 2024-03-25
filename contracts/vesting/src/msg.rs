use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub receiver: Receiver,
    pub strategy: VestingStrategy,
}

#[cw_serde]
pub enum ExecuteMsg {
    Claim {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetClaimable {},
}

#[cw_serde]
pub enum Receiver {
    Ibc {
        address: String,
        channel_id: String,
        claimer: String,
    },
    Native {
        address: String,
    },
}

#[cw_serde]
pub enum VestingStrategy {
    Hour,
    Day,
    Week,
    Month,
}

impl VestingStrategy {
    pub fn to_seconds(&self) -> u64 {
        match self {
            VestingStrategy::Hour => 3600,
            VestingStrategy::Day => 86400,
            VestingStrategy::Week => 604800,
            VestingStrategy::Month => 2592000,
        }
    }
}
