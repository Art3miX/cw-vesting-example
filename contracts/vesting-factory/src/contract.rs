#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    instantiate2_address, to_json_binary, Binary, CodeInfoResponse, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdResult, SubMsg, WasmMsg,
};

use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use vesting::msg::{Receiver, VestingStrategy};

use sha2::{Digest, Sha256};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{INIT_FOR, VESTING_CODE_ID, VESTING_CONTRACTS},
};

const CONTRACT_NAME: &str = "crates.io:vesting-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_VESTING_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    VESTING_CODE_ID.save(deps.storage, &msg.vesting_code_id)?;

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
        ExecuteMsg::CreateVesting {
            receiver,
            vesting_strategy,
            label,
        } => execute_create_vesting(deps, env, info, receiver, vesting_strategy, label),
    }
}

pub fn execute_create_vesting(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    receiver: Receiver,
    vesting_strategy: VestingStrategy,
    label: String,
) -> Result<Response, ContractError> {
    if info.funds.is_empty() {
        return Err(ContractError::NoFundsSent);
    }

    let code_id = VESTING_CODE_ID.load(deps.storage)?;
    let sub_msg = SubMsg::reply_on_success(
        WasmMsg::Instantiate {
            admin: None,
            code_id,
            msg: to_json_binary(&vesting::msg::InstantiateMsg {
                receiver: receiver.clone(),
                strategy: vesting_strategy,
            })?,
            funds: info.funds,
            label,
        },
        INSTANTIATE_VESTING_ID,
    );

    // Save the receiver address for the reply
    let receiver_addr = match receiver {
        Receiver::Ibc { address, .. } => address,
        Receiver::Native { address } => address,
    };

    INIT_FOR.save(deps.storage, &receiver_addr)?;

    Ok(Response::new().add_submessage(sub_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(
    deps: DepsMut,
    _env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    match reply.id {
        INSTANTIATE_VESTING_ID => {
            let reply = parse_reply_instantiate_data(reply)?;

            // Get the address we instantiated the vesting for
            let receiver = INIT_FOR.load(deps.storage)?;

            // Save in our map to track for receiver -> vesting contract address
            VESTING_CONTRACTS.save(
                deps.storage,
                receiver,
                &deps.api.addr_validate(&reply.contract_address)?,
            )?;

            // Remove the INIT_FOR storage
            INIT_FOR.remove(deps.storage);

            Ok(Response::default())
        }
        _ => Err(ContractError::InvalidReplyId(reply.id)),
    }
}

// pub fn execute_create_vesting(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     receiver: Receiver,
//     vesting_strategy: VestingStrategy,
//     label: String,
// ) -> Result<Response, ContractError> {
//     if info.funds.is_empty() {
//         return Err(ContractError::NoFundsSent);
//     }

//     let code_id = VESTING_CODE_ID.load(deps.storage)?;

//     // Get the receiver address
//     let receiver_addr = match receiver.clone() {
//         Receiver::Ibc { address, .. } => address,
//         Receiver::Native { address } => address,
//     };

//     // Generate a salt
//     let mut hasher = Sha256::new();
//     hasher.update(receiver_addr.clone());
//     let salt = hasher.finalize().to_vec();

//     // Get the canonical address of the contract creator
//     let canonical_creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;

//     // get the checksum of the contract we're going to instantiate
//     let CodeInfoResponse { checksum, .. } = deps.querier.query_wasm_code_info(code_id)?;

//     let canonical_cw721_addr = instantiate2_address(&checksum, &canonical_creator, &salt)?;
//     let vesting_addr = deps.api.addr_humanize(&canonical_cw721_addr)?;

//     // Save the generated vesting address to the receiver address
//     VESTING_CONTRACTS.save(deps.storage, receiver_addr, &vesting_addr)?;

//     let msg = WasmMsg::Instantiate2 {
//         admin: None,
//         code_id,
//         label,
//         msg: to_json_binary(&vesting::msg::InstantiateMsg {
//             receiver,
//             strategy: vesting_strategy,
//         })?,
//         funds: info.funds,
//         salt: salt.into(),
//     };

//     Ok(Response::new().add_message(msg))
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetVestingAddr { receiver } => {
            to_json_binary(&VESTING_CONTRACTS.load(deps.storage, receiver)?)
        }
    }
}
