use cosmwasm_std::{Instantiate2AddressError, StdError};
use cw_utils::{ParseReplyError, PaymentError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Never {}

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    Payment(#[from] PaymentError),

    #[error(transparent)]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error(transparent)]
    ParseReplyError(#[from] ParseReplyError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Must send funds to start vesting")]
    NoFundsSent,

    #[error("Reply id not found = {0}")]
    InvalidReplyId(u64),
}
