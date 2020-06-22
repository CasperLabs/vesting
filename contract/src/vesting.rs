use casperlabs_contract::contract_api::runtime;
use casperlabs_types::U512;

use logic::VestingTrait;

use crate::utils;

type Amount = U512;
type Time = U512;

pub struct VestingContract;

impl VestingTrait<Amount, Time> for VestingContract {
    fn set_amount(&mut self, name: &str, value: Amount) {
        utils::set_key(name, value)
    }
    fn amount(&self, name: &str) -> Amount {
        utils::get_key(name)
    }
    fn set_time(&mut self, name: &str, value: Time) {
        utils::set_key(name, value);
    }
    fn time(&self, name: &str) -> Time {
        utils::get_key(name)
    }
    fn set_boolean(&mut self, name: &str, value: bool) {
        utils::set_key(name, value)
    }
    fn boolean(&self, name: &str) -> bool {
        utils::get_key(name)
    }
    fn current_timestamp(&self) -> Time {
        let time: u64 = runtime::get_blocktime().into();
        time.into()
    }
}
