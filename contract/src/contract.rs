use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};

use casperlabs_contract::{
    contract_api::{account, runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casperlabs_types::{
    account::PublicKey,
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    runtime_args, CLType, CLTyped, Group, Key, Parameter, RuntimeArgs, URef, U512,
};

use logic::{VestingError, VestingTrait};

use crate::error::Error;
use crate::utils;
use crate::vesting::VestingContract;

mod key {
    pub const VESTING_CONTRACT: &str = "vesting_contract";
    pub const VESTING_CONTRACT_HASH: &str = "vesting_contract_hash";
    pub const PURSE_NAME: &str = "vesting_main_purse";
    pub const ADMIN: &str = "admin_account";
    pub const RECIPIENT: &str = "recipient_account";
    pub const INIT_GROUP: &str = "init_group";
}

mod method {
    pub const INIT: &str = "init";
    pub const PAUSE: &str = "pause";
    pub const UNPAUSE: &str = "unpause";
    pub const WITHDRAW: &str = "withdraw";
    pub const ADMIN_RELEASE: &str = "admin_release";
}

mod arg {
    pub const ADMIN: &str = "admin";
    pub const RECIPIENT: &str = "recipient";
    pub const CLIFF_TIMESTAMP: &str = "cliff_timestamp";
    pub const CLIFF_AMOUNT: &str = "cliff_amount";
    pub const DRIP_DURATION: &str = "drip_duration";
    pub const DRIP_AMOUNT: &str = "drip_amount";
    pub const TOTAL_AMOUNT: &str = "total_amount";
    pub const ADMIN_RELEASE_DURATION: &str = "admin_release_duration";
    pub const AMOUNT: &str = "amount";
}

pub struct VestingConfig {
    pub cliff_timestamp: U512,
    pub cliff_amount: U512,
    pub drip_duration: U512,
    pub drip_amount: U512,
    pub total_amount: U512,
    pub admin_release_duration: U512,
}

#[no_mangle]
fn init() {
    let mut vault = VestingContract;
    let admin: PublicKey = runtime::get_named_arg(arg::ADMIN);
    let recipient: PublicKey = runtime::get_named_arg(arg::RECIPIENT);
    set_admin_account(admin);
    set_recipient_account(recipient);
    let vesting_config = get_vesting_config_from_args();
    vault.init(
        vesting_config.cliff_timestamp,
        vesting_config.cliff_amount,
        vesting_config.drip_duration,
        vesting_config.drip_amount,
        vesting_config.total_amount,
        vesting_config.admin_release_duration,
    );
}

#[no_mangle]
fn pause() {
    verify_admin_account();
    let mut vault = VestingContract;
    match vault.pause() {
        Ok(()) => {}
        Err(VestingError::AlreadyPaused) => runtime::revert(Error::AlreadyPaused),
        _ => runtime::revert(Error::UnexpectedVestingError),
    }
}

#[no_mangle]
fn unpause() {
    verify_admin_account();
    let mut vault = VestingContract;
    match vault.unpause() {
        Ok(()) => {}
        Err(VestingError::AlreadyUnpaused) => runtime::revert(Error::AlreadyUnpaused),
        _ => runtime::revert(Error::UnexpectedVestingError),
    }
}

#[no_mangle]
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

#[no_mangle]
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

pub fn deploy() {
    let (contract_package_hash, _) = storage::create_contract_package_at_hash();

    // Create the init group.
    // Note that the init_access_uref is never saved, so it's only possible to
    // call the init method during this call.
    let _init_access_uref: URef = storage::create_contract_user_group(
        contract_package_hash,
        key::INIT_GROUP,
        1,
        BTreeSet::new(),
    )
    .unwrap_or_revert()
    .pop()
    .unwrap_or_revert();
    let init_group = Group::new(key::INIT_GROUP);

    // Define entry points.
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        method::INIT.to_string(),
        vec![
            Parameter::new(arg::ADMIN, PublicKey::cl_type()),
            Parameter::new(arg::RECIPIENT, PublicKey::cl_type()),
            Parameter::new(arg::CLIFF_TIMESTAMP, CLType::U512),
            Parameter::new(arg::CLIFF_AMOUNT, CLType::U512),
            Parameter::new(arg::DRIP_DURATION, CLType::U512),
            Parameter::new(arg::DRIP_AMOUNT, CLType::U512),
            Parameter::new(arg::TOTAL_AMOUNT, CLType::U512),
            Parameter::new(arg::ADMIN_RELEASE_DURATION, CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Groups(vec![init_group]),
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        method::PAUSE.to_string(),
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        method::UNPAUSE.to_string(),
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        method::WITHDRAW.to_string(),
        vec![Parameter::new(arg::AMOUNT, CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        method::ADMIN_RELEASE.to_string(),
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Read arguments
    let admin: PublicKey = runtime::get_named_arg(arg::ADMIN);
    let recipient: PublicKey = runtime::get_named_arg(arg::RECIPIENT);
    let vesting_config = get_vesting_config_from_args();

    // // Prepare contract's purse.
    let main_purse = account::get_main_purse();
    let vesting_purse = system::create_purse();
    system::transfer_from_purse_to_purse(main_purse, vesting_purse, vesting_config.total_amount)
        .unwrap_or_revert_with(Error::PurseTransferError);
    let mut vesting_keys: BTreeMap<String, Key> = BTreeMap::new();
    vesting_keys.insert(String::from(key::PURSE_NAME), vesting_purse.into());

    // Deploy smart contract.
    let contract_hash =
        storage::add_contract_version(contract_package_hash, entry_points, vesting_keys);

    // // Save contract under Account's keys.
    runtime::put_key(key::VESTING_CONTRACT, contract_hash.into());
    let contract_hash_pack = storage::new_uref(contract_hash);
    runtime::put_key(key::VESTING_CONTRACT_HASH, contract_hash_pack.into());

    // Call init method.
    runtime::call_contract::<()>(
        contract_hash,
        method::INIT,
        runtime_args! {
            arg::ADMIN => admin,
            arg::RECIPIENT => recipient,
            arg::CLIFF_TIMESTAMP => vesting_config.cliff_timestamp,
            arg::CLIFF_AMOUNT => vesting_config.cliff_amount,
            arg::DRIP_DURATION => vesting_config.drip_duration,
            arg::DRIP_AMOUNT => vesting_config.drip_amount,
            arg::TOTAL_AMOUNT => vesting_config.total_amount,
            arg::ADMIN_RELEASE_DURATION => vesting_config.admin_release_duration
        },
    );
}

fn get_vesting_config_from_args() -> VestingConfig {
    VestingConfig {
        cliff_timestamp: runtime::get_named_arg(arg::CLIFF_TIMESTAMP),
        cliff_amount: runtime::get_named_arg(arg::CLIFF_AMOUNT),
        drip_duration: runtime::get_named_arg(arg::DRIP_DURATION),
        drip_amount: runtime::get_named_arg(arg::DRIP_AMOUNT),
        total_amount: runtime::get_named_arg(arg::TOTAL_AMOUNT),
        admin_release_duration: runtime::get_named_arg(arg::ADMIN_RELEASE_DURATION),
    }
}

fn set_recipient_account(recipient: PublicKey) {
    utils::set_key(key::RECIPIENT, recipient);
}

fn set_admin_account(admin: PublicKey) {
    utils::set_key(key::ADMIN, admin);
}

fn recipient_account() -> PublicKey {
    utils::get_key(key::RECIPIENT)
}

fn admin_account() -> PublicKey {
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
