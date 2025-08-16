use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Deadline exceeded")]
    DeadlineExceeded {},

    #[error("Withdrawal not allowed")]
    WithdrawalNotAllowed {},

    #[error("Refund not allowed")]
    RefundNotAllowed {},

    #[error("New deadline must be in the future")]
    InvalidDeadline {},

    #[error("Contribution amount must be positive")]
    EmptyContribution {},
    
    #[error("Arithmetic overflow occurred")]
    Overflow {},

    #[error("Must send exactly 1 'earth' token")]
    InvalidFunds {},
}

impl From<cosmwasm_std::OverflowError> for ContractError {
    fn from(err: cosmwasm_std::OverflowError) -> Self {
        ContractError::Std(StdError::overflow(err))
    }
}