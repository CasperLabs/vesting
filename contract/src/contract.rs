use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};

use contract_macro::{
    casperlabs_constructor, casperlabs_contract, casperlabs_initiator, casperlabs_method,
};
use contract::{
    contract_api::{account, runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    bytesrepr::ToBytes,
    account::AccountHash,
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    runtime_args, CLType, CLTyped, CLValue, Group, Key, Parameter, RuntimeArgs, URef, U512,
};

use logic::{VestingError, VestingTrait};

use crate::error::Error;
use crate::utils;
use crate::vesting::VestingContract;

mod key {
    pub const PURSE_NAME: &str = "vesting_main_purse";
    pub const ADMIN: &str = "admin_account";
    pub const RECIPIENT: &str = "recipient_account";
}

mod arg {
    pub const TOTAL_AMOUNT: &str = "total_amount";
    pub const AMOUNT: &str = "amount";
}

#[casperlabs_contract]
mod vesting_contract {
    use super::*;

    #[casperlabs_constructor]
    fn init(
        admin: AccountHash,
        recipient: AccountHash,
        cliff_timestamp: U512,
        cliff_amount: U512,
        drip_duration: U512,
        drip_amount: U512,
        total_amount: U512,
        admin_release_duration: U512,
    ) {
        let mut vault = VestingContract;
        set_admin_account(admin);
        set_recipient_account(recipient);
        vault.init(
            cliff_timestamp,
            cliff_amount,
            drip_duration,
            drip_amount,
            total_amount,
            admin_release_duration,
        );
    }

    #[casperlabs_method]
    fn pause() {
        verify_admin_account();
        let mut vault = VestingContract;
        match vault.pause() {
            Ok(()) => {}
            Err(VestingError::AlreadyPaused) => runtime::revert(Error::AlreadyPaused),
            _ => runtime::revert(Error::UnexpectedVestingError),
        }
    }

    #[casperlabs_method]
    fn unpause() {
        verify_admin_account();
        let mut vault = VestingContract;
        match vault.unpause() {
            Ok(()) => {}
            Err(VestingError::AlreadyUnpaused) => runtime::revert(Error::AlreadyUnpaused),
            _ => runtime::revert(Error::UnexpectedVestingError),
        }
    }

    #[casperlabs_method]
    fn withdraw() {
        verify_recipient_account();
        let mut vault = VestingContract;
        let amount = runtime::get_named_arg(arg::AMOUNT);
        match vault.withdraw(amount) {
            Ok(()) => utils::transfer_out_clx_to_caller(key::PURSE_NAME, amount),
            Err(VestingError::NotEnoughBalance) => runtime::revert(Error::NotEnoughBalance),
            _ => runtime::revert(Error::UnexpectedVestingError),
        }
    }

    #[casperlabs_method]
    fn admin_release() {
        verify_admin_account();
        let mut vault = VestingContract;
        match vault.admin_release() {
            Ok(amount) => utils::transfer_out_clx_to_caller(key::PURSE_NAME, amount),
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

    #[casperlabs_initiator]
    fn init_state() -> BTreeMap<String, Key> {
        let main_purse = account::get_main_purse();
        let vesting_purse = system::create_purse();
        let total_amount = runtime::get_named_arg(arg::TOTAL_AMOUNT);
        system::transfer_from_purse_to_purse(main_purse, vesting_purse, total_amount)
            .unwrap_or_revert_with(Error::PurseTransferError);
        let mut vesting_keys: BTreeMap<String, Key> = BTreeMap::new();
        vesting_keys.insert(String::from(key::PURSE_NAME), vesting_purse.into());
        vesting_keys
    }
}

fn set_recipient_account(recipient: AccountHash) {
    utils::set_key(key::RECIPIENT, recipient);
}

fn set_admin_account(admin: AccountHash) {
    utils::set_key(key::ADMIN, admin);
}

fn recipient_account() -> AccountHash {
    utils::get_key(key::RECIPIENT)
}

fn admin_account() -> AccountHash {
    utils::get_key(key::ADMIN)
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
