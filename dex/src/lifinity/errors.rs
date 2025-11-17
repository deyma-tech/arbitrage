use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;
#[derive(Clone, Copy, Debug, Eq, Error, num_derive::FromPrimitive, PartialEq)]
pub enum LifinityAmmV2Error {
    #[error("Swap account already in use")]
    AlreadyInUse = 6000,
    #[error("Invalid program address generated from bump seed and key")]
    InvalidProgramAddress = 6001,
    #[error("Input account owner is not the program address")]
    InvalidOwner = 6002,
    #[error("Output pool account owner cannot be the program address")]
    InvalidOutputOwner = 6003,
    #[error("Deserialized account is not an SPL Token mint")]
    ExpectedMint = 6004,
    #[error("Deserialized account is not an SPL Token account")]
    ExpectedAccount = 6005,
    #[error("Input token account empty")]
    EmptySupply = 6006,
    #[error("Pool token mint has a non-zero supply")]
    InvalidSupply = 6007,
    #[error("Token account has a delegate")]
    InvalidDelegate = 6008,
    #[error("InvalidInput")]
    InvalidInput = 6009,
    #[error("Address of the provided swap token account is incorrect")]
    IncorrectSwapAccount = 6010,
    #[error("Address of the provided pool token mint is incorrect")]
    IncorrectPoolMint = 6011,
    #[error("InvalidOutput")]
    InvalidOutput = 6012,
    #[error("General calculation failure due to overflow or underflow")]
    CalculationFailure = 6013,
    #[error("Invalid instruction")]
    InvalidInstruction = 6014,
    #[error("Swap input token accounts have the same mint")]
    RepeatedMint = 6015,
    #[error("Swap instruction exceeds desired slippage limit")]
    ExceededSlippage = 6016,
    #[error("Token account has a close authority")]
    InvalidCloseAuthority = 6017,
    #[error("Pool token mint has a freeze authority")]
    InvalidFreezeAuthority = 6018,
    #[error("Pool fee token account incorrect")]
    IncorrectFeeAccount = 6019,
    #[error("Given pool token amount results in zero trading tokens")]
    ZeroTradingTokens = 6020,
    #[error("Fee calculation failed due to overflow, underflow, or unexpected 0")]
    FeeCalculationFailure = 6021,
    #[error("Conversion to u64 failed with an overflow or underflow")]
    ConversionFailure = 6022,
    #[error("The provided fee does not match the program owner's constraints")]
    InvalidFee = 6023,
    #[error("The provided token program does not match the token program expected by the swap")]
    IncorrectTokenProgramId = 6024,
    #[error("Address of the provided oracle account is incorrect")]
    IncorrectOracleAccount = 6025,
    #[error("Address of the provided config account is incorrect")]
    IncorrectConfigAccount = 6026,
    #[error("The provided curve type is not supported by the program owner")]
    UnsupportedCurveType = 6027,
    #[error("The provided curve parameters are invalid")]
    InvalidCurve = 6028,
    #[error("The operation cannot be performed on the given curve")]
    UnsupportedCurveOperation = 6029,
    #[error("Pyth oracle status is not 'trading'")]
    InvalidPythStatus = 6030,
    #[error("Could not retrieve updated price feed from the Pyth oracle")]
    InvalidPythPrice = 6031,
    #[error("Address of the provided signer account is incorrect")]
    IncorrectSigner = 6032,
    #[error("Swap amount exceeds pool balance")]
    ExceedPoolBalance = 6033,
    #[error("Program is frozen")]
    ProgramIsFrozen = 6034,
    #[error("Oracle confidence is too high")]
    OracleConfidence = 6035,
    #[error("Over Pool Cap Amount")]
    OverCapAmount = 6036,
    #[error("Invalid update wallet address")]
    InvalidUpdateAccount = 6037,
    #[error("Invalid update param")]
    InvalidUpdateParam = 6038,
    #[error("Invalid inner swap account")]
    InvalidInnerSwapAccount = 6039,
    #[error("Exception err")]
    ExceptionErr = 6040,
}
impl From<LifinityAmmV2Error> for ProgramError {
    fn from(e: LifinityAmmV2Error) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for LifinityAmmV2Error {
    fn type_of() -> &'static str {
        "LifinityAmmV2Error"
    }
}
impl PrintProgramError for LifinityAmmV2Error {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + num_traits::FromPrimitive,
    {
        msg!(&self.to_string());
    }
}
