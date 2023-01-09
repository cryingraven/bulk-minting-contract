use near_sdk::{AccountId};

pub fn account_from(s: &str) -> AccountId {
    if s.len()==2{
        AccountId::new_unchecked(s.repeat(32).to_string())
    }else {
        AccountId::new_unchecked(s.repeat(64).to_string())
    }
}
