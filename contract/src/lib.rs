extern crate alloc;

mod contract;
mod error;
mod utils;
mod vesting;

#[no_mangle]
fn call() {
    contract::deploy();
}
