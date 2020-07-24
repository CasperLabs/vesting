use types::ApiError;

#[repr(u16)]
pub enum Error {
    AlreadyPaused = 1,
    AlreadyUnpaused = 2,
    NotTheAdminAccount = 3,
    NotTheRecipientAccount = 4,
    UnexpectedVestingError = 5,
    NotEnoughBalance = 6,
    PurseTransferError = 7,
    NotPaused = 8,
    NothingToWithdraw = 9,
    NotEnoughTimeElapsed = 10,
    LocalPurseKeyMissing = 11,
    UnexpectedType = 12,
    MissingKey = 13,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}
