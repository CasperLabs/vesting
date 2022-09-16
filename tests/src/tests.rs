use crate::vesting::{Vesting, VestingConfig};

#[test]
fn test_vesting_deploy() {
    let vesting = Vesting::deploy();
    let config: VestingConfig = Default::default();
    let amount = vesting.get_total_amount();
    assert_eq!(amount, config.total_amount.as_u64());
}

#[test]
fn test_withdraw() {
    let mut vesting = Vesting::deploy();
    let config: VestingConfig = Default::default();
    vesting.set_block_time(config.cliff_timestamp.as_u64());
    vesting.withdraw(vesting.ali_account.1, 1);
    let amount = vesting.get_released_amount();
    assert_eq!(amount, 1);
}

#[test]
#[should_panic]
fn test_withdraw_incorrect_recepient() {
    let mut vesting = Vesting::deploy();
    let config: VestingConfig = Default::default();
    vesting.set_block_time(config.cliff_timestamp.as_u64());
    vesting.withdraw(vesting.bob_account.1, 1);
}

#[test]
fn test_pause_by_admin() {
    let mut vesting = Vesting::deploy();
    vesting.pause(vesting.admin_account.1);
    let status = vesting.get_pause_status();
    assert!(status, "The contract is not paused");
}

#[test]
fn test_unpause_by_admin() {
    let mut vesting = Vesting::deploy();
    vesting.pause(vesting.admin_account.1);
    vesting.unpause(vesting.admin_account.1);
    let status = vesting.get_pause_status();
    assert!(!status, "The contract is still paused");
}

#[test]
#[should_panic]
fn test_pause_by_not_admin() {
    let mut vesting = Vesting::deploy();
    vesting.pause(vesting.ali_account.1);
}

#[test]
#[should_panic]
fn test_unpause_by_not_admin() {
    let mut vesting = Vesting::deploy();
    vesting.pause(vesting.admin_account.1);
    vesting.unpause(vesting.ali_account.1);
}

#[test]
fn test_admin_release() {
    let config: VestingConfig = Default::default();
    let mut vesting = Vesting::deploy();
    vesting.pause(vesting.admin_account.1);
    vesting.set_block_time(config.admin_release_duration.as_u64());
    vesting.admin_release(vesting.admin_account.1);
    let amount = vesting.get_released_amount();
    assert_eq!(amount, config.total_amount.as_u64());
}

#[test]
#[should_panic]
fn test_admin_release_by_not_admin() {
    let mut vesting = Vesting::deploy();
    let config: VestingConfig = Default::default();
    vesting.pause(vesting.admin_account.1);
    vesting.set_block_time(config.admin_release_duration.as_u64());
    vesting.admin_release(vesting.ali_account.1);
}
