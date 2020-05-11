use alloc::string::String;

use casperlabs_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casperlabs_types::{account::PublicKey, bytesrepr::FromBytes, CLTyped, ContractRef, URef, U512};

use crate::error::Error;

pub const DEPLOY: &str = "deploy";
pub const INIT: &str = "init";
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
pub enum Api {
    Deploy(String, PublicKey, PublicKey, VestingConfig),
    Init(PublicKey, PublicKey, VestingConfig),
    Pause,
    Unpause,
    WithdrawProxy(U512),
    Withdraw(URef, U512),
    AdminReleaseProxy,
    AdminRelease(URef),
}

fn get_arg<T: CLTyped + FromBytes>(i: u32) -> T {
    runtime::get_arg(i)
        .unwrap_or_revert_with(Error::missing_argument(i))
        .unwrap_or_revert_with(Error::invalid_argument(i))
}

impl Api {
    pub fn from_args() -> Api {
        Self::from_args_with_shift(0)
    }

    pub fn from_args_in_proxy() -> Api {
        Self::from_args_with_shift(1)
    }

    fn from_args_with_shift(arg_shift: u32) -> Api {
        let method_name: String = get_arg(arg_shift);
        match method_name.as_str() {
            DEPLOY => {
                let vesting_contract_name = get_arg(arg_shift + 1);
                let admin = get_arg(arg_shift + 2);
                let recipient = get_arg(arg_shift + 3);
                let vesting_conf = Self::get_vesting_config(arg_shift + 4);
                Api::Deploy(vesting_contract_name, admin, recipient, vesting_conf)
            }
            INIT => {
                let admin = get_arg(arg_shift + 1);
                let recipient = get_arg(arg_shift + 2);
                let vesting_conf = Self::get_vesting_config(arg_shift + 3);
                Api::Init(admin, recipient, vesting_conf)
            }
            PAUSE => Api::Pause,
            UNPAUSE => Api::Unpause,
            WITHDRAW_PROXY => {
                let amount = get_arg(arg_shift + 1);
                Api::WithdrawProxy(amount)
            }
            WITHDRAW => {
                let purse = get_arg(arg_shift + 1);
                let amount = get_arg(arg_shift + 2);
                Api::Withdraw(purse, amount)
            }
            ADMIN_RELEASE => {
                let purse: URef = get_arg(arg_shift + 1);
                Api::AdminRelease(purse)
            }
            ADMIN_RELEASE_PROXY => Api::AdminReleaseProxy,
            _ => runtime::revert(Error::UnknownApiCommand),
        }
    }

    pub fn destination_contract() -> ContractRef {
        ContractRef::Hash(get_arg(0))
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
}
