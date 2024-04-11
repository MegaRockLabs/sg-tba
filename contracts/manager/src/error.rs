use cosmwasm_std::{OverflowError, StdError, Uint128};
use cw_utils::{ParseReplyError, PaymentError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Insufficient fee: expected {0}, got {1}")]
    InsufficientFee(Uint128, Uint128),

    #[error("All provided fees are not sufficient")]
    InsufficienFees {},

    #[error("None of the sent tokens are accepted by the contract")]
    NoFeeTokens {},

    #[error("Code ids are invalid or not provided")]
    InvalidCodeIds {},

    #[error("Must provide at least one correct creating fee")]
    InvalidCreationFees {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not Supported Chain ID")]
    InvalidChainId {},

    #[error("Code ID not allowed")]
    InvalidCodeId {},

    #[error("Account for the given token already exists. Use `reset_account` to overwrite it and `migrate_account` to update it to a newer version")]
    AccountExists {},

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

impl From<ParseReplyError> for ContractError {
    fn from(err: ParseReplyError) -> Self {
        Self::Std(StdError::GenericErr { msg: err.to_string() })
    }
}