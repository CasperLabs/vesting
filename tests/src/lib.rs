#[cfg(test)]
mod vesting;

#[cfg(test)]
mod tests {
	use super::vesting;
	use vesting::{
		account::{ADMIN,ALI,BOB},
		VestingContract, VestingConfig ,Sender, VESTING_INIT_BALANCE,
	};

	#[test]
	fn test_vesting_deploy() {
		let vesting = VestingContract::deployed();
		let config: VestingConfig = Default::default();
		let amount = vesting.get_total_amount();
		assert_eq!(amount, config.total_amount.as_u64());
	}

	#[test]
	#[ignore]
	fn test_withdraw(){
		let config: VestingConfig = Default::default();
		let mut vesting = VestingContract::deployed();
		vesting.withdraw(Sender(ALI),1);
		assert_eq!(1,1);
	}

	#[test]
	fn test_pause_by_admin(){
		let mut vesting = VestingContract::deployed();
		vesting.pause(Sender(ADMIN));
		let status = vesting.get_pause_status();
		assert!(status,"The contract is not paused");
	}

	#[test]
	fn test_unpause_by_admin() {
		let mut vesting = VestingContract::deployed();
		vesting.pause(Sender(ADMIN));
		vesting.unpause(Sender(ADMIN));
		let status = vesting.get_pause_status();
		assert!(!status,"The contract is still paused");
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
	#[ignore]
	fn test_admin_release() {
		let mut vesting = VestingContract::deployed();
		vesting.pause(Sender(ADMIN));
		vesting.admin_release(Sender(ADMIN));
		let amount = vesting.get_total_amount();
		assert_eq!(amount, 0);
	}


}