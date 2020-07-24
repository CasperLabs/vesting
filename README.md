# Vesting Smart Contract

This is an implementation of a vesting smart contract for Casperlabs including a vesting smart contract separated into the smart contract and logic layers.

## Available methods

* Deploy
* Pause
* Unpause
* Admin Release
* Withdraw

After deployment, an additional smart contract is saved under the `vesting_proxy` key. Pause, Unpause, Admin Release and Withdraw methods have to call this contract and specify the vesting contract hash as a first argument.

## Deploy

When deploying, the following parameters have to be specified in the given order.

#### Parameters

| Name | Type | Description |
| ---  | --- | --- |
| method | string | Has to be `deploy`. |
| contract_name | string | Name of the key the contract is save under. |
| admin | bytes | Account base16 address, that will act as an administrator. Admin can pause, unpause the contract and release remaining amount when conditions are met. |
| recipient | bytes | Account base16 address, that is eligible to withdraw the funds from the contract when conditions are met. |
| cliff_timestamp | U512 | At this time, recipient is eligible to withdraw the `cliff_amount` from the contract. |
| cliff_amount | U512 | The amount the recipient can withdraw after at `cliff_timestamp` time. |
| drip_duration | U512 | After the `cliff_timestamp` is reached, evertime this period of time elapses, recipient is eligible to withdraw addtional `drip_amount` from the contract. |
| drip_amount | U512 | The additiona amount the recipient can withdraw after every `drip_duration` period. |
| total_amount | U512 | Total amount of CLX tokens locked in the smart contract at the deploy. |
| admin_release_duration | U512 | The time on pause, that has to elapse for the Admin to be able to withdraw all of the remainig balance from the contract. |

## Pause

Pausing stops the internal contract's clock. Only the Admin can pause the contract.

#### Parameters

| Name | Type | Description |
| ---  | --- | --- |
| vesting_contract_address | bytes | Address of the vesting contract. |
| method | string | Has to be 'pause'. |

## Unpause

Unpausing resumes the internal contract's clock. Only the Admin can unpause the contract.

#### Parameters

| Name | Type | Description |
| ---  | --- | --- |
| vesting_contract_address | bytes | Address of the vesting contract. |
| method | string | Has to be 'unpause'. |

## Admin Release

#### Parameters

If the contract was paused without unpausing an `admin_release_duration` ago, then the Admin can call this method to withdraw all CLX tokens from the account to the Admin's main purse.

| Name | Type | Description |
| ---  | --- | --- |
| vesting_contract_address | bytes | Address of the vesting contract. |
| method | string | Has to be 'admin_release_proxy'. |

## Withdraw

#### Parameters

The Recipient account can withdraw the available CLX amount from the smart contract. This is transferred to the main purse of the recipient.

| Name | Type | Description |
| ---  | --- | --- |
| vesting_contract_address | bytes | Address of the vesting contract. |
| method | string | Has to be 'withdraw_release'. |
| amount | U512 | The amount to withdraw from the contract.  |


## Error codes

Code | Message |
--- | --- |
65537 | UnknownApiCommand            | 
65538 | UnknownDeployCommand         | 
65539 | UnknownProxyCommand          | 
65540 | UnknownConstructorCommand    | 
65541 | UnknownVestingCallCommand    | 
65542 | AlreadyPaused                | 
65543 | AlreadyUnpaused              | 
65544 | NotTheAdminAccount           | 
65545 | NotTheRecipientAccount       | 
65546 | UnexpectedVestingError       | 
65547 | NotEnoughBalance             | 
65548 | PurseTransferError           | 
65549 | PurseBalanceCheckError       | 
65550 | NotPaused                    | 
65551 | NothingToWithdraw            | 
65552 | NotEnoughTimeElapsed         | 
65553 | LocalPurseKeyMissing         | 
65554 | UnexpectedType               | 
65555 | MissingKey                   | 
65556 | MissingArgument0             | 
65557 | MissingArgument1             | 
65558 | MissingArgument2             | 
65559 | MissingArgument3             | 
65560 | MissingArgument4             | 
65561 | MissingArgument5             | 
65562 | MissingArgument6             | 
65563 | MissingArgument7             | 
65564 | MissingArgument8             | 
65565 | MissingArgument9             | 
65566 | InvalidArgument0             | 
65567 | InvalidArgument1             | 
65568 | InvalidArgument2             | 
65569 | InvalidArgument3             | 
65570 | InvalidArgument4             | 
65571 | InvalidArgument5             | 
65572 | InvalidArgument6             | 
65573 | InvalidArgument7             | 
65574 | InvalidArgument8             | 
65575 | InvalidArgument9             | 
65576 | UnsupportedNumberOfArguments |

## Example of deploy
```bash
CLIFF_TIMESTAMP=1583024523 # 2020/03/01 @ 1:02am (UTC)
CLIFF_AMOUNT=100
DRIP_DURATION=86400 # 1 day
DRIP_AMOUNT=20
TOTAL_AMOUNT=500
ADMIN_RELEASE_DURATION=432000 # 5 days

ARGS='[
    {"name": "method", "value": {"string_value": "deploy"}}, 
    {"name": "contract_name", "value": {"string_value": "vault_01"}}, 
    {"name": "admin", "value": {"bytes_value": "'$ADMIN_PUB'"}}, 
    {"name": "recipient", "value": {"bytes_value": "'$RECIPIENT_PUB'"}}, 
    {"name": "cliff_timestamp", "value": {"big_int": {"value": "'$CLIFF_TIMESTAMP'", "bit_width": 512}}}, 
    {"name": "cliff_amount", "value": {"big_int": {"value": "'$CLIFF_AMOUNT'", "bit_width": 512}}}, 
    {"name": "drip_timestamp", "value": {"big_int": {"value": "'$DRIP_DURATION'", "bit_width": 512}}}, 
    {"name": "drip_duration", "value": {"big_int": {"value": "'$DRIP_AMOUNT'", "bit_width": 512}}}, 
    {"name": "total_amount", "value": {"big_int": {"value": "'$TOTAL_AMOUNT'", "bit_width": 512}}}, 
    {"name": "admin_relase_duration", "value": {"big_int": {"value": "'$ADMIN_RELEASE_DURATION'", "bit_width": 512}}} 
]'

casperlabs_client --host deploy.casperlabs.io deploy \
    --private-key $SENDER_PRIVATE_KEY \
    --payment-amount 10000000 \
    --session $VESTING_WASM \
    --session-args "$ARGS"
```
## Testing in an Online IDE
[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#snapshot/dabd213a-67b7-42d0-b6de-8b8f6fd9cfd5)

You can follow the above link to open this project in an online IDE, to build/test the contract. 


