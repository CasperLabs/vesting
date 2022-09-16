#![no_main]

extern crate alloc;
mod error;
use contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use core::cmp;
use error::Error;
use std::convert::TryInto;
use types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    contracts::{EntryPoints, NamedKeys},
    CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, Parameter, URef, U512,
};

pub type Time = U512;
pub type Amount = U512;

pub const ADMIN: &str = "admin_account";
pub const ADMIN_RELEASE_DURATION: &str = "admin_release_duration";
pub const CLIFF_AMOUNT: &str = "cliff_amount";
pub const CLIFF_TIMESTAMP: &str = "cliff_timestamp";
pub const DRIP_AMOUNT: &str = "drip_amount";
pub const DRIP_DURATION: &str = "drip_duration";
pub const LAST_PAUSE_TIMESTAMP: &str = "last_pause_timestamp";
pub const ON_PAUSE_DURATION: &str = "on_pause_duration";
pub const PAUSE_FLAG: &str = "is_paused";
pub const PURSE_NAME: &str = "vesting_main_purse";
pub const RECIPIENT: &str = "recipient_account";
pub const RELEASED_AMOUNT: &str = "released_amount";
pub const TOTAL_AMOUNT: &str = "total_amount";

#[no_mangle]
pub extern "C" fn pause() {
    verify_admin_account();

    let is_paused: bool = get_key(PAUSE_FLAG);

    if !is_paused {
        set_key(LAST_PAUSE_TIMESTAMP, current_timestamp());
        set_key(PAUSE_FLAG, true);
    } else {
        runtime::revert(Error::AlreadyPaused);
    }
}

#[no_mangle]
pub extern "C" fn unpause() {
    verify_admin_account();

    let is_paused: bool = get_key(PAUSE_FLAG);

    if is_paused {
        let on_pause_duration: U512 = get_key(ON_PAUSE_DURATION);
        let last_pause_timestamp: U512 = get_key(LAST_PAUSE_TIMESTAMP);
        let elapsed_timestamp: U512 = current_timestamp() - last_pause_timestamp;
        set_key(ON_PAUSE_DURATION, on_pause_duration + elapsed_timestamp);
        set_key(PAUSE_FLAG, false);
    } else {
        runtime::revert(Error::AlreadyUnpaused);
    }
}

#[no_mangle]
pub extern "C" fn withdraw() {
    verify_recipient_account();

    let amount: U512 = runtime::get_named_arg("amount");
    let available_amount = available_amount();
    if available_amount < amount {
        runtime::revert(Error::NotEnoughBalance);
    } else {
        let released_amount: U512 = get_key(RELEASED_AMOUNT);
        set_key(RELEASED_AMOUNT, released_amount + amount);
        transfer_out_clx_to_caller(PURSE_NAME, amount);
    }
}

#[no_mangle]
pub extern "C" fn admin_release() {
    verify_admin_account();

    let is_paused: bool = get_key(PAUSE_FLAG);
    if !is_paused {
        runtime::revert(Error::NotPaused);
    }
    let last_pause_timestamp: U512 = get_key(LAST_PAUSE_TIMESTAMP);
    let since_last_pause = current_timestamp() - last_pause_timestamp;
    let required_wait_duration: U512 = get_key(ADMIN_RELEASE_DURATION);
    if since_last_pause < required_wait_duration {
        runtime::revert(Error::NotEnoughTimeElapsed);
    }
    let total_amount: U512 = get_key(TOTAL_AMOUNT);
    let released_amount: U512 = get_key(RELEASED_AMOUNT);
    if total_amount == released_amount {
        runtime::revert(Error::NothingToWithdraw);
    }
    let amount_to_withdraw = total_amount - released_amount;
    set_key(RELEASED_AMOUNT, total_amount);
    transfer_out_clx_to_caller(PURSE_NAME, amount_to_withdraw);
}

#[no_mangle]
pub extern "C" fn get_deposit_purse() {
    verify_admin_account();

    let vesting_purse = match runtime::get_key(PURSE_NAME) {
        Some(purse_key) => purse_key.into_uref().unwrap_or_revert(),
        None => {
            let new_purse = system::create_purse();
            runtime::put_key(PURSE_NAME, new_purse.into());
            new_purse
        }
    };
    runtime::ret(CLValue::from_t(vesting_purse).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn call() {
    let admin: AccountHash = runtime::get_named_arg("admin");
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let cliff_amount: U512 = runtime::get_named_arg("cliff_amount");
    let cliff_timestamp: U512 = runtime::get_named_arg("cliff_timestamp");
    let drip_duration: U512 = runtime::get_named_arg("drip_duration");
    let drip_amount: U512 = runtime::get_named_arg("drip_amount");
    let total_amount: U512 = runtime::get_named_arg("total_amount");
    let admin_release_duration: U512 = runtime::get_named_arg("admin_release_duration");

    let entry_points = get_entry_points();
    let named_keys = {
        let mut nk = NamedKeys::new();
        nk.insert(ADMIN.to_string(), storage::new_uref(admin).into());
        nk.insert(RECIPIENT.to_string(), storage::new_uref(recipient).into());
        nk.insert(
            CLIFF_AMOUNT.to_string(),
            storage::new_uref(cliff_amount).into(),
        );
        nk.insert(
            CLIFF_TIMESTAMP.to_string(),
            storage::new_uref(cliff_timestamp).into(),
        );
        nk.insert(
            DRIP_AMOUNT.to_string(),
            storage::new_uref(drip_amount).into(),
        );
        nk.insert(
            DRIP_DURATION.to_string(),
            storage::new_uref(drip_duration).into(),
        );
        nk.insert(
            TOTAL_AMOUNT.to_string(),
            storage::new_uref(total_amount).into(),
        );
        nk.insert(
            ADMIN_RELEASE_DURATION.to_string(),
            storage::new_uref(admin_release_duration).into(),
        );
        nk.insert(
            LAST_PAUSE_TIMESTAMP.to_string(),
            storage::new_uref(Time::zero()).into(),
        );
        nk.insert(
            ON_PAUSE_DURATION.to_string(),
            storage::new_uref(Time::zero()).into(),
        );
        nk.insert(PAUSE_FLAG.to_string(), storage::new_uref(false).into());
        nk.insert(
            RELEASED_AMOUNT.to_string(),
            storage::new_uref(Amount::zero()).into(),
        );
        nk
    };
    let (contract_hash, _version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(String::from("vesting_contract_package_hash")),
        None,
    );

    runtime::put_key("vesting_contract", contract_hash.into());
    runtime::put_key(
        "vesting_contract_hash",
        storage::new_uref(contract_hash).into(),
    );
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        "get_deposit_purse",
        vec![],
        CLType::Option(Box::new(CLType::URef)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "pause",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "unpause",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "withdraw",
        vec![Parameter::new("amount", U512::cl_type())],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "admin_release",
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points
}

fn verify_admin_account() {
    let admin: AccountHash = get_key(ADMIN);
    let caller = runtime::get_caller();
    if admin != caller {
        runtime::revert(Error::NotTheAdminAccount);
    }
}

fn verify_recipient_account() {
    let recipient: AccountHash = get_key(RECIPIENT);
    let caller = runtime::get_caller();
    if recipient != caller {
        runtime::revert(Error::NotTheRecipientAccount);
    }
}

fn available_amount() -> U512 {
    let current_timestamp = current_timestamp();
    let cliff_timestamp: U512 = get_key(CLIFF_TIMESTAMP);
    let last_pause_timestamp: U512 = get_key(LAST_PAUSE_TIMESTAMP);
    let is_paused: bool = get_key(PAUSE_FLAG);
    let on_pause_duration: U512 = get_key(ON_PAUSE_DURATION);
    let total_paused_duration = on_pause_duration
        + if is_paused {
            current_timestamp - last_pause_timestamp
        } else {
            Time::zero()
        };
    let cliff_timestamp_adjusted = cliff_timestamp + total_paused_duration;
    if current_timestamp < cliff_timestamp_adjusted {
        Amount::zero()
    } else {
        let drip_duration: U512 = get_key(DRIP_DURATION);
        let time_diff: Time = current_timestamp - cliff_timestamp_adjusted;
        let available_drips = if drip_duration == Time::zero() {
            Amount::zero()
        } else {
            time_diff / drip_duration
        };
        let total_amount: U512 = get_key(TOTAL_AMOUNT);
        let drip_amount: U512 = get_key(DRIP_AMOUNT);
        let released_amount: U512 = get_key(RELEASED_AMOUNT);
        let mut counter: U512 = get_key(CLIFF_AMOUNT);
        counter += drip_amount * available_drips;
        counter = cmp::min(counter, total_amount);
        counter - released_amount
    }
}

fn current_timestamp() -> U512 {
    let time: u64 = runtime::get_blocktime().into();
    time.into()
}

pub fn get_key<T: FromBytes + CLTyped>(name: &str) -> T {
    let key = runtime::get_key(name)
        .unwrap_or_revert_with(Error::MissingKey)
        .try_into()
        .unwrap_or_revert_with(Error::UnexpectedType);
    storage::read(key)
        .unwrap_or_revert_with(Error::MissingKey)
        .unwrap_or_revert_with(Error::UnexpectedType)
}

pub fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
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

pub fn transfer_out_clx_to_caller(purse_name: &str, amount: U512) {
    let key = runtime::get_key(purse_name).unwrap_or_revert_with(Error::LocalPurseKeyMissing);
    let local_purse: URef = key.into_uref().unwrap_or_revert_with(Error::UnexpectedType);

    system::transfer_from_purse_to_account(local_purse, runtime::get_caller(), amount, None)
        .unwrap_or_revert_with(Error::PurseTransferErr);
}
