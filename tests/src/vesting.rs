use std::path::PathBuf;

use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::bytesrepr::FromBytes;
use casper_types::system::mint;
use casper_types::{account::AccountHash, runtime_args, PublicKey, RuntimeArgs, SecretKey, U512};
use casper_types::{CLTyped, ContractHash, Key};
use rand::Rng;

mod arg {
    pub const ADMIN: &str = "admin";
    pub const RECIPIENT: &str = "recipient";
    pub const CLIFF_TIME: &str = "cliff_timestamp";
    pub const CLIFF_AMOUNT: &str = "cliff_amount";
    pub const DRIP_DURATION: &str = "drip_duration";
    pub const DRIP_AMOUNT: &str = "drip_amount";
    pub const TOTAL_AMOUNT: &str = "total_amount";
    pub const PAUSE_FLAG: &str = "is_paused";
    pub const RELEASED_AMOUNT: &str = "released_amount";
    pub const ADMIN_RELEASE_DURATION: &str = "admin_release_duration";
    pub const DEPOSIT_CONTRACT_HASH: &str = "deposit_contract_hash";
    pub const AMOUNT: &str = "amount";
}

mod method {
    pub const WITHDRAW: &str = "withdraw";
    pub const PAUSE: &str = "pause";
    pub const UNPAUSE: &str = "unpause";
    pub const ADMIN_RELEASE: &str = "admin_release";
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

pub struct Vesting {
    pub builder: WasmTestBuilder<InMemoryGlobalState>,
    pub contract_hash: ContractHash,
    // pub package_hash: ContractPackageHash,
    pub admin_account: (PublicKey, AccountHash),
    pub ali_account: (PublicKey, AccountHash),
    pub bob_account: (PublicKey, AccountHash),
    pub current_time: u64,
}

impl Vesting {
    pub fn deploy() -> Self {
        let mut rng = rand::thread_rng();
        let admin_public_key: PublicKey =
            (&SecretKey::ed25519_from_bytes([1u8; 32]).unwrap()).into();
        let ali_public_key: PublicKey = (&SecretKey::ed25519_from_bytes([2u8; 32]).unwrap()).into();
        let bob_public_key: PublicKey = (&SecretKey::ed25519_from_bytes([3u8; 32]).unwrap()).into();
        let admin_account_addr = AccountHash::from(&admin_public_key);
        let ali_account_addr = AccountHash::from(&ali_public_key);
        let bob_account_addr = AccountHash::from(&bob_public_key);

        let mut code = PathBuf::from("contract.wasm");
        let config: VestingConfig = Default::default();
        let mut args = runtime_args! {
            arg::ADMIN => admin_account_addr,
            arg::RECIPIENT => ali_account_addr,
            arg::CLIFF_TIME => config.cliff_timestamp,
            arg::CLIFF_AMOUNT => config.cliff_amount,
            arg::DRIP_DURATION => config.drip_duration,
            arg::DRIP_AMOUNT => config.drip_amount,
            arg::TOTAL_AMOUNT => config.total_amount,
            arg::ADMIN_RELEASE_DURATION => config.admin_release_duration
        };

        let mut deploy = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_session_code(code, args)
            .with_address(admin_account_addr)
            .with_authorization_keys(&[admin_account_addr])
            .with_deploy_hash(rng.gen())
            .build();
        let mut execute_request = ExecuteRequestBuilder::from_deploy_item(deploy)
            .with_block_time(0)
            .build();
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let fund_my_account_request = {
            let deploy_item = DeployItemBuilder::new()
                .with_address(*DEFAULT_ACCOUNT_ADDR)
                .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
                .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_transfer_args(runtime_args! {
                    mint::ARG_AMOUNT => U512::from(50_000_000_000_000_u64),
                    mint::ARG_TARGET => admin_public_key.clone(),
                    mint::ARG_ID => <Option::<u64>>::None
                })
                .with_deploy_hash(rng.gen())
                .build();

            ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
        };
        builder
            .exec(fund_my_account_request)
            .commit()
            .expect_success();

        let fund_my_account_request = {
            let deploy_item = DeployItemBuilder::new()
                .with_address(*DEFAULT_ACCOUNT_ADDR)
                .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
                .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_transfer_args(runtime_args! {
                    mint::ARG_AMOUNT => U512::from(30_000_000_000_000u64),
                    mint::ARG_TARGET => ali_public_key.clone(),
                    mint::ARG_ID => <Option::<u64>>::None
                })
                .with_deploy_hash(rng.gen())
                .build();

            ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
        };
        builder
            .exec(fund_my_account_request)
            .commit()
            .expect_success();

        let fund_my_account_request = {
            let deploy_item = DeployItemBuilder::new()
                .with_address(*DEFAULT_ACCOUNT_ADDR)
                .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
                .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_transfer_args(runtime_args! {
                    mint::ARG_AMOUNT => U512::from(30_000_000_000_000u64),
                    mint::ARG_TARGET => bob_public_key.clone(),
                    mint::ARG_ID => <Option::<u64>>::None
                })
                .with_deploy_hash(rng.gen())
                .build();

            ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
        };
        builder
            .exec(fund_my_account_request)
            .expect_success()
            .commit();

        builder.exec(execute_request).expect_success().commit();

        let contract_hash = builder
            .query(
                None,
                Key::Account(admin_account_addr),
                &["vesting_contract_hash".to_string()],
            )
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t()
            .expect("should be string.");
        code = PathBuf::from("deposit.wasm");
        args = runtime_args! {
            arg::DEPOSIT_CONTRACT_HASH => contract_hash,
            arg::AMOUNT => config.total_amount,
        };
        deploy = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_session_code(code, args)
            .with_address(admin_account_addr)
            .with_authorization_keys(&[admin_account_addr])
            .with_deploy_hash(rng.gen())
            .build();
        execute_request = ExecuteRequestBuilder::from_deploy_item(deploy).build();
        builder.exec(execute_request).expect_success().commit();

        Self {
            builder,
            admin_account: (admin_public_key, admin_account_addr),
            contract_hash,
            ali_account: (ali_public_key, ali_account_addr),
            bob_account: (bob_public_key, bob_account_addr),
            current_time: 0,
        }
    }

    pub fn query_contract<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        match self.builder.query(
            None,
            Key::Account(self.admin_account.1),
            &["vesting_contract".to_string(), name.to_string()],
        ) {
            Err(_) => None,
            Ok(maybe_value) => {
                let value = maybe_value
                    .as_cl_value()
                    .expect("should be cl value.")
                    .clone()
                    .into_t()
                    .expect("should be string.");
                Some(value)
            }
        }
    }

    pub fn get_total_amount(&self) -> u64 {
        let balance: Option<U512> = self.query_contract(arg::TOTAL_AMOUNT);
        balance.unwrap().as_u64()
    }

    pub fn get_released_amount(&self) -> u64 {
        let amount: Option<U512> = self.query_contract(arg::RELEASED_AMOUNT);
        amount.unwrap().as_u64()
    }

    pub fn get_pause_status(&self) -> bool {
        let status: Option<bool> = self.query_contract(arg::PAUSE_FLAG);
        status.unwrap()
    }

    pub fn set_block_time(&mut self, block_time: u64) {
        self.current_time = block_time;
    }

    pub fn withdraw(&mut self, sender: AccountHash, amount: u64) {
        self.call_indirect(
            sender,
            method::WITHDRAW,
            runtime_args! {
                arg::AMOUNT => U512::from(amount)
            },
        );
    }

    pub fn pause(&mut self, sender: AccountHash) {
        self.call_indirect(sender, method::PAUSE, runtime_args! {});
    }

    pub fn unpause(&mut self, sender: AccountHash) {
        self.call_indirect(sender, method::UNPAUSE, runtime_args! {});
    }

    pub fn admin_release(&mut self, sender: AccountHash) {
        self.call_indirect(sender, method::ADMIN_RELEASE, runtime_args! {});
    }

    fn call_indirect(&mut self, sender: AccountHash, method: &str, args: RuntimeArgs) {
        let mut rng = rand::thread_rng();
        let deploy = DeployItemBuilder::new()
            .with_stored_session_hash(self.contract_hash, method, args)
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_address(sender)
            .with_authorization_keys(&[sender])
            .with_deploy_hash(rng.gen())
            .build();
        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy)
            .with_block_time(self.current_time)
            .build();
        self.builder.exec(execute_request).expect_success().commit();
    }
}
