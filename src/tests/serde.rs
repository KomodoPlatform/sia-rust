// // FIXME reminder to populate the following tests
// #[test]
// #[ignore]
// fn test_serde_block_id() {
//     test_serde!(
//         BlockID,
//         json!("bid:c67c3b2e57490617a25a9fcb9fd54ab6acbe72fc1e4f1f432cb9334177917667")
//     );
//     test_serde!(BlockID, json!("bid:badc0de"));
//     test_serde!(BlockID, json!("bid:1badc0de"));
//     test_serde!(BlockID, json!("1badc0de"));
//     test_serde!(BlockID, json!(1));
// }

#[test]
#[ignore] // FIXME I don't have a good test case for this yet because wallet_test.go TestEventTypes doesn't output this type
fn test_serde_event_v2_contract_resolution_finalization() {
    use crate::types::Event;
    let j = json!(
      {
        "id": "4057e021e1d6dec8d4e4ef9d6e9fa2e4491c559144848b9af5765e03b39bb69d",
        "index": {
          "height": 0,
          "id": "bid:0000000000000000000000000000000000000000000000000000000000000000"
        },
        "timestamp": "2024-07-12T10:04:18.564506-07:00",
        "maturityHeight": 0,
        "type": "v2ContractResolution",
        "data": {
          "parent": {
            "id": "ee87ab83f9d16c9377d6154c477ac40d2ee70619de2ba146fcfe36fd0de86bf5",
            "leafIndex": 6680213938505633000u64,
            "merkleProof": [
              "0000000000000000000000000000000000000000000000000000000000000000",
              "0000000000000000000000000000000000000000000000000000000000000000",
              "0000000000000000000000000000000000000000000000000000000000000000",
              "0000000000000000000000000000000000000000000000000000000000000000",
              "0000000000000000000000000000000000000000000000000000000000000000"
            ],
            "v2FileContract": {
              "filesize": 0,
              "fileMerkleRoot": "0000000000000000000000000000000000000000000000000000000000000000",
              "proofHeight": 10,
              "expirationHeight": 20,
              "renterOutput": {
                "value": "10000000000000000000000000000",
                "address": "c899f7795bb20c94e57c764f06699e09e6ad071ad95539eef4fb505e79ab22e8be4d64067ccc"
              },
              "hostOutput": {
                "value": "0",
                "address": "000000000000000000000000000000000000000000000000000000000000000089eb0d6a8a69"
              },
              "missedHostValue": "0",
              "totalCollateral": "0",
              "renterPublicKey": "ed25519:65ea9701c409d4457a830b6fe7a2513d6f466ab4e424b3941de9f34a4a2d6170",
              "hostPublicKey": "ed25519:65ea9701c409d4457a830b6fe7a2513d6f466ab4e424b3941de9f34a4a2d6170",
              "revisionNumber": 0,
              "renterSignature": "sig:bd1794b9266fa0de94aea0f0ffb6550efd7e8874133963022413c8ccfe1a0e31c14690d3a5bbd343b160ed59219bd67f79103c45aee07f519d72b5ab4319440f",
              "hostSignature": "sig:bd1794b9266fa0de94aea0f0ffb6550efd7e8874133963022413c8ccfe1a0e31c14690d3a5bbd343b160ed59219bd67f79103c45aee07f519d72b5ab4319440f"
            }
          },
          "type": "finalization",
          "resolution": {
            "filesize": 0,
            "fileMerkleRoot": "0000000000000000000000000000000000000000000000000000000000000000",
            "proofHeight": 10,
            "expirationHeight": 20,
            "renterOutput": {
              "value": "10000000000000000000000000000",
              "address": "c899f7795bb20c94e57c764f06699e09e6ad071ad95539eef4fb505e79ab22e8be4d64067ccc"
            },
            "hostOutput": {
              "value": "0",
              "address": "000000000000000000000000000000000000000000000000000000000000000089eb0d6a8a69"
            },
            "missedHostValue": "0",
            "totalCollateral": "0",
            "renterPublicKey": "ed25519:65ea9701c409d4457a830b6fe7a2513d6f466ab4e424b3941de9f34a4a2d6170",
            "hostPublicKey": "ed25519:65ea9701c409d4457a830b6fe7a2513d6f466ab4e424b3941de9f34a4a2d6170",
            "revisionNumber": 18446744073709551615u64,
            "renterSignature": "sig:bd1794b9266fa0de94aea0f0ffb6550efd7e8874133963022413c8ccfe1a0e31c14690d3a5bbd343b160ed59219bd67f79103c45aee07f519d72b5ab4319440f",
            "hostSignature": "sig:bd1794b9266fa0de94aea0f0ffb6550efd7e8874133963022413c8ccfe1a0e31c14690d3a5bbd343b160ed59219bd67f79103c45aee07f519d72b5ab4319440f"
          }
        }
      }
    );

    let _event = serde_json::from_value::<Event>(j).unwrap();

    // FIXME this should deserialize from a JSON object generated from walletd and recalcuate the txid to check encoding/serde
}

#[cfg(test)]
mod test {
    macro_rules! test_serde {
        ($type:ty, $json_value:expr) => {{
            let json_str = $json_value.to_string();
            let value: $type = serde_json::from_str(&json_str).unwrap();
            let serialized = serde_json::to_string(&value).unwrap();
            let serialized_json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
            assert_eq!($json_value, serialized_json_value);
        }};
    }
    // Ensure the original value matches the value after round-trip (serialize -> deserialize -> serialize)
    use crate::types::{Address, Event, Hash256, SiacoinElement, SiacoinOutput, StateElement, UnlockKey, V2Transaction};

    cross_target_tests! {
            fn test_serde_address() {
                test_serde!(
                    Address,
                    json!("591fcf237f8854b5653d1ac84ae4c107b37f148c3c7b413f292d48db0c25a8840be0653e411f")
                );
            }

            fn test_serde_unlock_key() {
                test_serde!(
                    UnlockKey,
                    json!("ed25519:0102030000000000000000000000000000000000000000000000000000000000")
                );
            }

            fn test_serde_sia_hash() {
                test_serde!(
                    Hash256,
                    json!("dc07e5bf84fbda867a7ed7ca80c6d1d81db05cef16ff38f6ba80b6bf01e1ddb1")
                );
            }

            fn test_serde_siacoin_output() {
                let j = json!({
                    "value": "300000000000000000000000000000",
                    "address": "591fcf237f8854b5653d1ac84ae4c107b37f148c3c7b413f292d48db0c25a8840be0653e411f"
                });
                test_serde!(SiacoinOutput, j);
            }

            // check that merkleProof field serde is the same when it is null, missing or empty
            fn test_serde_state_element() {
                let j = json!({
                    "id": "dc07e5bf84fbda867a7ed7ca80c6d1d81db05cef16ff38f6ba80b6bf01e1ddb1",
                    "leafIndex": 21,
                    "merkleProof": null
                });
                let null_proof = serde_json::from_value::<StateElement>(j).unwrap();

                let j = json!({
                    "id": "dc07e5bf84fbda867a7ed7ca80c6d1d81db05cef16ff38f6ba80b6bf01e1ddb1",
                    "leafIndex": 21,
                    "merkleProof": []
                });
                let empty_proof = serde_json::from_value::<StateElement>(j).unwrap();

                let j = json!({
                    "id": "dc07e5bf84fbda867a7ed7ca80c6d1d81db05cef16ff38f6ba80b6bf01e1ddb1",
                    "leafIndex": 21
                });
                let missing_proof = serde_json::from_value::<StateElement>(j).unwrap();

                assert_eq!(null_proof, empty_proof);
                assert_eq!(null_proof, missing_proof);
            }

            fn test_serde_siacoin_element() {
                let j = json!(  {
                    "id": "0102030000000000000000000000000000000000000000000000000000000000",
                    "stateElement": {
                        "leafIndex": 1,
                        "merkleProof": [
                            "0405060000000000000000000000000000000000000000000000000000000000",
                            "0708090000000000000000000000000000000000000000000000000000000000"
                        ]
                    },
                    "siacoinOutput": {
                        "value": "1",
                        "address": "72b0762b382d4c251af5ae25b6777d908726d75962e5224f98d7f619bb39515dd64b9a56043a"
                    },
                    "maturityHeight": 0
                }
            );
            serde_json::from_value::<SiacoinElement>(j).unwrap();
        }

        fn test_serde_siacoin_element_missing_merkle_proof() {
            let json_str = r#"
            {
                "id": "16406893374eb18eeea95e8c0d6b6c325275ecb99cf2fec7a6708b0b8def75bd",
                "stateElement": {
                    "leafIndex": 391
                },
                "siacoinOutput": {
                    "value": "10000000000000000000000000000",
                    "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                },
                "maturityHeight": 334
            }"#;
        serde_json::from_str::<SiacoinElement>(json_str).unwrap();
    }

    fn test_serde_event_v2_contract_resolution_storage_proof() {
        let j = r#"
            {
                "id": "16406893374eb18eeea95e8c0d6b6c325275ecb99cf2fec7a6708b0b8def75bd",
                "index": {
                "height": 190,
                "id": "22693d8885ad7b5e2abf22fe838fd6ae9856142f898607ffd2ddb8dd3d7ca67b"
                },
                "confirmations": 17,
                "type": "v2ContractResolution",
                "data": {
                "resolution": {
                    "parent": {
                    "id": "e5adb3e8e49d9bd29e54966e809cc652f08dfca2183fad00f3da29df83f65091",
                    "stateElement": {
                        "leafIndex": 351
                    },
                    "v2FileContract": {
                        "capacity": 0,
                        "filesize": 0,
                        "fileMerkleRoot": "0000000000000000000000000000000000000000000000000000000000000000",
                        "proofHeight": 179,
                        "expirationHeight": 189,
                        "renterOutput": {
                        "value": "10000000000000000000000000000",
                        "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                        },
                        "hostOutput": {
                        "value": "0",
                        "address": "000000000000000000000000000000000000000000000000000000000000000089eb0d6a8a69"
                        },
                        "missedHostValue": "0",
                        "totalCollateral": "0",
                        "renterPublicKey": "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc",
                        "hostPublicKey": "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc",
                        "revisionNumber": 0,
                        "renterSignature": "88b5f53a69759264f60cb227e7d4fdb25ee185f9c9b9bcf4f6e94c413ace76e1d1dcf72d509670e3d4e89d3dccb326d9c74411909e0a2e0e7e1e18bf3acb6c0c",
                        "hostSignature": "88b5f53a69759264f60cb227e7d4fdb25ee185f9c9b9bcf4f6e94c413ace76e1d1dcf72d509670e3d4e89d3dccb326d9c74411909e0a2e0e7e1e18bf3acb6c0c"
                    }
                    },
                    "type": "expiration",
                    "resolution": {}
                },
                "siacoinElement": {
                    "id": "16406893374eb18eeea95e8c0d6b6c325275ecb99cf2fec7a6708b0b8def75bd",
                    "stateElement": {
                    "leafIndex": 391
                    },
                    "siacoinOutput": {
                    "value": "10000000000000000000000000000",
                    "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                    },
                    "maturityHeight": 334
                },
                "missed": true
                },
                "maturityHeight": 334,
                "timestamp": "2024-11-15T19:41:06Z",
                "relevant": [
                "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                ]
            }
        "#;

        let _event = serde_json::from_str::<Event>(j).unwrap();
    }

    fn test_serde_event_v2_contract_resolution_renewal() {
        let json_str = r#"
            {
                "id": "5d565129957e1493902123f6d58775593a53ccbff1e30342defaf563853c30b4",
                "index": {
                  "height": 203,
                  "id": "f5674e39f155f1d5afe6cd2315a8b6c89843c1fbc19b13d8c6b3636b20cb537c"
                },
                "confirmations": 4,
                "type": "v2ContractResolution",
                "data": {
                  "resolution": {
                    "parent": {
                      "id": "d219a1300698e798338df61f6f816f593672f71bce274d5130e1ba95e1d63814",
                      "stateElement": {
                        "leafIndex": 423
                      },
                      "v2FileContract": {
                        "capacity": 0,
                        "filesize": 0,
                        "fileMerkleRoot": "0000000000000000000000000000000000000000000000000000000000000000",
                        "proofHeight": 211,
                        "expirationHeight": 221,
                        "renterOutput": {
                          "value": "10000000000000000000000000000",
                          "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                        },
                        "hostOutput": {
                          "value": "0",
                          "address": "000000000000000000000000000000000000000000000000000000000000000089eb0d6a8a69"
                        },
                        "missedHostValue": "0",
                        "totalCollateral": "0",
                        "renterPublicKey": "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc",
                        "hostPublicKey": "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc",
                        "revisionNumber": 0,
                        "renterSignature": "3aaf47eb60d992bced4818291eb1b7773e20a731df48857474715602db31a12fddf29170337803f6dd1ce95e1e2043714c2b3bcb99925ea37ad2cf4880922c02",
                        "hostSignature": "3aaf47eb60d992bced4818291eb1b7773e20a731df48857474715602db31a12fddf29170337803f6dd1ce95e1e2043714c2b3bcb99925ea37ad2cf4880922c02"
                      }
                    },
                    "type": "renewal",
                    "resolution": {
                      "finalRevision": {
                        "capacity": 0,
                        "filesize": 0,
                        "fileMerkleRoot": "0000000000000000000000000000000000000000000000000000000000000000",
                        "proofHeight": 211,
                        "expirationHeight": 221,
                        "renterOutput": {
                          "value": "10000000000000000000000000000",
                          "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                        },
                        "hostOutput": {
                          "value": "0",
                          "address": "000000000000000000000000000000000000000000000000000000000000000089eb0d6a8a69"
                        },
                        "missedHostValue": "0",
                        "totalCollateral": "0",
                        "renterPublicKey": "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc",
                        "hostPublicKey": "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc",
                        "revisionNumber": 18446744073709551615,
                        "renterSignature": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                        "hostSignature": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                      },
                      "newContract": {
                        "capacity": 0,
                        "filesize": 0,
                        "fileMerkleRoot": "0000000000000000000000000000000000000000000000000000000000000000",
                        "proofHeight": 221,
                        "expirationHeight": 231,
                        "renterOutput": {
                          "value": "10000000000000000000000000000",
                          "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                        },
                        "hostOutput": {
                          "value": "0",
                          "address": "000000000000000000000000000000000000000000000000000000000000000089eb0d6a8a69"
                        },
                        "missedHostValue": "0",
                        "totalCollateral": "0",
                        "renterPublicKey": "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc",
                        "hostPublicKey": "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc",
                        "revisionNumber": 0,
                        "renterSignature": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                        "hostSignature": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                      },
                      "renterRollover": "0",
                      "hostRollover": "0",
                      "renterSignature": "f43d4b5d859931669f7db479af2e3064ed40cfa333b625120521f20f9cf9867b2c38d95cc2ee6f2d75e438ad6c25ce9f7b436e6ccbe70237f0b66e2d1dae720f",
                      "hostSignature": "f43d4b5d859931669f7db479af2e3064ed40cfa333b625120521f20f9cf9867b2c38d95cc2ee6f2d75e438ad6c25ce9f7b436e6ccbe70237f0b66e2d1dae720f"
                    }
                  },
                  "siacoinElement": {
                    "id": "5d565129957e1493902123f6d58775593a53ccbff1e30342defaf563853c30b4",
                    "stateElement": {
                      "leafIndex": 427
                    },
                    "siacoinOutput": {
                      "value": "10000000000000000000000000000",
                      "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                    },
                    "maturityHeight": 347
                  },
                  "missed": false
                },
                "maturityHeight": 347,
                "timestamp": "2024-11-15T19:41:06Z",
                "relevant": [
                  "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                ]
              }
        "#;

        let _event = serde_json::from_str::<Event>(json_str).unwrap();
    }

    #[ignore] // FIXME Error("expected an empty map for expiration", line: 0, column: 0)
    fn test_serde_event_v2_contract_resolution_expiration() {
        let j = json!(
            {
                "id": "4c0170b9e82eacc2d14a13b974ce0c03560358276f135403bd060b53ce53be1c",
                "index": {
                    "height": 190,
                    "id": "bid:730f554f8cd5e6bd855b21b8c53f59808f3aa7351093f44da7761181283e3c6b"
                },
                "timestamp": "2024-07-18T19:04:16Z",
                "maturityHeight": 334,
                "type": "v2ContractResolution",
                "data": {
                    "resolution": {
                        "parent": {
                            "id": "34f6bb9b9ed58dedebce2f39d29a526ea3012e9ae005cfca6a5257761c5412f6",
                            "leafIndex": 351,
                            "merkleProof": [
                            "e805430ecdd47bcaca574f78721c3b6a24f0a877110fc9fa7ab347fd231a9885",
                            "70782818a59e512d4995efd4ee94299e601496011b9c42b47eb0a3cd65aa89c9",
                            "42ab48d2ef2b54352d44ab2ef33c1a6d76589360c0dd556d703a452b7d3e4a2c",
                            "4af61bcae0a46d70f9b826b9bace336647389c38e6cb4c54356b9dd7fd6060aa",
                            "59d21dd10aa3def083106844e23ad7f6b93e309c80b24a03e2c9b6eba8acef33",
                            "f95c3f0fc4d632e5da8adcaa9249ea6b0c5fe66466a951871f5dc30a0c96b76d",
                            "3374baebf913a23e0b9811ae22e72f6cdf6999d332ccda4b4dbab87f58b2a574"
                            ],
                            "v2FileContract": {
                                "filesize": 0,
                                "fileMerkleRoot": "0000000000000000000000000000000000000000000000000000000000000000",
                                "proofHeight": 179,
                                "expirationHeight": 189,
                                "renterOutput": {
                                    "value": "10000000000000000000000000000",
                                    "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                                },
                                "hostOutput": {
                                    "value": "0",
                                    "address": "000000000000000000000000000000000000000000000000000000000000000089eb0d6a8a69"
                                },
                                "missedHostValue": "0",
                                "totalCollateral": "0",
                                "renterPublicKey": "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc",
                                "hostPublicKey": "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc",
                                "revisionNumber": 0,
                                "renterSignature": "sig:c293b22c9feee5a081699ddbf83486704df855129c2bbe27c2dc56afcb7e68cd355785fa36954471c1e48691864b240969168422b1fd6396e18f720ebec50e00",
                                "hostSignature": "sig:c293b22c9feee5a081699ddbf83486704df855129c2bbe27c2dc56afcb7e68cd355785fa36954471c1e48691864b240969168422b1fd6396e18f720ebec50e00"
                            }
                        },
                        "type": "expiration",
                        "resolution": {}
                    },
                    "siacoinElement": {
                        "id": "4c0170b9e82eacc2d14a13b974ce0c03560358276f135403bd060b53ce53be1c",
                        "leafIndex": 391,
                        "merkleProof": null,
                        "siacoinOutput": {
                            "value": "10000000000000000000000000000",
                            "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                        },
                        "maturityHeight": 334
                    },
                    "missed": true
                }
            }
        );

        let _event = serde_json::from_value::<Event>(j).unwrap();
    }

    fn test_serde_event_v2_transaction() {
        let j = json!(
            {
                "id": "5900e475aace932c94bcc94cf296596ccff1d77d9aba52a079e9f429605671cd",
                "index": {
                    "height": 203,
                    "id": "bid:bd04c08bb96203c7f24adf2d405cb1069c7da8573573011379a986be62fc2a29"
                },
                "timestamp": "2024-07-18T19:04:16Z",
                "maturityHeight": 203,
                "type": "v2Transaction",
                "data": {
                    "siacoinInputs": [
                    {
                        "parent": {
                            "id": "78d58090bcdeaccf22abf99b6e0de25273e9eb82210359a16cefbd743a85fd50",
                            "leafIndex": 421,
                            "merkleProof": [
                            "f26accb7c256e867a9ed62671ebe6c3eb34d085e5266f67073af2daa549f980d",
                            "d39e139147168c70da11c3f6db4fa54d35914ef67ba5654a75107da9c099ddda",
                            "f447a5360e1a7c4cab3062dd1699f56ea642b4f6cc6464fdfca0d1aa15fa436c"
                            ],
                            "siacoinOutput": {
                                "value": "256394172736732570239334030000",
                                "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                            },
                            "maturityHeight": 0
                        },
                        "satisfiedPolicy": {
                            "policy": {
                                "type": "uc",
                                "policy": {
                                    "timelock": 0,
                                    "publicKeys": [
                                    "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc"
                                    ],
                                    "signaturesRequired": 1
                                }
                            },
                            "signatures": [
                            "sig:c432fea5f147205e49235ddbd75c232fd8e9c7526b2b1575f70653ae2b3c0d0338c7fe710be338482060cf6ef2dea5e2319252fc28deaf70c77a2be60a533400"
                            ]
                        }
                    }
                    ],
                    "siacoinOutputs": [
                    {
                        "value": "10400000000000000000000000000",
                        "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                    },
                    {
                        "value": "245994172736732570239334030000",
                        "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                    }
                    ],
                    "minerFee": "0"
                }
            }
        );
        test_serde!(Event, j);
    }

    fn test_v2_transaction_serde_basic_send() {
        let j = json!(
            {
                "siacoinInputs": [
                {
                    "parent": {
                        "id": "f59e395dc5cbe3217ee80eff60585ffc9802e7ca580d55297782d4a9b4e08589",
                        "leafIndex": 3,
                        "merkleProof": [
                        "ab0e1726444c50e2c0f7325eb65e5bd262a97aad2647d2816c39d97958d9588a",
                        "467e2be4d8482eca1f99440b6efd531ab556d10a8371a98a05b00cb284620cf0",
                        "64d5766fce1ff78a13a4a4744795ad49a8f8d187c01f9f46544810049643a74a",
                        "31d5151875152bc25d1df18ca6bbda1bef5b351e8d53c277791ecf416fcbb8a8",
                        "12a92a1ba87c7b38f3c4e264c399abfa28fb46274cfa429605a6409bd6d0a779",
                        "eda1d58a9282dbf6c3f1beb4d6c7bdc036d14a1cfee8ab1e94fabefa9bd63865",
                        "e03dee6e27220386c906f19fec711647353a5f6d76633a191cbc2f6dce239e89",
                        "e70fcf0129c500f7afb49f4f2bb82950462e952b7cdebb2ad0aa1561dc6ea8eb"
                        ],
                        "siacoinOutput": {
                            "value": "300000000000000000000000000000",
                            "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                        },
                        "maturityHeight": 145
                    },
                    "satisfiedPolicy": {
                        "policy": {
                            "type": "uc",
                            "policy": {
                                "timelock": 0,
                                "publicKeys": [
                                "ed25519:cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc"
                                ],
                                "signaturesRequired": 1
                            }
                        },
                        "signatures": [
                        "sig:f0a29ba576eb0dbc3438877ac1d3a6da4f3c4cbafd9030709c8a83c2fffa64f4dd080d37444261f023af3bd7a10a9597c33616267d5371bf2c0ade5e25e61903"
                        ]
                    }
                }
                ],
                "siacoinOutputs": [
                {
                    "value": "1000000000000000000000000000",
                    "address": "000000000000000000000000000000000000000000000000000000000000000089eb0d6a8a69"
                },
                {
                    "value": "299000000000000000000000000000",
                    "address": "f7843ac265b037658b304468013da4fd0f304a1b73df0dc68c4273c867bfa38d01a7661a187f"
                }
                ],
                "minerFee": "0"
            }
        );
        let tx = serde_json::from_value::<V2Transaction>(j).unwrap();

        let j2 = serde_json::to_value(&tx).unwrap().to_string();
        let tx2 = serde_json::from_str::<V2Transaction>(&j2).unwrap();
        assert_eq!(tx, tx2);
    }
    }
}
