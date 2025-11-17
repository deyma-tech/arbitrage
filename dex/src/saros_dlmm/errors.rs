use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;
#[derive(Clone, Copy, Debug, Eq, Error, num_derive::FromPrimitive, PartialEq)]
pub enum LiquidityBookError {}
impl From<LiquidityBookError> for ProgramError {
    fn from(e: LiquidityBookError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for LiquidityBookError {
    fn type_of() -> &'static str {
        "LiquidityBookError"
    }
}
impl PrintProgramError for LiquidityBookError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + num_traits::FromPrimitive,
    {
        msg!(&self.to_string());
    }
}
