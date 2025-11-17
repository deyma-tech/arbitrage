use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;
#[derive(Clone, Copy, Debug, Eq, Error, num_derive::FromPrimitive, PartialEq)]
pub enum GammaError {}
impl From<GammaError> for ProgramError {
    fn from(e: GammaError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for GammaError {
    fn type_of() -> &'static str {
        "GammaError"
    }
}
impl PrintProgramError for GammaError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + num_traits::FromPrimitive,
    {
        msg!(&self.to_string());
    }
}
