use crate::error::Error;
use alloc::string::String;
use casperlabs_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casperlabs_types::{
    account::PublicKey,
    bytesrepr::{Error as ApiError, FromBytes},
    CLTyped, ContractRef, URef, U512,
};

pub const DEPLOY: &str = "deploy";
pub const PAUSE: &str = "pause";
pub const UNPAUSE: &str = "unpause";
pub const WITHDRAW: &str = "withdraw";
pub const WITHDRAW_PROXY: &str = "withdraw_proxy";
pub const ADMIN_RELEASE: &str = "admin_release";
pub const ADMIN_RELEASE_PROXY: &str = "admin_release_proxy";

pub struct VestingConfig {
    pub cliff_time: U512,
    pub cliff_amount: U512,
    pub drip_period: U512,
    pub drip_amount: U512,
    pub total_amount: U512,
    pub admin_release_period: U512,
}

#[allow(clippy::large_enum_variant)]
pub enum Input {
    Deploy(String, PublicKey, PublicKey, VestingConfig),
    Pause,
    Unpause,
    WithdrawProxy(U512),
    Withdraw(URef, U512),
    AdminReleaseProxy,
    AdminRelease(URef),
}

pub fn from_args() -> Input {
    let method: String = method_name();
    match method.as_str() {
        DEPLOY => Input::Deploy(get_arg(1), get_arg(2), get_arg(3), get_vesting_config(4)),
        PAUSE => Input::Pause,
        UNPAUSE => Input::Unpause,
        WITHDRAW_PROXY => Input::WithdrawProxy(get_arg(1)),
        WITHDRAW => Input::Withdraw(get_arg(1), get_arg(2)),
        ADMIN_RELEASE => Input::AdminRelease(get_arg(1)),
        ADMIN_RELEASE_PROXY => Input::AdminReleaseProxy,
        _ => runtime::revert(Error::UnknownApiCommand),
    }
}

pub fn destination_contract() -> ContractRef {
    let (_, hash): (String, [u8; 32]) = get_arg(0);
    ContractRef::Hash(hash)
}

fn get_arg<T: CLTyped + FromBytes>(i: u32) -> T {
    runtime::get_arg(i)
        .unwrap_or_revert_with(Error::missing_argument(i))
        .unwrap_or_revert_with(Error::invalid_argument(i))
}

fn method_name() -> String {
    let maybe_argument: Result<String, ApiError> =
        runtime::get_arg(0).unwrap_or_revert_with(Error::missing_argument(0));
    match maybe_argument {
        Ok(method) => method,
        Err(_) => {
            let (method, _): (String, [u8; 32]) = get_arg(0);
            method
        }
    }
}

fn get_vesting_config(arg_shift: u32) -> VestingConfig {
    VestingConfig {
        cliff_time: get_arg(arg_shift),
        cliff_amount: get_arg(arg_shift + 1),
        drip_period: get_arg(arg_shift + 2),
        drip_amount: get_arg(arg_shift + 3),
        total_amount: get_arg(arg_shift + 4),
        admin_release_period: get_arg(arg_shift + 5),
    }
}
