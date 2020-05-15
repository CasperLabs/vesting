use alloc::{collections::BTreeMap, string::String};

use crate::{
    input_parser::{self, Input, VestingConfig},
    error::Error,
};
use casperlabs_contract::{
    contract_api::{account, runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casperlabs_types::{account::PublicKey, ContractRef, Key, URef};

use crate::vesting::PURSE_NAME;

const VESTING_CONTRACT_NAME: &str = "vesting";
const VESTING_PROXY_CONTRACT_NAME: &str = "vesting_proxy";

pub fn deploy() {
    match input_parser::from_args() {
        Input::Deploy(name, admin, recipient, vesting_config) => {
            deploy_vesting_contract(&name, admin, recipient, vesting_config);
            deploy_proxy();
        }
        _ => runtime::revert(Error::UnknownDeployCommand),
    }
}

fn deploy_vesting_contract(
    name: &str,
    admin: PublicKey,
    recipient: PublicKey,
    vesting_config: VestingConfig,
) {
    // Create a smart contract purse.
    let main_purse = account::get_main_purse();
    let vesting_purse = system::create_purse();
    system::transfer_from_purse_to_purse(main_purse, vesting_purse, vesting_config.total_amount)
        .unwrap_or_revert_with(Error::PurseTransferError);
    let mut vesting_keys: BTreeMap<String, Key> = BTreeMap::new();
    vesting_keys.insert(String::from(PURSE_NAME), vesting_purse.into());

    // Create vesting instance.
    let vesting_ref: ContractRef =
        storage::store_function_at_hash(VESTING_CONTRACT_NAME, vesting_keys);

    // Initialize vesting contract.
    runtime::call_contract::<_, ()>(
        vesting_ref.clone(),
        (
            input_parser::DEPLOY,
            name,
            admin,
            recipient,
            vesting_config.cliff_time,
            vesting_config.cliff_amount,
            vesting_config.drip_period,
            vesting_config.drip_amount,
            vesting_config.total_amount,
            vesting_config.admin_release_period,
        ),
    );

    let vesting_key: Key = vesting_ref.into();

    // Save it under a new URef.
    let vesting_uref: URef = storage::new_uref(vesting_key);

    // Save URef under readable name.
    runtime::put_key(&name, vesting_uref.into());
}

fn deploy_proxy() {
    // Create proxy instance.
    let proxy_ref: ContractRef =
        storage::store_function_at_hash(VESTING_PROXY_CONTRACT_NAME, Default::default());

    let proxy_key: Key = proxy_ref.into();

    // Save it under a new URef.
    let proxy_uref: URef = storage::new_uref(proxy_key);

    // Save URef under readable name.
    runtime::put_key(VESTING_PROXY_CONTRACT_NAME, proxy_uref.into());
}
