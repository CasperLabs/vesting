use crate::{VestingError, VestingTrait};
extern crate alloc;
use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
};

type Amount = u64;
type Time = u64;

struct Vault {
    amounts: BTreeMap<String, Amount>,
    times: BTreeMap<String, Time>,
    booleans: BTreeMap<String, bool>,
    current_timestamp: Time,
}

impl Vault {
    fn new(conf: VestingConfig) -> Vault {
        let mut vault = Vault {
            amounts: BTreeMap::new(),
            times: BTreeMap::new(),
            booleans: BTreeMap::new(),
            current_timestamp: 0,
        };
        vault.init(
            conf.cliff_timestamp,
            conf.cliff_amount,
            conf.drip_duration,
            conf.drip_amount,
            conf.total_amount,
            conf.admin_release_duration,
        );
        vault
    }

    fn set_current_timestamp(&mut self, new_time: Time) {
        self.current_timestamp = new_time;
    }
}

impl VestingTrait<Amount, Time> for Vault {
    fn set_amount(&mut self, name: &str, value: Amount) {
        self.amounts.insert(name.to_string(), value);
    }
    fn amount(&self, name: &str) -> Amount {
        self.amounts.get(&name.to_string()).cloned().unwrap()
    }
    fn set_time(&mut self, name: &str, value: Time) {
        self.times.insert(name.to_string(), value);
    }
    fn time(&self, name: &str) -> Time {
        self.times.get(&name.to_string()).cloned().unwrap()
    }
    fn set_boolean(&mut self, name: &str, value: bool) {
        self.booleans.insert(name.to_string(), value);
    }
    fn boolean(&self, name: &str) -> bool {
        self.booleans.get(&name.to_string()).cloned().unwrap()
    }
    fn current_timestamp(&self) -> Time {
        self.current_timestamp
    }
}

#[derive(Clone)]
struct VestingConfig {
    cliff_timestamp: Time,
    cliff_amount: Amount,
    drip_duration: Time,
    drip_amount: Amount,
    total_amount: Amount,
    admin_release_duration: Time,
}

impl Default for VestingConfig {
    fn default() -> VestingConfig {
        VestingConfig {
            cliff_timestamp: 10,
            cliff_amount: 2,
            drip_duration: 3,
            drip_amount: 5,
            total_amount: 1000,
            admin_release_duration: 123,
        }
    }
}

#[test]
fn test_pausing() {
    let mut vault = Vault::new(Default::default());
    let first_pause = 2;
    let first_unpause = 5;
    let second_pause = 8;
    let second_unpause = 10;
    assert_eq!(vault.on_pause_duration(), 0);
    vault.set_current_timestamp(first_pause);
    assert_eq!(vault.on_pause_duration(), 0);
    let _ = vault.pause();
    vault.set_current_timestamp(first_unpause);
    let _ = vault.unpause();
    assert_eq!(vault.on_pause_duration(), first_unpause - first_pause);
    vault.set_current_timestamp(second_pause);
    let _ = vault.pause();
    vault.set_current_timestamp(second_unpause);
    let _ = vault.unpause();
    assert_eq!(
        vault.on_pause_duration(),
        first_unpause - first_pause + second_unpause - second_pause
    );
}

#[test]
fn test_pausing_already_paused_error() {
    let mut vault = Vault::new(Default::default());
    let first = vault.pause();
    assert!(first.is_ok());
    let second = vault.pause();
    let expected = VestingError::AlreadyPaused;
    assert_eq!(second.unwrap_err(), expected);
}

#[test]
fn test_pausing_already_unpaused_error() {
    let mut vault = Vault::new(Default::default());
    let result = vault.unpause();
    let expected = VestingError::AlreadyUnpaused;
    assert_eq!(result.unwrap_err(), expected);
}

#[test]
fn test_available_no_pause() {
    let cfg = VestingConfig::default();
    let mut vault = Vault::new(cfg.clone());
    assert_eq!(vault.available_amount(), 0);
    vault.set_current_timestamp(cfg.cliff_timestamp);
    assert_eq!(vault.available_amount(), cfg.cliff_amount);
    let max = (cfg.total_amount - cfg.cliff_amount) / cfg.drip_amount + 1;
    for period in 0..(2 * max) {
        for i in 0..cfg.drip_duration {
            vault.set_current_timestamp(cfg.cliff_timestamp + period * cfg.drip_duration + i);
            if period < max {
                assert_eq!(
                    vault.available_amount(),
                    cfg.cliff_amount + period * cfg.drip_amount
                );
            } else {
                assert_eq!(vault.available_amount(), cfg.total_amount);
            };
        }
    }
}

#[test]
fn test_available_with_pause() {
    let cfg = VestingConfig::default();
    let mut vault = Vault::new(cfg.clone());
    let pause = 2;
    let unpause = 5;
    vault.set_current_timestamp(pause);
    let _ = vault.pause();
    vault.set_current_timestamp(unpause);
    let _ = vault.unpause();
    assert_eq!(vault.available_amount(), 0);
    vault.set_current_timestamp(cfg.cliff_timestamp + unpause - pause);
    assert_eq!(vault.available_amount(), cfg.cliff_amount);
}

#[test]
fn test_withdraw_not_enough_balance_error() {
    let cfg = VestingConfig::default();
    let mut vault = Vault::new(cfg.clone());
    vault.set_current_timestamp(cfg.cliff_timestamp);
    let result = vault.withdraw(cfg.cliff_amount + 1);
    let expected = VestingError::NotEnoughBalance;
    assert_eq!(result.unwrap_err(), expected);
}

#[test]
fn test_withdraw() {
    let cfg = VestingConfig::default();
    let mut vault = Vault::new(cfg.clone());
    let pause = cfg.cliff_timestamp;
    let unpause = cfg.cliff_timestamp + cfg.drip_duration;
    let withdraw_amount = 1;
    vault.set_current_timestamp(pause);
    let result = vault.withdraw(withdraw_amount);
    assert!(result.is_ok());
    assert_eq!(vault.available_amount(), cfg.cliff_amount - withdraw_amount);
    let _ = vault.pause();
    vault.set_current_timestamp(unpause);
    let result = vault.withdraw(cfg.cliff_amount - withdraw_amount);
    assert!(result.is_ok());
    assert_eq!(vault.available_amount(), 0);
}

#[test]
fn test_admin_release() {
    let cfg = VestingConfig::default();
    let mut vault = Vault::new(cfg.clone());
    let _ = vault.pause();
    vault.set_current_timestamp(cfg.admin_release_duration);
    let result = vault.admin_release().unwrap();
    assert_eq!(result, cfg.total_amount);
    assert_eq!(vault.available_amount(), 0);
}

#[test]
fn test_admin_release_not_paused_error() {
    let cfg = VestingConfig::default();
    let mut vault = Vault::new(cfg.clone());
    vault.set_current_timestamp(cfg.admin_release_duration);
    let result = vault.admin_release().unwrap_err();
    assert_eq!(result, VestingError::AdminReleaseErrorNotPaused);
}

#[test]
fn test_admin_release_not_enough_time_elapsed_error() {
    let cfg = VestingConfig::default();
    let mut vault = Vault::new(cfg.clone());
    vault.set_current_timestamp(1);
    let _ = vault.pause();
    vault.set_current_timestamp(cfg.admin_release_duration);
    let result = vault.admin_release().unwrap_err();
    assert_eq!(result, VestingError::AdminReleaseErrorNotEnoughTimeElapsed);
}

#[test]
fn test_admin_release_nothing_to_withdraw_error() {
    let cfg = VestingConfig::default();
    let mut vault = Vault::new(cfg.clone());
    let last_drip_period = (cfg.total_amount - cfg.cliff_amount) / cfg.drip_amount + 1;
    let after_last_drip_timestamp = cfg.cliff_timestamp + last_drip_period * cfg.drip_duration;
    vault.set_current_timestamp(after_last_drip_timestamp);
    let result = vault.withdraw(cfg.total_amount);
    assert!(result.is_ok());
    let _ = vault.pause();
    vault.set_current_timestamp(after_last_drip_timestamp + cfg.admin_release_duration);
    let result = vault.admin_release().unwrap_err();
    assert_eq!(result, VestingError::AdminReleaseErrorNothingToWithdraw);
}

#[test]
fn test_long_wait_attact_handled() {
    let cfg = VestingConfig::default();
    let mut vault = Vault::new(cfg.clone());
    // Wait until it's possible to withdraw all + one more drip_duration.
    let last_drip_period = (cfg.total_amount - cfg.cliff_amount) / cfg.drip_amount + 1;
    let time = cfg.cliff_timestamp + (last_drip_period + 1) * cfg.drip_duration;
    vault.set_current_timestamp(time);
    // Withdraw drip_amount.
    let result = vault.withdraw(cfg.drip_amount);
    assert!(result.is_ok());
    // Available amount should be correct.
    let expected = cfg.total_amount - cfg.drip_amount;
    assert_eq!(expected, vault.available_amount());
}
