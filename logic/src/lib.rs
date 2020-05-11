#![no_std]

use core::{
    cmp::{self, Ord},
    ops::{Add, Div, Mul, Sub},
};
use num_traits::{One, Zero};

#[cfg(test)]
mod tests;

#[derive(PartialEq, Debug)]
pub enum VestingError {
    NotEnoughBalance,
    AdminReleaseErrorNotPaused,
    AdminReleaseErrorNothingToWithdraw,
    AdminReleaseErrorNotEnoughTimeElapsed,
    AlreadyPaused,
    AlreadyUnpaused,
}

pub mod key {
    pub const CLIFF_TIMESTAMP: &str = "cliff_timestamp";
    pub const CLIFF_AMOUNT: &str = "cliff_amount";
    pub const DRIP_DURATION: &str = "drip_duration";
    pub const DRIP_AMOUNT: &str = "drip_amount";
    pub const TOTAL_AMOUNT: &str = "total_amount";
    pub const RELEASED_AMOUNT: &str = "released_amount";
    pub const ADMIN_RELEASE_DURATION: &str = "admin_release_duration";
    pub const PAUSE_FLAG: &str = "is_paused";
    pub const ON_PAUSE_DURATION: &str = "on_pause_duration";
    pub const LAST_PAUSE_TIMESTAMP: &str = "last_pause_timestamp";
}

pub trait VestingTrait<
    Amount: Copy + Zero + One + Add<Output = Amount> + Sub<Output = Amount> + Mul<Output = Amount> + Ord,
    Time: Copy + Zero + PartialOrd + Sub<Output = Time> + Div<Output = Amount>,
>
{
    fn init(
        &mut self,
        cliff_timestamp: Time,
        cliff_amount: Amount,
        drip_duration: Time,
        drip_amount: Amount,
        total_amount: Amount,
        admin_release_duration: Time,
    ) {
        self.set_cliff_timestamp(cliff_timestamp);
        self.set_cliff_amount(cliff_amount);
        self.set_drip_duration(drip_duration);
        self.set_drip_amount(drip_amount);
        self.set_total_amount(total_amount);
        self.set_admin_release_duration(admin_release_duration);
        self.set_released_amount(Amount::zero());
        self.set_paused_flag(false);
        self.set_on_pause_duration(Time::zero());
        self.set_last_pause_timestamp(Time::zero());
    }

    fn available_amount(&self) -> Amount {
        let current_timestamp = self.current_timestamp();
        let cliff_timestamp = self.cliff_timestamp();
        let total_paused_duration = self.total_paused_duration();
        let cliff_timestamp_adjusted = cliff_timestamp + total_paused_duration;
        if current_timestamp < cliff_timestamp_adjusted {
            Amount::zero()
        } else {
            let drip_duration = self.drip_duration();
            let time_diff: Time = current_timestamp - cliff_timestamp_adjusted;
            let available_drips = if drip_duration == Time::zero() {
                Amount::zero()
            } else {
                time_diff / drip_duration
            };
            let total_amount = self.total_amount();
            let drip_amount = self.drip_amount();
            let released_amount = self.released_amount();
            let mut counter = self.cliff_amount();
            counter = counter + drip_amount * available_drips;
            counter = cmp::min(counter, total_amount);
            counter - released_amount
        }
    }

    fn withdraw(&mut self, amount: Amount) -> Result<(), VestingError> {
        let available_amount = self.available_amount();
        if available_amount < amount {
            Err(VestingError::NotEnoughBalance)
        } else {
            let released_amount = self.released_amount();
            self.set_released_amount(released_amount + amount);
            Ok(())
        }
    }

    fn pause(&mut self) -> Result<(), VestingError> {
        if !self.is_paused() {
            self.set_last_pause_timestamp(self.current_timestamp());
            self.set_paused_flag(true);
            Ok(())
        } else {
            Err(VestingError::AlreadyPaused)
        }
    }

    fn unpause(&mut self) -> Result<(), VestingError> {
        if self.is_paused() {
            let on_pause_duration = self.on_pause_duration();
            let last_pause_timestamp = self.last_pause_timestamp();
            let elapsed_timestamp = self.current_timestamp() - last_pause_timestamp;
            self.set_on_pause_duration(on_pause_duration + elapsed_timestamp);
            self.set_paused_flag(false);
            Ok(())
        } else {
            Err(VestingError::AlreadyUnpaused)
        }
    }

    fn total_paused_duration(&self) -> Time {
        self.on_pause_duration()
            + if self.is_paused() {
                self.current_timestamp() - self.last_pause_timestamp()
            } else {
                Time::zero()
            }
    }

    fn is_paused(&self) -> bool {
        self.paused_flag()
    }

    fn admin_release(&mut self) -> Result<Amount, VestingError> {
        if !self.is_paused() {
            return Err(VestingError::AdminReleaseErrorNotPaused);
        }
        let last_pause_timestamp = self.last_pause_timestamp();
        let since_last_pause = self.current_timestamp() - last_pause_timestamp;
        let required_wait_duration = self.admin_release_duration();
        if since_last_pause < required_wait_duration {
            return Err(VestingError::AdminReleaseErrorNotEnoughTimeElapsed);
        }
        let total_amount = self.total_amount();
        let released_amount = self.released_amount();
        if total_amount == released_amount {
            return Err(VestingError::AdminReleaseErrorNothingToWithdraw);
        }
        let amount_to_withdraw = total_amount - released_amount;
        self.set_released_amount(total_amount);
        Ok(amount_to_withdraw)
    }

    fn set_cliff_timestamp(&mut self, cliff_timestamp: Time) {
        self.set_time(key::CLIFF_TIMESTAMP, cliff_timestamp);
    }

    fn cliff_timestamp(&self) -> Time {
        self.time(key::CLIFF_TIMESTAMP)
    }

    fn set_cliff_amount(&mut self, cliff_amount: Amount) {
        self.set_amount(key::CLIFF_AMOUNT, cliff_amount);
    }

    fn cliff_amount(&self) -> Amount {
        self.amount(key::CLIFF_AMOUNT)
    }

    fn set_drip_duration(&mut self, drip_duration: Time) {
        self.set_time(key::DRIP_DURATION, drip_duration);
    }

    fn drip_duration(&self) -> Time {
        self.time(key::DRIP_DURATION)
    }

    fn set_drip_amount(&mut self, drip_amount: Amount) {
        self.set_amount(key::DRIP_AMOUNT, drip_amount);
    }

    fn drip_amount(&self) -> Amount {
        self.amount(key::DRIP_AMOUNT)
    }

    fn set_total_amount(&mut self, total_amount: Amount) {
        self.set_amount(key::TOTAL_AMOUNT, total_amount);
    }

    fn total_amount(&self) -> Amount {
        self.amount(key::TOTAL_AMOUNT)
    }

    fn set_released_amount(&mut self, released_amount: Amount) {
        self.set_amount(key::RELEASED_AMOUNT, released_amount);
    }

    fn released_amount(&self) -> Amount {
        self.amount(key::RELEASED_AMOUNT)
    }

    fn set_admin_release_duration(&mut self, admin_release_duration: Time) {
        self.set_time(key::ADMIN_RELEASE_DURATION, admin_release_duration);
    }

    fn admin_release_duration(&self) -> Time {
        self.time(key::ADMIN_RELEASE_DURATION)
    }

    fn set_on_pause_duration(&mut self, on_pause_duration: Time) {
        self.set_time(key::ON_PAUSE_DURATION, on_pause_duration);
    }

    fn on_pause_duration(&self) -> Time {
        self.time(key::ON_PAUSE_DURATION)
    }

    fn set_last_pause_timestamp(&mut self, last_pause_timestamp: Time) {
        self.set_time(key::LAST_PAUSE_TIMESTAMP, last_pause_timestamp);
    }

    fn last_pause_timestamp(&self) -> Time {
        self.time(key::LAST_PAUSE_TIMESTAMP)
    }

    fn set_paused_flag(&mut self, is_paused: bool) {
        self.set_boolean(key::PAUSE_FLAG, is_paused);
    }

    fn paused_flag(&self) -> bool {
        self.boolean(key::PAUSE_FLAG)
    }

    fn current_timestamp(&self) -> Time;
    fn set_amount(&mut self, name: &str, value: Amount);
    fn amount(&self, name: &str) -> Amount;
    fn set_time(&mut self, name: &str, value: Time);
    fn time(&self, name: &str) -> Time;
    fn set_boolean(&mut self, name: &str, value: bool);
    fn boolean(&self, name: &str) -> bool;
}
