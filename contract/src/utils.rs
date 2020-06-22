use core::convert::TryInto;

use casperlabs_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casperlabs_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped, URef, U512,
};

use crate::error::Error;

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

    system::transfer_from_purse_to_account(local_purse, runtime::get_caller(), amount)
        .unwrap_or_revert_with(Error::PurseTransferError);
}
