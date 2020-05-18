#![no_std]
#![no_main]

extern crate alloc;

mod deployer;
mod error;
mod indirect;
mod input_parser;
mod vesting;

#[no_mangle]
pub extern "C" fn call() {
    deployer::deploy();
}
