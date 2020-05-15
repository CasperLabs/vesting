#![no_std]
#![no_main]

extern crate alloc;

mod input_parser;
mod deployer;
mod error;
mod proxy;
mod vesting;

#[no_mangle]
pub extern "C" fn call() {
    deployer::deploy();
}
