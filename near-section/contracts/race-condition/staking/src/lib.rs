use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    collections::{LookupSet, UnorderedMap},
    env,
    json_types::U128,
    log, near_bindgen, require, AccountId, PanicOnDefault, Promise,
};

pub const TGAS: u64 = 1_000_000_000_000;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Staking {
    stake_map: UnorderedMap<(AccountId, AccountId), U128>,
    allowlist: LookupSet<AccountId>,
}

#[near_bindgen]
impl Staking {
    #[init]
    pub fn new(account: AccountId) -> Self {
        let stake_map = UnorderedMap::new(b"u");
        let mut allowlist = LookupSet::new(b"a");

        allowlist.insert(&account);
        Self {
            stake_map,
            allowlist,
        }
    }

    pub fn stake(&mut self, beneficiary: AccountId, validator: AccountId, amount: U128) {
        let caller = env::predecessor_account_id();

        assert!(self.allowlist.contains(&caller), "ACCESS DENIED");

        if let Some(stake) = self
            .stake_map
            .get(&(beneficiary.clone(), validator.clone()))
        {
            let new_stake = U128::from(stake.0 + amount.0);

            log!(format!(
                "Staked by {:?}, For {:?}, Amount {:?}",
                beneficiary, validator, amount
            ));

            self.stake_map.insert(&(beneficiary, validator), &new_stake);
        } else {
            log!(format!(
                "Staked by {:?}, For {:?}, Amount {:?}",
                beneficiary, validator, amount
            ));

            self.stake_map.insert(&(beneficiary, validator), &amount);
        }
    }

    pub fn withdraw_stake(&mut self, amount: U128, validator: AccountId) {
        let caller = env::predecessor_account_id();
        let mut beneficiary_stake = self.view_stake(caller.clone(), validator.clone());

        require!(amount.0 != 0, "Amount should not be 0");
        require!(beneficiary_stake.0 != 0, "Nothing to withdraw");
        require!(
            beneficiary_stake >= amount.clone(),
            "Not enough funds to withdraw"
        );

        beneficiary_stake = U128(beneficiary_stake.0 - amount.0);

        self.stake_map
            .insert(&(caller.clone(), validator), &beneficiary_stake);

        log!(format!(
            "Transferred: {:?}. Current Stake: {:?}",
            amount, beneficiary_stake
        ));

        Promise::new(caller).transfer(amount.0);
    }

    pub fn view_stake(&mut self, account: AccountId, validator: AccountId) -> U128 {
        self.stake_map
            .get(&(account, validator))
            .unwrap_or_else(|| env::panic_str("No Stake"))
    }
}
