use {pinocchio::program_error::ProgramError, std::mem::size_of};

#[derive(Clone, Debug, PartialEq)]
pub enum RecordInstruction {
    Initialize,
    WriteU64 { offset: u64 },
    CheckAdd { offset: u64, addition: u64 },
    SetAuthority,
    CloseAccount,
}

impl RecordInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        const U64_BYTES: usize = 8;

        let (&tag, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match tag {
            0 => Self::Initialize,
            1 => Self::WriteU64 {
                offset: rest
                    .get(..U64_BYTES)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?,
            },
            2 => Self::CheckAdd {
                offset: rest
                    .get(..U64_BYTES)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?,
                addition: rest[U64_BYTES..]
                    .get(..U64_BYTES)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?,
            },
            3 => Self::SetAuthority,
            4 => Self::CloseAccount,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }

    /// Packs a [`RecordInstruction`] into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::Initialize => buf.push(0),
            Self::WriteU64 { offset } => {
                buf.push(1);
                buf.extend_from_slice(&offset.to_le_bytes());
            }
            Self::CheckAdd { offset, addition } => {
                buf.push(2);
                buf.extend_from_slice(&offset.to_le_bytes());
                buf.extend_from_slice(&addition.to_le_bytes());
            }
            Self::SetAuthority => buf.push(3),
            Self::CloseAccount => buf.push(4),
        };
        buf
    }
}
