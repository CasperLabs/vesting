use crate::{
    api::{self, Api},
    error::Error,
};
use casperlabs_contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};

#[no_mangle]
pub extern "C" fn vesting_proxy() {
    let vault_ref = Api::destination_contract();
    match Api::from_args_in_proxy() {
        Api::Pause => {
            runtime::call_contract::<_, ()>(vault_ref, (api::PAUSE,));
        }
        Api::Unpause => {
            runtime::call_contract::<_, ()>(vault_ref, (api::UNPAUSE,));
        }
        Api::WithdrawProxy(amount) => {
        //    let new_purse = system::create_purse();
        //    runtime::call_contract::<_, ()>(vault_ref, (api::WITHDRAW, new_purse, amount));
        //   let main_purse = account::get_main_purse();
        //    system::transfer_from_purse_to_purse(new_purse, main_purse, amount)
        //        .unwrap_or_revert_with(Error::PurseTransferError);
        } 
        Api::AdminReleaseProxy => {
            let new_purse = system::create_purse();
            runtime::call_contract::<_, ()>(vault_ref, (api::ADMIN_RELEASE, new_purse));
            let main_purse = account::get_main_purse();
            let amount =
                system::get_balance(new_purse).unwrap_or_revert_with(Error::PurseBalanceCheckError);
            system::transfer_from_purse_to_purse(new_purse, main_purse, amount)
                .unwrap_or_revert_with(Error::PurseTransferError);
        }
        _ => runtime::revert(Error::UnknownProxyCommand),
    }
}
