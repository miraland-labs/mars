use num_enum::IntoPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum MarsError {
    #[error("The starting time has not passed yet")]
    NotStarted = 0,
    #[error("The epoch has ended and needs reset")]
    NeedsReset = 1,
    #[error("The epoch is active and cannot be reset at this time")]
    ResetTooEarly = 2,
    #[error("The provided hash was invalid")]
    HashInvalid = 3,
    #[error("The provided hash does not satisfy the difficulty requirement")]
    DifficultyNotSatisfied = 4,
    #[error("The bus does not have enough rewards to issue at this time")]
    BusRewardsInsufficient = 5,
    #[error("The claim amount cannot be greater than the claimable rewards")]
    ClaimTooLarge = 6,
    #[error("The mining has ended")]
    HasEnded = 7,
}

impl From<MarsError> for ProgramError {
    fn from(e: MarsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
