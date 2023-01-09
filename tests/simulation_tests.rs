use near_sdk::AccountId;
use bulk_minting::{CheckResult};
use near_sdk_sim::{
    init_simulator, to_yocto, UserAccount, DEFAULT_GAS
};
use near_sdk::serde_json::json;
mod utils;
pub const FACTORY_CONTRACT_ID: &str = "factory";

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FACTORY_WASM_BYTES => "out/main.wasm",
}

// Added after running simulation test -> with max token series id and 64 byte account
pub const STORAGE_MINT_ESTIMATE: u128 = 11280000000000000000000;
pub const STORAGE_CREATE_SERIES_ESTIMATE: u128 = 8540000000000000000000;
pub const STORAGE_APPROVE: u128 = 2610000000000000000000;

pub fn init() -> (UserAccount, UserAccount, UserAccount) {
    let root = init_simulator(None);
    let factory_contract_id = AccountId::new_unchecked(FACTORY_CONTRACT_ID.to_string());
    let factory_contract = root.deploy(&FACTORY_WASM_BYTES, factory_contract_id.clone(), to_yocto("100"));

    factory_contract.call(
        factory_contract_id,
        "new",
        &json!({
        })
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        0,
    );

    let nft_creator = root.create_user(utils::account_from(&"al"), to_yocto("100"));
    (root, factory_contract, nft_creator)
}

#[test]
fn test_deployment() {
    let (_root, contract, nft_creator) = init();
    nft_creator.call(
        contract.account_id(),
        "create_nft_contract",
        &json!({
            "collection": "newnft",
            "public_key": nft_creator.signer.public_key.to_string(),
            "metadata": {
                "spec": "nft-1.0.0",
                "symbol": "nn-01",
                "name": "NEW NFT",
                "base_uri": "https://google.com"
            },
            "supply": 999,
            "sale": {
                "royalties": {
                    "accounts":{
                       "al": 10000
                    },
                    "percent": 1000
                },
                "initial_royalties": {
                    "accounts":{
                       "al": 10000
                    },
                    "percent": 1000
                },
                "price": to_yocto("1").to_string()
            }
        })
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        to_yocto("10")
    ).assert_success();

    let check_contract: CheckResult = nft_creator.view(
        contract.account_id(),
        "check_contract_exist",
        &json!({
            "nft_account_id": "newnft.factory"
        }).to_string()
            .into_bytes()
    ).unwrap_json();
    assert_eq!(true, check_contract.result);

    let nft_contract_id = AccountId::new_unchecked("newnft.factory".to_string());

    nft_creator.call(
        nft_contract_id,
        "nft_mint_many",
        &json!({
           "num": 3
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        0
    ).assert_success();
}

