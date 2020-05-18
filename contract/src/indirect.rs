use casperlabs_contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};

use crate::{
    error::Error,
    input_parser::{self, Input},
};

#[no_mangle]
pub extern "C" fn vesting_proxy() {
    let vault_ref = input_parser::destination_contract();
    match input_parser::from_args() {
        Input::Pause => {
            runtime::call_contract::<_, ()>(vault_ref, (input_parser::PAUSE,));
        }
        Input::Unpause => {
            runtime::call_contract::<_, ()>(vault_ref, (input_parser::UNPAUSE,));
        }
        Input::WithdrawProxy(amount) => {
            let new_purse = system::create_purse();
            runtime::call_contract::<_, ()>(vault_ref, (input_parser::WITHDRAW, new_purse, amount));
            let main_purse = account::get_main_purse();
            system::transfer_from_purse_to_purse(new_purse, main_purse, amount)
                .unwrap_or_revert_with(Error::PurseTransferError);
        }
        Input::AdminReleaseProxy => {
            let new_purse = system::create_purse();
            runtime::call_contract::<_, ()>(vault_ref, (input_parser::ADMIN_RELEASE, new_purse));
            let main_purse = account::get_main_purse();
            let amount =
                system::get_balance(new_purse).unwrap_or_revert_with(Error::PurseBalanceCheckError);
            system::transfer_from_purse_to_purse(new_purse, main_purse, amount)
                .unwrap_or_revert_with(Error::PurseTransferError);
        }
        _ => runtime::revert(Error::UnknownProxyCommand),
    }
}
