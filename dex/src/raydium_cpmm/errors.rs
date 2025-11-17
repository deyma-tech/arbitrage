use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;
#[derive(Clone, Copy, Debug, Eq, Error, num_derive::FromPrimitive, PartialEq)]
pub enum RaydiumCpmmError {
    #[error("Not approved")]
    NotApproved = 6000,
    #[error("Input account owner is not the program address")]
    InvalidOwner = 6001,
    #[error("Input token account empty")]
    EmptySupply = 6002,
    #[error("InvalidInput")]
    InvalidInput = 6003,
    #[error("Address of the provided lp token mint is incorrect")]
    IncorrectLpMint = 6004,
    #[error("Exceeds desired slippage limit")]
    ExceededSlippage = 6005,
    #[error("Given pool token amount results in zero trading tokens")]
    ZeroTradingTokens = 6006,
    #[error("Not support token_2022 mint extension")]
    NotSupportMint = 6007,
    #[error("invaild vault")]
    InvalidVault = 6008,
    #[error("Init lp amount is too less(Because 100 amount lp will be locked)")]
    InitLpAmountTooLess = 6009,
}
impl From<RaydiumCpmmError> for ProgramError {
    fn from(e: RaydiumCpmmError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for RaydiumCpmmError {
    fn type_of() -> &'static str {
        "RaydiumCpmmError"
    }
}
impl PrintProgramError for RaydiumCpmmError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + num_traits::FromPrimitive,
    {
        msg!(&self.to_string());
    }
}
