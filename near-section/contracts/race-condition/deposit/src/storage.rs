use near_sdk::{
    borsh::{self, BorshSerialize},
    AccountId,
};
/// Defines a set of [`StorageKey`]s for [`UnorderedSet`]'s and [`UnorderedMap`]'s prefixes.
/// It is used to namespace the collections in the NEAR VM and prevent collisions in this contract.
#[derive(Debug, Clone, BorshSerialize, near_sdk::BorshStorageKey)]
pub(crate) enum TokenStorageKey {
    Accounts,
    // StorageKey for a temporary UnorderedMap to map a spender_id to its allowed spending amount.
    // This is the nested UnorderedMap value inside Allowed, mapping to the key: the holder_id.
    Allowance { account_id: AccountId },
}
