#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_json_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, IbcMsg, IbcTimeout,
    MessageInfo, Response, StdResult, Timestamp, Uint128,
};

use cw2::set_contract_version;
use cw_utils::must_pay;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Receiver},
    state::{Config, CONFIG},
};

const CONTRACT_NAME: &str = "crates.io:vesting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Make sure we have at least 1 funding token
    if info.funds.is_empty() {
        return Err(ContractError::NoFundsSent);
    }

    must_pay(&info, &info.funds[0].denom)?;

    let (receiver, ibc_channel_id, claimer) = match msg.receiver {
        Receiver::Ibc {
            address,
            channel_id,
            claimer,
        } => (address, Some(channel_id), deps.api.addr_validate(&claimer)?),
        Receiver::Native { address } => {
            let addr = deps.api.addr_validate(&address)?;
            (addr.to_string(), None, addr)
        }
    };

    // Get vesting strategy in seconds
    let seconds_till_end = msg.strategy.to_seconds();

    let config = Config {
        denom: info.funds[0].denom.clone(),
        receiver,
        claimer,
        start: env.block.time,
        end: env.block.time.plus_seconds(seconds_till_end),
        ibc_channel_id,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
    }
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // verify the sender is the claimer
    if info.sender != config.claimer {
        return Err(ContractError::Unauthorized(
            "Only the claimer can call this message".to_string(),
        ));
    }

    // Get the contract balance
    let balance = deps
        .querier
        .query_balance(env.contract.address, &config.denom)?;

    // If current time is after the end time, we pay everything.
    let amount = if env.block.time.seconds() > config.end.seconds() {
        balance.amount
    } else {
        // calculate how much per second we need to pay
        let total_seconds = config.end.seconds() - config.start.seconds();
        let amount_per_second = balance.amount / Uint128::from(total_seconds);
        let seconds_from_start = env.block.time.seconds() - config.start.seconds();

        amount_per_second * Uint128::from(seconds_from_start)
    };

    // construct the message based on if the receiver is over IBC or native
    let msg: CosmosMsg = match config.ibc_channel_id.clone() {
        Some(channel_id) => IbcMsg::Transfer {
            channel_id,
            amount: coin(amount.u128(), &config.denom),
            to_address: config.receiver.clone(),
            timeout: IbcTimeout::with_timestamp(Timestamp::from_seconds(
                env.block.time.seconds() + 60 * 60,
            )),
        }
        .into(),
        None => BankMsg::Send {
            to_address: config.receiver.clone(),
            amount: vec![coin(amount.u128(), &config.denom)],
        }
        .into(),
    };

    config.start = env.block.time;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("amount_sent", amount))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetClaimable {} => {
            let config = CONFIG.load(deps.storage)?;

            let balance = deps
                .querier
                .query_balance(env.contract.address, &config.denom)?;

            let amount = if env.block.time.seconds() > config.end.seconds() {
                balance.amount
            } else {
                // calculate how much per second we need to pay
                let total_seconds = config.end.seconds() - config.start.seconds();
                let amount_per_second = balance.amount / Uint128::from(total_seconds);
                let seconds_from_start = env.block.time.seconds() - config.start.seconds();

                amount_per_second * Uint128::from(seconds_from_start)
            };

            to_json_binary(&amount)
        }
    }
}
