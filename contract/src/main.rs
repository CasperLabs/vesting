#![no_std]
#![no_main]

extern crate alloc;

mod deployer;
mod error;
mod input_parser;
mod indirect;
mod vesting;

#[no_mangle]
pub extern "C" fn call() {
    deployer::deploy();
}
