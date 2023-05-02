use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Two custom coins must be deposited")]
    NoCoinsSent {},

    #[error("A maximum of two custom coins can be deposited")]
    TooManyCoins {},

    #[error("The deposited denom is not supported")]
    DenomsNotSupported { denoms: Vec<String> },

    #[error("You are only allowed to deposit a minimum of two custom coins to the pool")]
    OnlyTwoAllowed {},

    #[error("There is insufficient balance for the denom you would like to withdraw")]
    InsufficientBalance { denom: String },
}
