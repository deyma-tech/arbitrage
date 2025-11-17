use solana_program::{
    decode_error::DecodeError, msg, program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;
#[derive(Clone, Copy, Debug, Eq, Error, num_derive::FromPrimitive, PartialEq)]
pub enum NumeraireError {}
impl From<NumeraireError> for ProgramError {
    fn from(e: NumeraireError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for NumeraireError {
    fn type_of() -> &'static str {
        "NumeraireError"
    }
}
impl PrintProgramError for NumeraireError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError
            + num_traits::FromPrimitive,
    {
        msg!(& self.to_string());
    }
}
