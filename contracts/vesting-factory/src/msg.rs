use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use vesting::msg::{Receiver, VestingStrategy};

#[cw_serde]
pub struct InstantiateMsg {
    pub vesting_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Generate new vesting contract using those parameters
    CreateVesting {
        /// Receiver of the vested funds
        receiver: Receiver,
        /// Vesting strategy
        vesting_strategy: VestingStrategy,
        /// Label the created vesting contract
        label: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    GetVestingAddr { receiver: String },
}
