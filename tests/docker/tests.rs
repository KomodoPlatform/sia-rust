use super::utils::{block_on, mine_blocks, SIA_WALLETD_RPC_URL};

use http::StatusCode;
use sia_rust::transport::client::native::{Conf, NativeClient};
use sia_rust::transport::client::{ApiClient, ApiClientError};
use sia_rust::transport::endpoints::{AddressBalanceRequest, ConsensusTipRequest, GetAddressUtxosRequest,
                                     TxpoolBroadcastRequest};
use sia_rust::types::{Address, Currency, Keypair, SiacoinOutput, SpendPolicy, V2TransactionBuilder};
use std::str::FromStr;
use url::Url;

#[test]
fn test_sia_new_client() {
    let conf = Conf {
        server_url: Url::parse(SIA_WALLETD_RPC_URL).unwrap(),
        password: Some("password".to_string()),
        timeout: None,
    };
    let _api_client = block_on(NativeClient::new(conf)).unwrap();
}

#[test]
fn test_sia_client_bad_auth() {
    let conf = Conf {
        server_url: Url::parse(SIA_WALLETD_RPC_URL).unwrap(),
        password: Some("foo".to_string()),
        timeout: None,
    };
    let result = block_on(NativeClient::new(conf));
    assert!(matches!(
        result,
        Err(ApiClientError::UnexpectedHttpStatus {
            status: StatusCode::UNAUTHORIZED,
            ..
        })
    ));
}

#[test]
fn test_sia_client_consensus_tip() {
    let conf = Conf {
        server_url: Url::parse(SIA_WALLETD_RPC_URL).unwrap(),
        password: Some("password".to_string()),
        timeout: None,
    };
    let api_client = block_on(NativeClient::new(conf)).unwrap();
    let _response = block_on(api_client.dispatcher(ConsensusTipRequest)).unwrap();
}

// This test likely needs to be removed because mine_blocks has possibility of interfering with other async tests
// related to block height
#[test]
fn test_sia_client_address_balance() {
    let conf = Conf {
        server_url: Url::parse(SIA_WALLETD_RPC_URL).unwrap(),
        password: Some("password".to_string()),
        timeout: None,
    };
    let api_client = block_on(NativeClient::new(conf)).unwrap();

    let address =
        Address::from_str("addr:591fcf237f8854b5653d1ac84ae4c107b37f148c3c7b413f292d48db0c25a8840be0653e411f").unwrap();
    mine_blocks(10, &address);

    let request = AddressBalanceRequest { address };
    let response = block_on(api_client.dispatcher(request)).unwrap();

    assert_eq!(response.siacoins, Currency(1000000000000000000000000000000000000));
}

#[test]
fn test_sia_client_build_tx() {
    let conf = Conf {
        server_url: Url::parse(SIA_WALLETD_RPC_URL).unwrap(),
        password: Some("password".to_string()),
        timeout: None,
    };
    let api_client = block_on(NativeClient::new(conf)).unwrap();
    let keypair = Keypair::from_private_bytes(
        &hex::decode("0100000000000000000000000000000000000000000000000000000000000000").unwrap(),
    )
    .unwrap();
    let spend_policy = SpendPolicy::PublicKey(keypair.public());

    let address = spend_policy.address();

    mine_blocks(201, &address);

    let utxos = block_on(api_client.dispatcher(GetAddressUtxosRequest {
        address: address.clone(),
        limit: None,
        offset: None,
    }))
    .unwrap();
    let spend_this = utxos[0].clone();
    let vin = spend_this.clone();
    println!("utxo[0]: {:?}", spend_this);
    let vout = SiacoinOutput {
        value: spend_this.siacoin_output.value,
        address,
    };
    let tx = V2TransactionBuilder::new()
        .add_siacoin_input(vin, spend_policy)
        .add_siacoin_output(vout)
        .sign_simple(vec![&keypair])
        .build();

    let req = TxpoolBroadcastRequest {
        transactions: vec![],
        v2transactions: vec![tx],
    };
    let _response = block_on(api_client.dispatcher(req)).unwrap();
}
