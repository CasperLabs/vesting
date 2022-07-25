#![no_main]

extern crate alloc;
mod error;
use contract::{
    contract_api::{account, runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use core::cmp;
use error::Error;
use std::{collections::BTreeMap, convert::TryInto};
use types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    contracts::EntryPoints,
    CLTyped, EntryPoint, EntryPointAccess, EntryPointType, Group, Key, Parameter, URef, U512,
};

pub type Time = U512;
pub type Amount = U512;

mod key {
    pub const PURSE_NAME: &str = "vesting_main_purse";
    pub const ADMIN: &str = "admin_account";
    pub const RECIPIENT: &str = "recipient_account";
    pub const CLIFF_TIMESTAMP: &str = "cliff_timestamp";
    pub const CLIFF_AMOUNT: &str = "cliff_amount";
    pub const DRIP_DURATION: &str = "drip_duration";
    pub const DRIP_AMOUNT: &str = "drip_amount";
    pub const TOTAL_AMOUNT: &str = "total_amount";
    pub const RELEASED_AMOUNT: &str = "released_amount";
    pub const ADMIN_RELEASE_DURATION: &str = "admin_release_duration";
    pub const PAUSE_FLAG: &str = "is_paused";
    pub const ON_PAUSE_DURATION: &str = "on_pause_duration";
    pub const LAST_PAUSE_TIMESTAMP: &str = "last_pause_timestamp";
}

mod arg {
    pub const TOTAL_AMOUNT: &str = "total_amount";
    pub const AMOUNT: &str = "amount";
}

#[no_mangle]
pub extern "C" fn constructor() {
    let admin: AccountHash = runtime::get_named_arg("admin");
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let cliff_timestamp: U512 = runtime::get_named_arg("cliff_timestamp");
    let cliff_amount: U512 = runtime::get_named_arg("cliff_amount");
    let drip_duration: U512 = runtime::get_named_arg("drip_duration");
    let drip_amount: U512 = runtime::get_named_arg("drip_amount");
    let total_amount: U512 = runtime::get_named_arg("total_amount");
    let admin_release_duration: U512 = runtime::get_named_arg("admin_release_duration");

    set_admin_account(admin);
    set_recipient_account(recipient);
    set_key(key::CLIFF_TIMESTAMP, cliff_timestamp);
    set_key(key::CLIFF_AMOUNT, cliff_amount);
    set_key(key::DRIP_DURATION, drip_duration);
    set_key(key::DRIP_AMOUNT, drip_amount);
    set_key(key::TOTAL_AMOUNT, total_amount);
    set_key(key::ADMIN_RELEASE_DURATION, admin_release_duration);
}

#[no_mangle]
pub extern "C" fn pause() {
    verify_admin_account();

    let is_paused: bool = get_key(key::PAUSE_FLAG);

    if !is_paused {
        set_key(key::LAST_PAUSE_TIMESTAMP, current_timestamp());
        set_key(key::PAUSE_FLAG, true);
    } else {
        runtime::revert(Error::AlreadyPaused);
    }
}

#[no_mangle]
pub extern "C" fn unpause() {
    verify_admin_account();

    let is_paused: bool = get_key(key::PAUSE_FLAG);

    if is_paused {
        let on_pause_duration: U512 = get_key(key::ON_PAUSE_DURATION);
        let last_pause_timestamp: U512 = get_key(key::LAST_PAUSE_TIMESTAMP);
        let elapsed_timestamp: U512 = current_timestamp() - last_pause_timestamp;
        set_key(
            key::ON_PAUSE_DURATION,
            on_pause_duration + elapsed_timestamp,
        );
        set_key(key::PAUSE_FLAG, false);
    } else {
        runtime::revert(Error::AlreadyUnpaused);
    }
}

#[no_mangle]
pub extern "C" fn withdraw() {
    verify_recipient_account();

    let amount: U512 = runtime::get_named_arg(arg::AMOUNT);
    let available_amount = available_amount();
    if available_amount < amount {
        runtime::revert(Error::NotEnoughBalance);
    } else {
        let released_amount: U512 = get_key(key::RELEASED_AMOUNT);
        set_key(key::RELEASED_AMOUNT, released_amount + amount);
    }
}

#[no_mangle]
pub extern "C" fn admin_release() {
    verify_admin_account();

    let is_paused: bool = get_key(key::PAUSE_FLAG);
    if !is_paused {
        runtime::revert(Error::NotPaused);
    }
    let last_pause_timestamp: U512 = get_key(key::LAST_PAUSE_TIMESTAMP);
    let since_last_pause = current_timestamp() - last_pause_timestamp;
    let required_wait_duration: U512 = get_key(key::ADMIN_RELEASE_DURATION);
    if since_last_pause < required_wait_duration {
        runtime::revert(Error::NotEnoughTimeElapsed);
    }
    let total_amount: U512 = get_key(key::TOTAL_AMOUNT);
    let released_amount: U512 = get_key(key::RELEASED_AMOUNT);
    if total_amount == released_amount {
        runtime::revert(Error::NothingToWithdraw);
    }
    let amount_to_withdraw = total_amount - released_amount;
    set_key(key::TOTAL_AMOUNT, amount_to_withdraw);
    transfer_out_clx_to_caller(key::PURSE_NAME, amount_to_withdraw);
}

#[no_mangle]
pub extern "C" fn call() {
    let (contract_package_hash, _) = storage::create_contract_package_at_hash();
    let entry_points = get_entry_points();
    let vesting_keys = init();
    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, entry_points, Default::default());
    runtime::put_key("vesting_contract_hash", contract_hash.into());
    let contract_hash_pack = storage::new_uref(contract_hash);
    runtime::put_key("vesting_contract_hash_wrapped", contract_hash_pack.into())
}

fn init() -> BTreeMap<String, Key> {
    let main_purse = account::get_main_purse();
    let vesting_purse = system::create_purse();
    let total_amount: U512 = runtime::get_named_arg(arg::TOTAL_AMOUNT);
    system::transfer_from_purse_to_purse(main_purse, vesting_purse, total_amount, None)
        .unwrap_or_revert_with(Error::PurseTransferError);
    let mut vesting_keys: BTreeMap<String, Key> = BTreeMap::new();
    vesting_keys.insert(String::from(key::PURSE_NAME), vesting_purse.into());
    vesting_keys
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        "constructor",
        vec![
            Parameter::new("admin", Key::cl_type()),
            Parameter::new("recipient", Key::cl_type()),
            Parameter::new("cliff_timestamp", U512::cl_type()),
            Parameter::new("cliff_amount", U512::cl_type()),
            Parameter::new("drip_duration", U512::cl_type()),
            Parameter::new("drip_amount", U512::cl_type()),
            Parameter::new("total_amount", U512::cl_type()),
            Parameter::new("admin_release_duration", U512::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Groups(vec![Group::new("constructor")]),
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

fn set_recipient_account(recipient: AccountHash) {
    set_key(key::RECIPIENT, recipient);
}

fn set_admin_account(admin: AccountHash) {
    set_key(key::ADMIN, admin);
}

fn verify_admin_account() {
    let admin: AccountHash = get_key(key::ADMIN);
    let caller = runtime::get_caller();
    if admin != caller {
        runtime::revert(Error::NotTheAdminAccount);
    }
}

fn verify_recipient_account() {
    let recipient: AccountHash = get_key(key::RECIPIENT);
    let caller = runtime::get_caller();
    if recipient != caller {
        runtime::revert(Error::NotTheRecipientAccount);
    }
}

fn available_amount() -> U512 {
    let current_timestamp = current_timestamp();
    let cliff_timestamp: U512 = get_key(key::CLIFF_TIMESTAMP);
    let last_pause_timestamp: U512 = get_key(key::LAST_PAUSE_TIMESTAMP);
    let is_paused: bool = get_key(key::PAUSE_FLAG);
    let on_pause_duration: U512 = get_key(key::ON_PAUSE_DURATION);
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
        let drip_duration: U512 = get_key(key::DRIP_DURATION);
        let time_diff: Time = current_timestamp - cliff_timestamp_adjusted;
        let available_drips = if drip_duration == Time::zero() {
            Amount::zero()
        } else {
            time_diff / drip_duration
        };
        let total_amount: U512 = get_key(key::TOTAL_AMOUNT);
        let drip_amount: U512 = get_key(key::DRIP_AMOUNT);
        let released_amount: U512 = get_key(key::RELEASED_AMOUNT);
        let mut counter: U512 = get_key(key::CLIFF_AMOUNT);
        counter = counter + drip_amount * available_drips;
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
        .unwrap_or_revert_with(Error::PurseTransferError);
}
