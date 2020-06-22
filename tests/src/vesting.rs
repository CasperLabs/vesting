use casperlabs_engine_test_support::{Code, Hash, SessionBuilder, TestContext, TestContextBuilder};
use casperlabs_types::{
    account::PublicKey, bytesrepr::FromBytes, runtime_args, CLTyped, RuntimeArgs, U512,
};

const VESTING_WASM: &str = "contract.wasm";

pub mod account {
    use super::PublicKey;
    pub const ADMIN: PublicKey = PublicKey::ed25519_from([1u8; 32]);
    pub const ALI: PublicKey = PublicKey::ed25519_from([2u8; 32]);
    pub const BOB: PublicKey = PublicKey::ed25519_from([3u8; 32]);
}

mod key {
    pub const VESTING_CONTRACT_NAME: &str = "vesting_contract";
    pub const VESTING_CONTRACT_HASH: &str = "vesting_contract_hash";
    pub const TOTAL_AMOUNT: &str = "total_amount";
    pub const RELEASED_AMOUNT: &str = "released_amount";
    pub const PAUSE_FLAG: &str = "is_paused";
}

mod arg {
    pub const ADMIN: &str = "admin";
    pub const RECIPIENT: &str = "recipient";
    pub const CLIFF_TIME: &str = "cliff_timestamp";
    pub const CLIFF_AMOUNT: &str = "cliff_amount";
    pub const DRIP_DURATION: &str = "drip_duration";
    pub const DRIP_AMOUNT: &str = "drip_amount";
    pub const TOTAL_AMOUNT: &str = "total_amount";
    pub const ADMIN_RELEASE_DURATION: &str = "admin_release_duration";
    pub const AMOUNT: &str = "amount";
}

mod method {
    pub const PAUSE: &str = "pause";
    pub const UNPAUSE: &str = "unpause";
    pub const WITHDRAW: &str = "withdraw";
    pub const ADMIN_RELEASE: &str = "admin_release";
    pub const INIT: &str = "init";
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

pub struct VestingContract {
    pub context: TestContext,
    pub contract_hash: Hash,
    pub current_time: u64,
}

pub struct Sender(pub PublicKey);

impl VestingContract {
    pub fn deployed() -> Self {
        let clx_init_balance = U512::from(10_000_000_000u64);
        let mut context = TestContextBuilder::new()
            .with_account(account::ADMIN, clx_init_balance)
            .with_account(account::ALI, clx_init_balance)
            .with_account(account::BOB, clx_init_balance)
            .build();
        let code = Code::from(VESTING_WASM);
        let config: VestingConfig = Default::default();
        let args = runtime_args! {
            arg::ADMIN => account::ADMIN,
            arg::RECIPIENT => account::ALI,
            arg::CLIFF_TIME => config.cliff_timestamp,
            arg::CLIFF_AMOUNT => config.cliff_amount,
            arg::DRIP_DURATION => config.drip_duration,
            arg::DRIP_AMOUNT => config.drip_amount,
            arg::TOTAL_AMOUNT => config.total_amount,
            arg::ADMIN_RELEASE_DURATION => config.admin_release_duration
        };

        let session = SessionBuilder::new(code, args)
            .with_address(account::ADMIN)
            .with_authorization_keys(&[account::ADMIN])
            .with_block_time(0)
            .build();
        context.run(session);
        let contract_hash = Self::contract_hash(&context, key::VESTING_CONTRACT_HASH);
        Self {
            context,
            contract_hash,
            current_time: 0,
        }
    }

    pub fn set_block_time(&mut self, block_time: u64) {
        self.current_time = block_time;
    }

    pub fn contract_hash(context: &TestContext, name: &str) -> Hash {
        context
            .query(account::ADMIN, &[name])
            .unwrap_or_else(|_| panic!("{} contract not found", name))
            .into_t()
            .unwrap_or_else(|_| panic!("{} has wrong type", name))
    }

    fn call_indirect(&mut self, sender: Sender, method: &str, args: RuntimeArgs) {
        let Sender(address) = sender;
        let code = Code::Hash(self.contract_hash, method.to_string());
        let session = SessionBuilder::new(code, args)
            .with_address(address)
            .with_authorization_keys(&[address])
            .with_block_time(self.current_time)
            .build();
        self.context.run(session);
    }

    pub fn query_contract<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        match self.context.query(
            account::ADMIN,
            &[key::VESTING_CONTRACT_NAME, &name.to_string()],
        ) {
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
        let status: Option<bool> = self.query_contract(key::PAUSE_FLAG);
        status.unwrap()
    }

    pub fn get_released_amount(&self) -> u64 {
        let amount: Option<U512> = self.query_contract(key::RELEASED_AMOUNT);
        amount.unwrap().as_u64()
    }

    pub fn get_total_amount(&self) -> u64 {
        let balance: Option<U512> = self.query_contract(key::TOTAL_AMOUNT);
        balance.unwrap().as_u64()
    }

    pub fn withdraw(&mut self, sender: Sender, amount: u64) {
        self.call_indirect(
            sender,
            method::WITHDRAW,
            runtime_args! {
                arg::AMOUNT => U512::from(amount)
            },
        );
    }

    pub fn pause(&mut self, sender: Sender) {
        self.call_indirect(sender, method::PAUSE, runtime_args! {});
    }

    pub fn unpause(&mut self, sender: Sender) {
        self.call_indirect(sender, method::UNPAUSE, runtime_args! {});
    }

    pub fn admin_release(&mut self, sender: Sender) {
        self.call_indirect(sender, method::ADMIN_RELEASE, runtime_args! {});
    }

    pub fn init(
        &mut self,
        sender: Sender,
        admin: PublicKey,
        recipient: PublicKey,
        config: VestingConfig,
    ) {
        self.call_indirect(
            sender,
            method::INIT,
            runtime_args! {
                arg::ADMIN => admin,
                arg::RECIPIENT => recipient,
                arg::CLIFF_TIME => config.cliff_timestamp,
                arg::CLIFF_AMOUNT => config.cliff_amount,
                arg::DRIP_DURATION => config.drip_duration,
                arg::DRIP_AMOUNT => config.drip_amount,
                arg::TOTAL_AMOUNT => config.total_amount,
                arg::ADMIN_RELEASE_DURATION => config.admin_release_duration
            },
        );
    }
}
