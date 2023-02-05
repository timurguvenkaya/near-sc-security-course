use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, require, AccountId, PanicOnDefault};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct StatusMessage {
    data: String,
    pause_status: bool,
    owner: AccountId,
}

#[near_bindgen]
impl StatusMessage {
    #[init]
    pub fn init(data: String, owner: AccountId) -> Self {
        Self {
            owner,
            data: data,
            pause_status: false,
        }
    }

    pub fn get_pause_status(&self) -> bool {
        self.pause_status
    }

    pub fn get_data(&self) -> String {
        self.when_not_paused();
        self.data.clone()
    }

    pub fn pub_toggle_pause(&mut self) {
        require!(
            env::predecessor_account_id() == self.owner,
            "Only owner can call this function"
        );
        self.toggle_pause()
    }

    pub fn set_owner(&mut self, new_owner: AccountId) {
        require!(
            env::signer_account_id() == self.owner,
            "Only owner can call this function"
        );
        self.owner = new_owner;
    }
}

pub trait Pausable {
    fn toggle_pause(&mut self);
    fn pause(&mut self);
    fn unpause(&mut self);
    fn when_not_paused(&self);
}

impl Pausable for StatusMessage {
    fn toggle_pause(&mut self) {
        if !self.pause_status {
            self.pause()
        } else {
            self.unpause()
        }
    }

    fn pause(&mut self) {
        self.pause_status = true;
        env::log_str("The system is paused")
    }

    fn unpause(&mut self) {
        self.pause_status = false;
        env::log_str("The system is unpaused")
    }

    fn when_not_paused(&self) {
        if self.pause_status {
            env::panic_str("Function is paused")
        }
    }
}
