use casper_types::{account::AccountHash, runtime_args, RuntimeArgs};

use test_env::{TestContract, TestEnv};
pub struct Vesting(TestContract);

impl Vesting {
    pub fn new(env: &TestEnv, contract_name: &str, sender: AccountHash) -> KVStorage {
        Vesting(TestContract::new(
            env,
            "contract.wasm",
            contract_name,
            sender,
            runtime_args! {},
        ))
    }

    // pub fn create(&self, sender: AccountHash, uref_name: &str, dict_key: &str, dict_value: String) {
    //     self.0.call_contract(
    //         sender,
    //         "create",
    //         runtime_args! {
    //             "uref" => uref_name,
    //             "dict_key" => dict_key,
    //             "dict_value" => dict_value
    //         },
    //     )
    // }

    // pub fn update(&self, sender: AccountHash, uref_name: &str, dict_key: &str, dict_value: String) {
    //     self.0.call_contract(
    //         sender,
    //         "update",
    //         runtime_args! {
    //             "uref" => uref_name,
    //             "dict_key" => dict_key,
    //             "dict_value" => dict_value
    //         },
    //     )
    // }

    // pub fn delete(&self, sender: AccountHash, uref_name: &str, dict_key: &str) {
    //     self.0.call_contract(
    //         sender,
    //         "delete",
    //         runtime_args! {
    //             "uref" => uref_name,
    //             "dict_key" => dict_key,
    //         },
    //     )
    // }

    // pub fn read(&self, uref_name: &str, dict_key: &str) -> String {
    //     self.0
    //         .query_dictionary(uref_name, dict_key.to_string())
    //         .unwrap()
    // }
}
