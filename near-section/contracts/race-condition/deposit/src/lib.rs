use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    collections::UnorderedMap, env, ext_contract, json_types::U128, log, near_bindgen, require,
    AccountId, Balance, Gas, PanicOnDefault, Promise, PromiseError, ONE_NEAR,
};

pub const TGAS: u64 = 1_000_000_000_000;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct ReentrancyCheck {
    user_near: UnorderedMap<AccountId, U128>,
    staking_contract: AccountId,
}
#[ext_contract(staking)]
trait Staking {
    fn stake(&self, beneficiary: AccountId, validator: AccountId, amount: U128);
}

#[near_bindgen]
impl ReentrancyCheck {
    #[init]
    pub fn new(staking_contract: AccountId) -> Self {
        let user_near = UnorderedMap::new(b"u");

        Self {
            user_near,
            staking_contract,
        }
    }
    // Deposit some cash

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

    pub fn stake(&self, validator: AccountId, amount: U128) {
        let beneficiary = env::predecessor_account_id();

        let near_deposit = self
            .user_near
            .get(&beneficiary)
            .unwrap_or_else(|| env::panic_str("User does not exist"));

        require!(amount <= near_deposit, "Not enough money");

        log!(format!(
            "Inside deposit contract: Staked by {:?}, For {:?}, Amount {:?}",
            beneficiary, validator, amount
        ));

        staking::ext(self.staking_contract.clone())
            .with_static_gas(Gas(3 * TGAS))
            .stake(beneficiary.clone(), validator.clone(), amount)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(3 * TGAS))
                    .resolve_staking(amount, beneficiary.clone()),
            );
    }

    pub fn get_current_balance(&self) -> Balance {
        env::account_balance()
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
    #[private]
    pub fn resolve_staking(
        &mut self,
        #[callback_result] call_result: Result<(), PromiseError>,
        amount: U128,
        caller: AccountId,
    ) {
        if call_result.is_err() {
            env::panic_str(format!("ERROR STAKING: {:?}", call_result.err().unwrap()).as_str());
        } else {
            log!("ALL GOOD");
            self.decrease_balance(caller, amount)
        }
    }

    pub fn view_near_deposit(&self, acc: AccountId) -> U128 {
        let near_deposit = self
            .user_near
            .get(&acc)
            .unwrap_or_else(|| env::panic_str("User does not exist"));

        near_deposit
    }

    #[private]
    pub fn decrease_balance(&mut self, account: AccountId, amount: U128) {
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
}
