use crate::vesting::{
    account::{ADMIN, ALI, BOB},
    Sender, VestingConfig, VestingContract,
};

#[test]
fn test_vesting_deploy() {
    let vesting = VestingContract::deployed();
    let config: VestingConfig = Default::default();
    let amount = vesting.get_total_amount();
    assert_eq!(amount, config.total_amount.as_u64());
}

#[test]
fn test_withdraw() {
    let config: VestingConfig = Default::default();
    let mut vesting = VestingContract::deployed();
    vesting.set_block_time(config.cliff_timestamp.as_u64());
    vesting.withdraw(Sender(ALI), 1);
    let amount = vesting.get_released_amount();
    assert_eq!(amount, 1);
}

#[test]
#[should_panic]
fn test_withdraw_incorrect_recepient() {
    let mut vesting = VestingContract::deployed();
    let config: VestingConfig = Default::default();
    vesting.set_block_time(config.cliff_timestamp.as_u64());
    vesting.withdraw(Sender(BOB), 1);
}

#[test]
fn test_pause_by_admin() {
    let mut vesting = VestingContract::deployed();
    vesting.pause(Sender(ADMIN));
    let status = vesting.get_pause_status();
    assert!(status, "The contract is not paused");
}

#[test]
fn test_unpause_by_admin() {
    let mut vesting = VestingContract::deployed();
    vesting.pause(Sender(ADMIN));
    vesting.unpause(Sender(ADMIN));
    let status = vesting.get_pause_status();
    assert!(!status, "The contract is still paused");
}

#[test]
#[should_panic]
fn test_pause_by_not_admin() {
    let mut vesting = VestingContract::deployed();
    vesting.pause(Sender(ALI));
}

#[test]
#[should_panic]
fn test_unpause_by_not_admin() {
    let mut vesting = VestingContract::deployed();
    vesting.pause(Sender(ADMIN));
    vesting.unpause(Sender(ALI));
}

#[test]
fn test_admin_release() {
    let config: VestingConfig = Default::default();
    let mut vesting = VestingContract::deployed();
    vesting.pause(Sender(ADMIN));
    vesting.set_block_time(config.admin_release_duration.as_u64());
    vesting.admin_release(Sender(ADMIN));
    let amount = vesting.get_released_amount();
    assert_eq!(amount, config.total_amount.as_u64());
}

#[test]
#[should_panic]
fn test_admin_release_by_not_admin() {
    let mut vesting = VestingContract::deployed();
    let config: VestingConfig = Default::default();
    vesting.pause(Sender(ADMIN));
    vesting.set_block_time(config.admin_release_duration.as_u64());
    vesting.admin_release(Sender(ALI));
}

#[test]
#[should_panic]
fn test_not_possible_to_call_init() {
    let mut vesting = VestingContract::deployed();
    let config: VestingConfig = Default::default();
    vesting.init(Sender(ADMIN), ADMIN, ALI, config);
}
