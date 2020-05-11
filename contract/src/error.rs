use casperlabs_types::ApiError;

#[repr(u16)]
pub enum Error {
    UnknownApiCommand = 1,             // 65537
    UnknownDeployCommand = 2,          // 65538
    UnknownProxyCommand = 3,           // 65539
    UnknownConstructorCommand = 4,     // 65540
    UnknownVestingCallCommand = 5,     // 65541
    AlreadyPaused = 6,                 // 65542
    AlreadyUnpaused = 7,               // 65543
    NotTheAdminAccount = 8,            // 65544
    NotTheRecipientAccount = 9,        // 65545
    UnexpectedVestingError = 10,       // 65546
    NotEnoughBalance = 11,             // 65547
    PurseTransferError = 12,           // 65548
    PurseBalanceCheckError = 13,       // 65549
    NotPaused = 14,                    // 65550
    NothingToWithdraw = 15,            // 65551
    NotEnoughTimeElapsed = 16,         // 65552
    LocalPurseKeyMissing = 17,         // 65553
    UnexpectedType = 18,               // 65554
    MissingKey = 19,                   // 65555
    MissingArgument0 = 20,             // 65556
    MissingArgument1 = 21,             // 65557
    MissingArgument2 = 22,             // 65558
    MissingArgument3 = 23,             // 65559
    MissingArgument4 = 24,             // 65560
    MissingArgument5 = 25,             // 65561
    MissingArgument6 = 26,             // 65562
    MissingArgument7 = 27,             // 65563
    MissingArgument8 = 28,             // 65564
    MissingArgument9 = 29,             // 65565
    InvalidArgument0 = 30,             // 65566
    InvalidArgument1 = 31,             // 65567
    InvalidArgument2 = 32,             // 65568
    InvalidArgument3 = 33,             // 65569
    InvalidArgument4 = 34,             // 65570
    InvalidArgument5 = 35,             // 65571
    InvalidArgument6 = 36,             // 65572
    InvalidArgument7 = 37,             // 65573
    InvalidArgument8 = 38,             // 65574
    InvalidArgument9 = 39,             // 65575
    UnsupportedNumberOfArguments = 40, // 65576
}

impl Error {
    pub fn missing_argument(i: u32) -> Error {
        match i {
            0 => Error::MissingArgument0,
            1 => Error::MissingArgument1,
            2 => Error::MissingArgument2,
            3 => Error::MissingArgument3,
            4 => Error::MissingArgument4,
            5 => Error::MissingArgument5,
            6 => Error::MissingArgument6,
            7 => Error::MissingArgument7,
            8 => Error::MissingArgument8,
            9 => Error::MissingArgument9,
            _ => Error::UnsupportedNumberOfArguments,
        }
    }

    pub fn invalid_argument(i: u32) -> Error {
        match i {
            0 => Error::InvalidArgument0,
            1 => Error::InvalidArgument1,
            2 => Error::InvalidArgument2,
            3 => Error::InvalidArgument3,
            4 => Error::InvalidArgument4,
            5 => Error::InvalidArgument5,
            6 => Error::InvalidArgument6,
            7 => Error::InvalidArgument7,
            8 => Error::InvalidArgument8,
            9 => Error::InvalidArgument9,
            _ => Error::UnsupportedNumberOfArguments,
        }
    }
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}
