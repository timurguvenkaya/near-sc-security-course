use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    collections::UnorderedMap, env, json_types::U128, log, near_bindgen, require, AccountId,
    Promise, ONE_NEAR,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    user_near: UnorderedMap<AccountId, U128>,
}

//Default constructor
impl Default for FungibleToken {
    fn default() -> Self {
        Self {
            user_near: UnorderedMap::new(b"u"),
        }
    }
}

#[near_bindgen]
impl FungibleToken {
    // Deposit NEAR
    #[payable]
    pub fn deposit_near(&mut self) {
        let deposit = env::attached_deposit();

        require!(deposit >= ONE_NEAR, "Not enough deposit");

        let caller = env::predecessor_account_id();

        if self.user_near.get(&caller).is_none() {
            self.user_near.insert(&caller, &deposit.into());

            log!(format!("Added {} for {}", deposit, caller));
        } else {
            let near_deposit = self.user_near.get(&caller).unwrap();
            let new_near_deposit = U128::from(near_deposit.0 + deposit);

            self.user_near.insert(&caller, &new_near_deposit);

            log!(format!("Added {} for {}", new_near_deposit.0, caller))
        }
    }

    pub fn transfer_near(&mut self, receiver: AccountId, amount: U128) {
        let sender = env::predecessor_account_id();

        require!(sender != receiver, "Cannot transfer to yourself");

        self.internal_transfer_near(sender, receiver, amount)
    }

    pub fn withdraw_near(&mut self, amount: U128) {
        let caller = env::predecessor_account_id();
        let near_deposit = self
            .user_near
            .get(&caller)
            .unwrap_or_else(|| env::panic_str("User does not exist"));

        require!(amount <= near_deposit, "Not enough money");

        log!(format!(
            "Transferred: {:?}. Current Deposit: {:?}",
            amount, near_deposit
        ));

        Promise::new(caller.clone()).transfer(amount.0);
        self.decrease_balance(caller, amount)
    }

    pub fn view_near_deposit(&self, acc: AccountId) -> U128 {
        let near_deposit = self
            .user_near
            .get(&acc)
            .unwrap_or_else(|| env::panic_str("User does not exist"));

        near_deposit
    }

    fn decrease_balance(&mut self, account: AccountId, amount: U128) {
        let near_deposit = self
            .user_near
            .get(&account)
            .unwrap_or_else(|| env::panic_str("User does not exist"));

        let new_deposit = U128::from(
            near_deposit
                .0
                .checked_sub(amount.0)
                .unwrap_or_else(|| env::panic_str("Subtract with underflow")),
        );

        self.user_near.insert(&account, &new_deposit);

        log!(format!("Decreased {} of {}", new_deposit.0, account))
    }

    fn internal_transfer_near(&mut self, sender: AccountId, receiver: AccountId, amount: U128) {
        let sender_deposit = self
            .user_near
            .get(&sender)
            .unwrap_or_else(|| env::panic_str("User does not exist"));

        require!(amount <= sender_deposit, "Not enough money");

        let receiver_deposit = self
            .user_near
            .get(&receiver)
            .unwrap_or_else(|| env::panic_str("User does not exist"));

        let new_sender_deposit = U128::from(
            sender_deposit
                .0
                .checked_sub(amount.0)
                .unwrap_or_else(|| env::panic_str("Subtraction with underflow")),
        ); // 100 - 20 = 80
        let new_receiver_deposit = U128::from(
            receiver_deposit
                .0
                .checked_add(amount.0)
                .unwrap_or_else(|| env::panic_str("Addition with overflow")),
        ); // 100 + 20 = 120

        self.user_near.insert(&sender, &new_sender_deposit); // Attacker has 80
        self.user_near.insert(&receiver, &new_receiver_deposit); // Attacked has 120

        log!(format!(
            "Transferred {} from {} to {}",
            amount.0, sender, receiver
        ));
    }
}
