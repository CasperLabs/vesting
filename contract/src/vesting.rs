use core::convert::TryInto;

use casperlabs_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casperlabs_types::{
    account::PublicKey,
    bytesrepr::{FromBytes, ToBytes},
    CLTyped, URef, U512,
};

use crate::{api::Api, error::Error};
use logic::{VestingError, VestingTrait};

pub const INIT_FLAG_KEY: &str = "is_initialized";
pub const ADMIN_KEY: &str = "admin_account";
pub const RECIPIENT_KEY: &str = "recipient_account";
pub const PURSE_NAME: &str = "vesting_main_purse";

type Amount = U512;
type Time = U512;

struct VestingContract;

impl VestingTrait<Amount, Time> for VestingContract {
    fn set_amount(&mut self, name: &str, value: Amount) {
        set_key(name, value)
    }
    fn amount(&self, name: &str) -> Amount {
        key(name)
    }
    fn set_time(&mut self, name: &str, value: Time) {
        set_key(name, value);
    }
    fn time(&self, name: &str) -> Time {
        key(name)
    }
    fn set_boolean(&mut self, name: &str, value: bool) {
        set_key(name, value)
    }
    fn boolean(&self, name: &str) -> bool {
        key(name)
    }
    fn current_timestamp(&self) -> Time {
        let time: u64 = runtime::get_blocktime().into();
        time.into()
    }
}

fn construct() {
    let mut vault = VestingContract;
    match Api::from_args() {
        Api::Init(admin, recipient, vesting_config) => {
            set_admin_account(admin);
            set_recipient_account(recipient);
            vault.init(
                vesting_config.cliff_time,
                vesting_config.cliff_amount,
                vesting_config.drip_period,
                vesting_config.drip_amount,
                vesting_config.total_amount,
                vesting_config.admin_release_period,
            );
        }
        _ => runtime::revert(Error::UnknownConstructorCommand),
    }
}

fn entry_point() {
    let mut vault = VestingContract;
    match Api::from_args() {
        Api::Pause => {
            verify_admin_account();
            match vault.pause() {
                Ok(()) => {}
                Err(VestingError::AlreadyPaused) => runtime::revert(Error::AlreadyPaused),
                _ => runtime::revert(Error::UnexpectedVestingError),
            }
        }
        Api::Unpause => {
            verify_admin_account();
            match vault.unpause() {
                Ok(()) => {}
                Err(VestingError::AlreadyUnpaused) => runtime::revert(Error::AlreadyUnpaused),
                _ => runtime::revert(Error::UnexpectedVestingError),
            }
        }
        Api::Withdraw(purse, amount) => {
            verify_recipient_account();
            match vault.withdraw(amount) {
                Ok(()) => transfer_out_clx_to_purse(purse, amount),
                Err(VestingError::NotEnoughBalance) => runtime::revert(Error::NotEnoughBalance),
                _ => runtime::revert(Error::UnexpectedVestingError),
            }
        }
        Api::AdminRelease(purse) => {
            verify_admin_account();
            match vault.admin_release() {
                Ok(amount) => transfer_out_clx_to_purse(purse, amount),
                Err(VestingError::AdminReleaseErrorNotPaused) => runtime::revert(Error::NotPaused),
                Err(VestingError::AdminReleaseErrorNothingToWithdraw) => {
                    runtime::revert(Error::NothingToWithdraw)
                }
                Err(VestingError::AdminReleaseErrorNotEnoughTimeElapsed) => {
                    runtime::revert(Error::NotEnoughTimeElapsed)
                }
                _ => runtime::revert(Error::UnexpectedVestingError),
            }
        }
        _ => runtime::revert(Error::UnknownVestingCallCommand),
    }
}

fn is_initialized() -> bool {
    runtime::has_key(INIT_FLAG_KEY)
}

fn mark_as_initialized() {
    set_key(INIT_FLAG_KEY, 1);
}

fn set_admin_account(admin: PublicKey) {
    set_key(ADMIN_KEY, admin);
}

fn admin_account() -> PublicKey {
    key(ADMIN_KEY)
}

fn set_recipient_account(recipient: PublicKey) {
    set_key(RECIPIENT_KEY, recipient);
}

fn recipient_account() -> PublicKey {
    key(RECIPIENT_KEY)
}

fn verify_admin_account() {
    let admin = admin_account();
    let caller = runtime::get_caller();
    if admin != caller {
        runtime::revert(Error::NotTheAdminAccount);
    }
}

fn verify_recipient_account() {
    let recipient = recipient_account();
    let caller = runtime::get_caller();
    if recipient != caller {
        runtime::revert(Error::NotTheRecipientAccount);
    }
}

fn transfer_out_clx_to_purse(purse: URef, amount: U512) {
    let local_purse = local_purse();
    system::transfer_from_purse_to_purse(local_purse, purse, amount)
        .unwrap_or_revert_with(Error::PurseTransferError);
}

fn local_purse() -> URef {
    let key = runtime::get_key(PURSE_NAME).unwrap_or_revert_with(Error::LocalPurseKeyMissing);
    key.into_uref().unwrap_or_revert_with(Error::UnexpectedType)
}

fn key<T: FromBytes + CLTyped>(name: &str) -> T {
    let key = runtime::get_key(name)
        .unwrap_or_revert_with(Error::MissingKey)
        .try_into()
        .unwrap_or_revert_with(Error::UnexpectedType);
    storage::read(key)
        .unwrap_or_revert_with(Error::MissingKey)
        .unwrap_or_revert_with(Error::UnexpectedType)
}

fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}

#[no_mangle]
pub extern "C" fn vesting() {
    if !is_initialized() {
        construct();
        mark_as_initialized();
    } else {
        entry_point();
    }
}
