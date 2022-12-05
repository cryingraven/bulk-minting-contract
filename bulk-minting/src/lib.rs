use std::collections::HashMap;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise, ext_contract, Gas, StorageUsage, PanicOnDefault, is_promise_success};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata,
};
use near_sdk::json_types::U128;
use near_sdk::serde_json;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupSet;

const CONTRACT_BALANCE: Balance = 8_000_000_000_000_000_000_000_000; // 8 N
const DEPLOY_BALANCE: Balance = 10_000_000_000_000_000_000_000_000; // 10 N
const CODE: &[u8] = include_bytes!("nft.wasm");
const CREATE_CONTRACT: Gas = Gas(70 * 10u64.pow(12));
const CREATE_CALLBACK: Gas = Gas(10 * 10u64.pow(12));
const NFT_CONTRACT: StorageUsage = 550_000;
const TOKEN: StorageUsage = 360;
type BasisPoint = u16;

#[ext_contract(bulk_self)]
pub trait OnCreateCallback {
    fn on_create(
        &mut self,
        creator_id: AccountId,
        metadata: NFTContractMetadata,
        nft_account_id: AccountId,
        attached_deposit: U128,
    );
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Royalties {
    pub accounts: HashMap<AccountId, BasisPoint>,
    pub percent: BasisPoint,
}

#[derive(Deserialize, Serialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    pub royalties: Option<Royalties>,
    pub price: U128,
}

#[derive(Deserialize, Serialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTArgs {
    pub metadata: NFTContractMetadata,
    pub owner_id: AccountId,
    pub size: u32,
    pub sale: Sale
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ParasFactory {
    pub contracts: LookupSet<String>,
    pub owner_id: AccountId,
}

#[near_bindgen]
impl ParasFactory {
    #[init]
    pub fn new() -> Self {
        Self {
            contracts: LookupSet::new(b"t".to_vec()),
            owner_id: env::predecessor_account_id(),
        }
    }

    #[payable]
    pub fn create_child_contract(
        &mut self,
        collection: AccountId,
        metadata: NFTContractMetadata,
        supply: u32,
        sale: Sale
    ) -> Promise {
        self.assert_sufficient_attached_deposit();
        let subaccount_id = AccountId::new_unchecked(
          format!("{}.{}", collection, env::current_account_id())
        );
        self.assert_contract_id(subaccount_id.to_string());
        let init_args = serde_json::to_vec(&NFTArgs {
            metadata: metadata.clone(),
            owner_id:  env::predecessor_account_id(),
            size: supply,
            sale
        })
            .unwrap();
        Promise::new(subaccount_id.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(CONTRACT_BALANCE)
            .deploy_contract(CODE.to_vec())
            .function_call("new".to_string(), init_args, 0, CREATE_CONTRACT)
            .then(bulk_self::on_create(
                env::predecessor_account_id(),
                metadata,
                subaccount_id,
                env::attached_deposit().into(),
                env::current_account_id(),
                0,
                CREATE_CALLBACK,
            ))
    }
    #[private]
    pub fn on_create(
        &mut self,
        creator_id: AccountId,
        metadata: NFTContractMetadata,
        nft_account_id: AccountId,
        attached_deposit: U128,
    ) {
        let attached_deposit: u128 = attached_deposit.into();
        if is_promise_success() {
            self.contracts.insert(&nft_account_id.to_string());
        } else {
            // Refund
            Promise::new(creator_id).transfer(attached_deposit - CONTRACT_BALANCE);
            env::log_str("failed contract deployment");
        }
    }
    pub fn assert_sufficient_attached_deposit(&self) {
        assert!(
            env::attached_deposit() >= DEPLOY_BALANCE,
            "Not enough attached deposit to complete contract deployment. Need: {}, attached: {}",
            DEPLOY_BALANCE,
            env::attached_deposit()
        );
    }

    pub fn assert_contract_id(
        &self,
        nft_account_id: String,
    ) {
        assert!(
            !self.check_exist(nft_account_id),
            "Collection with that ID already exists"
        );
    }

    pub fn check_exist(
        &self,
        nft_account_id: String,
    ) -> bool {
        self.contracts.contains(&nft_account_id)
    }

}