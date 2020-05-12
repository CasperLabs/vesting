use std::{convert::TryFrom};
use casperlabs_contract::args_parser::ArgsParser;
use casperlabs_engine_test_support::{
    Code, Hash, SessionBuilder, TestContext, TestContextBuilder,
};
use casperlabs_engine_test_support::internal::{
    utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder as TestBuilder, DEFAULT_GENESIS_CONFIG,
};
use casperlabs_types::{account::PublicKey, bytesrepr::FromBytes, CLTyped, CLValue, Key, U512};

const VESTING_WASM: &str = "contract.wasm";
const VESTING_CONTRACT_NAME: &str = "vesting";
const VESTING_PROXY_CONTRACT_NAME: &str = "vesting_proxy";
pub const VESTING_INIT_BALANCE: u64 = 10000;

pub mod account {
    use super::PublicKey;
    pub const ADMIN: PublicKey = PublicKey::ed25519_from([1u8; 32]);
    pub const ALI: PublicKey = PublicKey::ed25519_from([2u8; 32]);
    pub const BOB: PublicKey = PublicKey::ed25519_from([3u8; 32]);
    pub const JOE: PublicKey = PublicKey::ed25519_from([4u8; 32]);
}

pub mod key {
	pub const VESTING: &str = "vesting";
	pub const VESTING_INDIRECT: &str = "vesting_indirect";
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
    pub const PURSE_NAME: &str = "vesting_main_purse";
}
pub struct VestingConfig {
    pub cliff_timestamp: U512,
    pub cliff_amount: U512,
    pub drip_duration: U512,
    pub drip_amount: U512,
    pub total_amount: U512,
    pub admin_release_duration: U512,
}

impl Default for VestingConfig {
    fn default() -> VestingConfig {
        VestingConfig {
            cliff_timestamp: 10.into(),
            cliff_amount: 2.into(),
            drip_duration: 3.into(),
            drip_amount: 5.into(),
            total_amount: 1000.into(),
            admin_release_duration: 123.into(),
        }
    }
}


mod method {
	pub const DEPLOY: &str = "deploy";
	pub const PAUSE: &str = "pause";
	pub const UNPAUSE: &str = "unpause";
	pub const RELEASE: &str = "release";
	pub const WITHDRAW_PROXY: &str = "withdraw_proxy";
    pub const ADMIN_RELEASE_PROXY: &str = "admin_release_proxy";
}

pub struct VestingContract {
	pub context: TestContext,
	pub contract_hash: Hash,
	pub indirect_hash: Hash,
    pub current_time: u64
}

pub struct Sender(pub PublicKey);

impl VestingContract {
	pub fn deployed() -> Self {
		let clx_init_balance = U512::from(10_000_000_000u64);
		let mut context = TestContextBuilder::new()
			.with_account(account::ADMIN,clx_init_balance)
			.with_account(account::ALI,clx_init_balance)
			.with_account(account::BOB,clx_init_balance)
			.build();
		let code = Code::from(VESTING_WASM);
		let config: VestingConfig = Default::default();
		let args = (method::DEPLOY,
				key::VESTING,
				account::ADMIN,
				account::ALI,
				config.cliff_timestamp,
				config.cliff_amount,
				config.drip_duration,
				config.drip_amount,
				config.total_amount,
				config.admin_release_duration);
		let session = SessionBuilder::new(code,args)
			.with_address(account::ADMIN)
			.with_authorization_keys(&[account::ADMIN])
            .with_block_time(0)
			.build();
		context.run(session);
		let contract_hash = Self::contract_hash(&context,VESTING_CONTRACT_NAME);
		let indirect_hash = Self::contract_hash(&context,VESTING_PROXY_CONTRACT_NAME);
		Self {
			context,
			contract_hash,
			indirect_hash,
            current_time: 0
		}
	}

    pub fn set_block_time(&mut self, block_time: u64) {
        self.current_time = block_time;
    }

	pub fn contract_hash(context: &TestContext, name: &str) -> Hash {
		let contract_ref: Key = context
			.query(account::ADMIN, &[name])
			.unwrap_or_else(|_| panic!("{} contract not found", name))
			.into_t()
			.unwrap_or_else(|_| panic!("{} is not a type Contract.", name));
		contract_ref
			.into_hash()
			.unwrap_or_else(|| panic!("{} is not a type Hash", name))
	}

    fn call_indirect(&mut self, sender: Sender, args: impl ArgsParser) {
        let Sender(address) = sender;
        let code = Code::Hash(self.indirect_hash);
        let session = SessionBuilder::new(code, args)
            .with_address(address)
            .with_authorization_keys(&[address])
            .with_block_time(self.current_time)
            .build();
        self.context.run(session);
    }


    pub fn query_contract<T: CLTyped + FromBytes>(&self, name: String) -> Option<T> {
        match self.context.query(account::ADMIN, &[key::VESTING, &name]) {
            Err(_) => None,
            Ok(maybe_value) => {
                let value = maybe_value
                    .into_t()
                    .unwrap_or_else(|_| panic!("{} is not expected type.", name));
                Some(value)
            }
        }
    }


    pub fn get_pause_status(&self) -> bool {
        let status: Option<bool> = self.query_contract(String::from(key::PAUSE_FLAG));
        status.unwrap()
    }



    pub fn get_total_amount(&self) -> u64 {
    	let balance: Option<U512> = self.query_contract(key::TOTAL_AMOUNT.to_string());
    	balance.unwrap().as_u64()
    }

    pub fn withdraw(
    	&mut self,
    	sender: Sender,
    	amount: u64
    ) {
    	self.call_indirect(
    		sender,
    		(
    			self.contract_hash,
    			method::WITHDRAW_PROXY,
    			U512::from(amount),
    		),
    	)
    }

    pub fn pause(
    	&mut self,
    	sender: Sender
    ){
    	self.call_indirect(
    		sender,
    		(
    			self.contract_hash,
    			method::PAUSE
    		),
    	)
    }


    pub fn unpause(
        &mut self,
        sender: Sender
    ){
        self.call_indirect(
            sender,
            (
                self.contract_hash,
                method::UNPAUSE
            ),
        )
    }


    pub fn admin_release(
        &mut self,
        sender: Sender
    ){
        self.call_indirect(
            sender,
            (
                self.contract_hash,
                method::ADMIN_RELEASE_PROXY
            ),
        )
    }
}