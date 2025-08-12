use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
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
}
