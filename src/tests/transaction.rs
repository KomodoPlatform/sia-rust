#[cfg(test)]
mod test {
    use crate::encoding::Encoder;
    use crate::types::{Address, Attestation, Currency, CurrencyVersion, FileContractRevisionV2, Hash256, Keypair,
                       Preimage, PublicKey, SatisfiedPolicy, SiacoinElement, SiacoinInputV1, SiacoinInputV2,
                       SiacoinOutput, SiacoinOutputId, SiacoinOutputVersion, Signature, SpendPolicy, StateElement,
                       UnlockCondition, V2FileContract, V2FileContractElement, V2Transaction};
    use std::convert::TryFrom;
    use std::str::FromStr;

    cross_target_tests! {
        fn test_siacoin_input_encode() {
            let public_key = PublicKey::from_bytes(
                &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();
            let unlock_condition = UnlockCondition::new(vec![public_key], 0, 1);

            let vin = SiacoinInputV1 {
                parent_id: Hash256::from_str("0405060000000000000000000000000000000000000000000000000000000000")
                    .unwrap()
                    .into(),
                unlock_condition,
            };

            let hash = Encoder::encode_and_hash(&vin);
            let expected = Hash256::from_str("1d4b77aaa82c71ca68843210679b380f9638f8bec7addf0af16a6536dd54d6b4").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_siacoin_currency_encode_v1() {
            let currency: Currency = 1u64.into();

            let hash = Encoder::encode_and_hash(&CurrencyVersion::V1(&currency));
            let expected = Hash256::from_str("a1cc3a97fc1ebfa23b0b128b153a29ad9f918585d1d8a32354f547d8451b7826").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_siacoin_currency_encode_v2() {
            let currency: Currency = 1u64.into();

            let hash = Encoder::encode_and_hash(&CurrencyVersion::V2(&currency));
            let expected = Hash256::from_str("a3865e5e284e12e0ea418e73127db5d1092bfb98ed372ca9a664504816375e1d").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_siacoin_currency_encode_v1_max() {
            let currency = Currency(u128::MAX);

            let hash = Encoder::encode_and_hash(&CurrencyVersion::V1(&currency));
            let expected = Hash256::from_str("4b9ed7269cb15f71ddf7238172a593a8e7ffe68b12c1bf73d67ac8eec44355bb").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_siacoin_currency_encode_v2_max() {
            let currency = Currency(u128::MAX);

            let hash = Encoder::encode_and_hash(&CurrencyVersion::V2(&currency));
            let expected = Hash256::from_str("681467b3337425fd38fa3983531ca1a6214de9264eebabdf9c9bc5d157d202b4").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_siacoin_output_encode_v1() {
            let vout = SiacoinOutput {
                value: 1u64.into(),
                address: Address::from_str("72b0762b382d4c251af5ae25b6777d908726d75962e5224f98d7f619bb39515dd64b9a56043a")
                    .unwrap(),
            };

            let hash = Encoder::encode_and_hash(&SiacoinOutputVersion::V1(&vout));
            let expected = Hash256::from_str("3253c57e76600721f2bdf03497a71ed47c09981e22ef49aed92e40da1ea91b28").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_siacoin_output_encode_v2() {
            let vout = SiacoinOutput {
                value: 1u64.into(),
                address: Address::from_str("72b0762b382d4c251af5ae25b6777d908726d75962e5224f98d7f619bb39515dd64b9a56043a")
                    .unwrap(),
            };

            let hash = Encoder::encode_and_hash(&SiacoinOutputVersion::V2(&vout));
            let expected = Hash256::from_str("c278eceae42f594f5f4ca52c8a84b749146d08af214cc959ed2aaaa916eaafd3").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_siacoin_element_encode() {
            let state_element = StateElement {
                leaf_index: 1,
                merkle_proof: vec![
                    Hash256::from_str("0405060000000000000000000000000000000000000000000000000000000000").unwrap(),
                    Hash256::from_str("0708090000000000000000000000000000000000000000000000000000000000").unwrap(),
                ],
            };
            let siacoin_element = SiacoinElement {
                id: Hash256::from_str("0102030000000000000000000000000000000000000000000000000000000000").unwrap().into(),
                state_element,
                siacoin_output: SiacoinOutput {
                    value: 1u64.into(),
                    address: Address::from_str(
                        "72b0762b382d4c251af5ae25b6777d908726d75962e5224f98d7f619bb39515dd64b9a56043a",
                    )
                    .unwrap(),
                },
                maturity_height: 0,
            };

            let hash = Encoder::encode_and_hash(&siacoin_element);
            let expected = Hash256::from_str("4c46cbe535099409d2ea4255debda3fb62993595e305c78688ec4306f8464d7d").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_state_element_encode() {
            let state_element = StateElement {
                leaf_index: 1,
                merkle_proof: vec![
                    Hash256::from_str("0405060000000000000000000000000000000000000000000000000000000000").unwrap(),
                    Hash256::from_str("0708090000000000000000000000000000000000000000000000000000000000").unwrap(),
                ],
            };

            let hash = Encoder::encode_and_hash(&state_element);
            let expected = Hash256::from_str("70f868873fcb6196cd54bbb1e9e480188043426d3f7c9dc8fc5a7a536981cef1").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_state_element_encode_null_merkle_proof() {
            let j = r#"{"leafIndex":1}"#;
            let state_element = serde_json::from_str::<StateElement>(j).unwrap();

            let hash = Encoder::encode_and_hash(&state_element);
            let expected = Hash256::from_str("a3865e5e284e12e0ea418e73127db5d1092bfb98ed372ca9a664504816375e1d").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_state_element_encode_empty_merkle_proof() {
            let j = r#"{"leafIndex":1,"merkleProof":[]}"#;
            let state_element = serde_json::from_str::<StateElement>(j).unwrap();

            let hash = Encoder::encode_and_hash(&state_element);
            let expected = Hash256::from_str("a3865e5e284e12e0ea418e73127db5d1092bfb98ed372ca9a664504816375e1d").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_siacoin_input_encode_v1() {
            let vin = SiacoinInputV1 {
                parent_id: Hash256::default().into(),
                unlock_condition: UnlockCondition::new(vec![], 0, 0),
            };

            let hash = Encoder::encode_and_hash(&vin);
            let expected = Hash256::from_str("2f806f905436dc7c5079ad8062467266e225d8110a3c58d17628d609cb1c99d0").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_signature_encode() {
            let signature = Signature::try_from(
                hex::decode("105641BF4AE119CB15617FC9658BEE5D448E2CC27C9BC3369F4BA5D0E1C3D01EBCB21B669A7B7A17CF8457189EAA657C41D4A2E6F9E0F25D0996D3A17170F309").unwrap().as_ref()).unwrap();

            let hash = Encoder::encode_and_hash(&signature);
            let expected = Hash256::from_str("1e6952fe04eb626ae759a0090af2e701ba35ee6ad15233a2e947cb0f7ae9f7c7").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_satisfied_policy_encode_public_key() {
            let public_key = PublicKey::from_bytes(
                &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();

            let policy = SpendPolicy::PublicKey(public_key);

            let signature = Signature::try_from(
                hex::decode("105641BF4AE119CB15617FC9658BEE5D448E2CC27C9BC3369F4BA5D0E1C3D01EBCB21B669A7B7A17CF8457189EAA657C41D4A2E6F9E0F25D0996D3A17170F309").unwrap()).unwrap();

            let satisfied_policy = SatisfiedPolicy {
                policy,
                signatures: vec![signature],
                preimages: vec![],
            };

            let hash = Encoder::encode_and_hash(&satisfied_policy);
            let expected = Hash256::from_str("51832be911c7382502a2011cbddf1a9f689c4ca08c6a83ae3d021fb0dc781822").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_satisfied_policy_encode_hash_empty() {
            let policy = SpendPolicy::Hash(Hash256::default());

            let satisfied_policy = SatisfiedPolicy {
                policy,
                signatures: vec![],
                preimages: vec![Preimage::default()],
            };

            let hash = Encoder::encode_and_hash(&satisfied_policy);
            let expected = Hash256::from_str("1e612d1ee36338b93a36bac0c52007a2d678cde0bd9b95c36a1f61166cf02b87").unwrap();
            assert_eq!(hash, expected);
        }

        // Adding a signature to SatisfiedPolicy of PolicyHash should have no effect
        fn test_satisfied_policy_encode_hash_frivulous_signature() {
            let policy = SpendPolicy::Hash(Hash256::default());

            let mut preimage = [0u8; 32];
            preimage[..4].copy_from_slice(&[1, 2, 3, 4]);

            let satisfied_policy = SatisfiedPolicy {
                policy,
                signatures: vec!(Signature::try_from(
                    hex::decode("105641BF4AE119CB15617FC9658BEE5D448E2CC27C9BC3369F4BA5D0E1C3D01EBCB21B669A7B7A17CF8457189EAA657C41D4A2E6F9E0F25D0996D3A17170F309").unwrap()).unwrap()),
                preimages: vec!(preimage.into()),
            };

            let hash = Encoder::encode_and_hash(&satisfied_policy);
            let expected = Hash256::from_str("80f3caa4507615945bc839c8505546decd91e9642120f26938b2fc370fa61992").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_satisfied_policy_encode_hash() {
            let policy = SpendPolicy::Hash(Hash256::default());

            let mut preimage = [0u8; 32];
            preimage[..4].copy_from_slice(&[1, 2, 3, 4]);
            let satisfied_policy = SatisfiedPolicy {
                policy,
                signatures: vec![],
                preimages: vec![preimage.into()],
            };

            let hash = Encoder::encode_and_hash(&satisfied_policy);
            let expected = Hash256::from_str("80f3caa4507615945bc839c8505546decd91e9642120f26938b2fc370fa61992").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_satisfied_policy_encode_unlock_condition_standard() {
            let pubkey = PublicKey::from_bytes(
                &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();

            let unlock_condition = UnlockCondition::new(vec![pubkey], 0, 1);

            let policy = SpendPolicy::UnlockConditions(unlock_condition);

            let signature = Signature::try_from(
                hex::decode("105641BF4AE119CB15617FC9658BEE5D448E2CC27C9BC3369F4BA5D0E1C3D01EBCB21B669A7B7A17CF8457189EAA657C41D4A2E6F9E0F25D0996D3A17170F309").unwrap()).unwrap();

            let satisfied_policy = SatisfiedPolicy {
                policy,
                signatures: vec![signature],
                preimages: vec![],
            };

            let hash = Encoder::encode_and_hash(&satisfied_policy);
            let expected = Hash256::from_str("c749f9ac53395ec557aed7e21d202f76a58e0de79222e5756b27077e9295931f").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_satisfied_policy_encode_unlock_condition_complex() {
            let pubkey0 = PublicKey::from_bytes(
                &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();
            let pubkey1 = PublicKey::from_bytes(
                &hex::decode("06C87838297B7BB16AB23946C99DFDF77FF834E35DB07D71E9B1D2B01A11E96D").unwrap(),
            )
            .unwrap();
            let pubkey2 = PublicKey::from_bytes(
                &hex::decode("BE043906FD42297BC0A03CAA6E773EF27FC644261C692D090181E704BE4A88C3").unwrap(),
            )
            .unwrap();

            let unlock_condition = UnlockCondition::new(vec![pubkey0, pubkey1, pubkey2], 77777777, 3);

            let policy = SpendPolicy::UnlockConditions(unlock_condition);

            let sig0 = Signature::try_from(
                hex::decode("105641BF4AE119CB15617FC9658BEE5D448E2CC27C9BC3369F4BA5D0E1C3D01EBCB21B669A7B7A17CF8457189EAA657C41D4A2E6F9E0F25D0996D3A17170F309").unwrap()).unwrap();
            let sig1 = Signature::try_from(
                hex::decode("0734761D562958F6A82819474171F05A40163901513E5858BFF9E4BD9CAFB04DEF0D6D345BACE7D14E50C5C523433B411C7D7E1618BE010A63C55C34A2DEE70A").unwrap()).unwrap();
            let sig2 = Signature::try_from(
                hex::decode("482A2A905D7A6FC730387E06B45EA0CF259FCB219C9A057E539E705F60AC36D7079E26DAFB66ED4DBA9B9694B50BCA64F1D4CC4EBE937CE08A34BF642FAC1F0C").unwrap()).unwrap();

            let satisfied_policy = SatisfiedPolicy {
                policy,
                signatures: vec![sig0, sig1, sig2],
                preimages: vec![],
            };

            let hash = Encoder::encode_and_hash(&satisfied_policy);
            let expected = Hash256::from_str("13806b6c13a97478e476e0e5a0469c9d0ad8bf286bec0ada992e363e9fc60901").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_satisfied_policy_encode_threshold_simple() {
            let sub_policy = SpendPolicy::Hash(Hash256::default());
            let policy = SpendPolicy::Threshold {
                n: 1,
                of: vec![sub_policy],
            };
            let mut preimage = [0u8; 32];
            preimage[..4].copy_from_slice(&[1, 2, 3, 4]);
            let satisfied_policy = SatisfiedPolicy {
                policy,
                signatures: vec![],
                preimages: vec![preimage.into()],
            };

            let hash = Encoder::encode_and_hash(&satisfied_policy);
            // FIXME update this in go equivalent. Preimage was changed from Vec<u8> to [u8; 32]
            let expected = Hash256::from_str("2200a1464864cfaea8d312c1f16b5e00b816110896bea32ef7e1ccd43042d312").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_satisfied_policy_encode_threshold_atomic_swap_success() {
            let alice_pubkey = PublicKey::from_bytes(
                &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();
            let bob_pubkey = PublicKey::from_bytes(
                &hex::decode("06C87838297B7BB16AB23946C99DFDF77FF834E35DB07D71E9B1D2B01A11E96D").unwrap(),
            )
            .unwrap();

            let secret_hash = Hash256::from_str("0100000000000000000000000000000000000000000000000000000000000000").unwrap();

            let policy = SpendPolicy::atomic_swap_success(&alice_pubkey, &bob_pubkey, 77777777, &secret_hash);
            let signature = Signature::try_from(
                hex::decode("105641BF4AE119CB15617FC9658BEE5D448E2CC27C9BC3369F4BA5D0E1C3D01EBCB21B669A7B7A17CF8457189EAA657C41D4A2E6F9E0F25D0996D3A17170F309").unwrap()).unwrap();

            let mut preimage = [0u8; 32];
            preimage[..4].copy_from_slice(&[1, 2, 3, 4]);
            let satisfied_policy = SatisfiedPolicy {
                policy,
                signatures: vec![signature],
                preimages: vec![preimage.into()],
            };

            let hash = Encoder::encode_and_hash(&satisfied_policy);
            // FIXME update this in go equivalent. Preimage was changed from Vec<u8> to [u8; 32]
            let expected = Hash256::from_str("08852e4ad99f726120028ecd82925b5f55fa441952cfc034a5cf4f09159b9372").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_satisfied_policy_encode_threshold_atomic_swap_refund() {
            let alice_pubkey = PublicKey::from_bytes(
                &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();
            let bob_pubkey = PublicKey::from_bytes(
                &hex::decode("06C87838297B7BB16AB23946C99DFDF77FF834E35DB07D71E9B1D2B01A11E96D").unwrap(),
            )
            .unwrap();

            let secret_hash = Hash256::from_str("0100000000000000000000000000000000000000000000000000000000000000").unwrap();

            let policy = SpendPolicy::atomic_swap_refund(&alice_pubkey, &bob_pubkey, 77777777, &secret_hash);
            let signature = Signature::try_from(
                hex::decode("105641BF4AE119CB15617FC9658BEE5D448E2CC27C9BC3369F4BA5D0E1C3D01EBCB21B669A7B7A17CF8457189EAA657C41D4A2E6F9E0F25D0996D3A17170F309").unwrap()).unwrap();

            let mut preimage = [0u8; 32];
            preimage[..4].copy_from_slice(&[1, 2, 3, 4]);
            let satisfied_policy = SatisfiedPolicy {
                policy,
                signatures: vec![signature],
                preimages: vec![preimage.into()],
            };

            let hash = Encoder::encode_and_hash(&satisfied_policy);
            let expected = Hash256::from_str("8975e8cf990d5a20d9ec3dae18ed3b3a0c92edf967a8d93fcdef6a1eb73bb348").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_siacoin_input_encode_v2() {
            let sub_policy = SpendPolicy::Hash(Hash256::default());
            let policy = SpendPolicy::Threshold {
                n: 1,
                of: vec![sub_policy],
            };
            let mut preimage = [0u8; 32];
            preimage[..4].copy_from_slice(&[1, 2, 3, 4]);

            let satisfied_policy = SatisfiedPolicy {
                policy: policy.clone(),
                signatures: vec![],
                preimages: vec![preimage.into()],
            };

            let vin = SiacoinInputV2 {
                parent: SiacoinElement {
                    id: SiacoinOutputId::default(),
                    state_element: StateElement {
                        leaf_index: 0,
                        merkle_proof: vec![Hash256::default()],
                    },
                    siacoin_output: SiacoinOutput {
                        value: 1u64.into(),
                        address: policy.address(),
                    },
                    maturity_height: 0,
                },
                satisfied_policy,
            };

            let hash = Encoder::encode_and_hash(&vin);
            // FIXME update this in go equivalent. Preimage was changed from Vec<u8> to [u8; 32]
            let expected = Hash256::from_str("d31a05b155113a5244f14ae833887fd8b30f555129be126ca4b90592290db24a").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_attestation_encode() {
            let public_key = PublicKey::from_bytes(
                &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();
            let signature = Signature::try_from(
                hex::decode("105641BF4AE119CB15617FC9658BEE5D448E2CC27C9BC3369F4BA5D0E1C3D01EBCB21B669A7B7A17CF8457189EAA657C41D4A2E6F9E0F25D0996D3A17170F309").unwrap()).unwrap();

            let attestation = Attestation {
                public_key,
                key: "HostAnnouncement".to_string(),
                value: vec![1u8, 2u8, 3u8, 4u8],
                signature,
            };

            let hash = Encoder::encode_and_hash(&attestation);
            let expected = Hash256::from_str("b28b32c6f91d1b57ab4a9ea9feecca16b35bb8febdee6a0162b22979415f519d").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_file_contract_v2_encode() {
            let pubkey0 = PublicKey::from_bytes(
                &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();
            let pubkey1 = PublicKey::from_bytes(
                &hex::decode("06C87838297B7BB16AB23946C99DFDF77FF834E35DB07D71E9B1D2B01A11E96D").unwrap(),
            )
            .unwrap();

            let sig0 = Signature::try_from(
                hex::decode("105641BF4AE119CB15617FC9658BEE5D448E2CC27C9BC3369F4BA5D0E1C3D01EBCB21B669A7B7A17CF8457189EAA657C41D4A2E6F9E0F25D0996D3A17170F309").unwrap()).unwrap();
            let sig1 = Signature::try_from(
                hex::decode("0734761D562958F6A82819474171F05A40163901513E5858BFF9E4BD9CAFB04DEF0D6D345BACE7D14E50C5C523433B411C7D7E1618BE010A63C55C34A2DEE70A").unwrap()).unwrap();

            let address0 = Address::standard_address_v1(&pubkey0);
            let address1 = Address::standard_address_v1(&pubkey1);

            let vout0 = SiacoinOutput {
                value: 1u64.into(),
                address: address0,
            };
            let vout1 = SiacoinOutput {
                value: 1u64.into(),
                address: address1,
            };

            let file_contract_v2 = V2FileContract {
                capacity: 0,
                filesize: 1,
                file_merkle_root: Hash256::default(),
                proof_height: 1,
                expiration_height: 1,
                renter_output: vout0,
                host_output: vout1,
                missed_host_value: 1u64.into(),
                total_collateral: 1u64.into(),
                renter_public_key: pubkey0,
                host_public_key: pubkey1,
                revision_number: 1,
                renter_signature: sig0,
                host_signature: sig1,
            };

            let hash = Encoder::encode_and_hash(&file_contract_v2);
            let expected = Hash256::from_str("e851362bab643dc066b9d3c22c0fa0d67bc7b0cb520c689765e2292f4e7f435e").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_file_contract_element_v2_encode() {
            let pubkey0 = PublicKey::from_bytes(
                &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();
            let pubkey1 = PublicKey::from_bytes(
                &hex::decode("06C87838297B7BB16AB23946C99DFDF77FF834E35DB07D71E9B1D2B01A11E96D").unwrap(),
            )
            .unwrap();

            let sig0 = Signature::try_from(
                hex::decode("105641BF4AE119CB15617FC9658BEE5D448E2CC27C9BC3369F4BA5D0E1C3D01EBCB21B669A7B7A17CF8457189EAA657C41D4A2E6F9E0F25D0996D3A17170F309").unwrap()).unwrap();
            let sig1 = Signature::try_from(
                hex::decode("0734761D562958F6A82819474171F05A40163901513E5858BFF9E4BD9CAFB04DEF0D6D345BACE7D14E50C5C523433B411C7D7E1618BE010A63C55C34A2DEE70A").unwrap()).unwrap();

            let address0 = Address::standard_address_v1(&pubkey0);
            let address1 = Address::standard_address_v1(&pubkey1);

            let vout0 = SiacoinOutput {
                value: 1u64.into(),
                address: address0,
            };
            let vout1 = SiacoinOutput {
                value: 1u64.into(),
                address: address1,
            };

            let file_contract_v2 = V2FileContract {
                capacity: 0,
                filesize: 1,
                file_merkle_root: Hash256::default(),
                proof_height: 1,
                expiration_height: 1,
                renter_output: vout0,
                host_output: vout1,
                missed_host_value: 1u64.into(),
                total_collateral: 1u64.into(),
                renter_public_key: pubkey0,
                host_public_key: pubkey1,
                revision_number: 1,
                renter_signature: sig0,
                host_signature: sig1,
            };

            let state_element = StateElement {
                leaf_index: 1,
                merkle_proof: vec![
                    Hash256::from_str("0405060000000000000000000000000000000000000000000000000000000000").unwrap(),
                    Hash256::from_str("0708090000000000000000000000000000000000000000000000000000000000").unwrap(),
                ],
            };

            let file_contract_element_v2 = V2FileContractElement {
                id: Hash256::from_str("0707070000000000000000000000000000000000000000000000000000000000").unwrap().into(),
                state_element,
                v2_file_contract: file_contract_v2,
            };

            let hash = Encoder::encode_and_hash(&file_contract_element_v2);
            let expected = Hash256::from_str("3005594b14c1615aadaef2d8558713ebeabfa7d54f1dec671ba67ea8264816e6").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_file_contract_revision_v2_encode() {
            let pubkey0 = PublicKey::from_bytes(
                &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();
            let pubkey1 = PublicKey::from_bytes(
                &hex::decode("06C87838297B7BB16AB23946C99DFDF77FF834E35DB07D71E9B1D2B01A11E96D").unwrap(),
            )
            .unwrap();

            let sig0 = Signature::try_from(
                hex::decode("105641BF4AE119CB15617FC9658BEE5D448E2CC27C9BC3369F4BA5D0E1C3D01EBCB21B669A7B7A17CF8457189EAA657C41D4A2E6F9E0F25D0996D3A17170F309").unwrap()).unwrap();
            let sig1 = Signature::try_from(
                hex::decode("0734761D562958F6A82819474171F05A40163901513E5858BFF9E4BD9CAFB04DEF0D6D345BACE7D14E50C5C523433B411C7D7E1618BE010A63C55C34A2DEE70A").unwrap()).unwrap();

            let address0 = Address::standard_address_v1(&pubkey0);
            let address1 = Address::standard_address_v1(&pubkey1);

            let vout0 = SiacoinOutput {
                value: 1u64.into(),
                address: address0,
            };
            let vout1 = SiacoinOutput {
                value: 1u64.into(),
                address: address1,
            };

            let file_contract_v2 = V2FileContract {
                capacity: 0,
                filesize: 1,
                file_merkle_root: Hash256::default(),
                proof_height: 1,
                expiration_height: 1,
                renter_output: vout0,
                host_output: vout1,
                missed_host_value: 1u64.into(),
                total_collateral: 1u64.into(),
                renter_public_key: pubkey0,
                host_public_key: pubkey1,
                revision_number: 1,
                renter_signature: sig0,
                host_signature: sig1,
            };

            let state_element = StateElement {
                leaf_index: 1,
                merkle_proof: vec![
                    Hash256::from_str("0405060000000000000000000000000000000000000000000000000000000000").unwrap(),
                    Hash256::from_str("0708090000000000000000000000000000000000000000000000000000000000").unwrap(),
                ],
            };

            let file_contract_element_v2 = V2FileContractElement {
                id: Hash256::from_str("0102030000000000000000000000000000000000000000000000000000000000").unwrap().into(),
                state_element,
                v2_file_contract: file_contract_v2.clone(),
            };

            let file_contract_revision_v2 = FileContractRevisionV2 {
                parent: file_contract_element_v2,
                revision: file_contract_v2,
            };

            let hash = Encoder::encode_and_hash(&file_contract_revision_v2);
            let expected = Hash256::from_str("4f23582ec40570345f72adab8cd6249c0167669b78aec9ac7209befefc281f4f").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_v2_transaction_sig_hash() {
            let j = json!(
                {
                    "siacoinInputs": [
                        {
                            "parent": {
                                "id": "b49cba94064a92a75bf8c6f9d32ab18f38bfb14a2252e3e117d04da89d536f29",
                                "stateElement": {
                                    "leafIndex": 302,
                                    "merkleProof": [
                                        "6f41d366712e9dfa423160b5388f3faf673addf43566d7b3562106d15b833f46",
                                        "eb7df5e13eccd812a47f29a233bbf3212b7379ca6dd20ba9981524bfd5eadce6",
                                        "04104cbada51333f8f37a6eb71f1e8cb287da2d62469568a8a36dc8c76602c80",
                                        "16aac5c671d49d8cfc5493cb4c6f34889e30a0d283745c6473406bd60ab5e754",
                                        "1b9ccf2b6f555687b1384091faa9ed1c154f41aaff81dcf393295383ca99f518",
                                        "31337c9db5cdd181f5ff142bd490f779eedb1485e5dd905743280aeac3cd7ac9"
                                    ],
                                },
                                "siacoinOutput": {
                                    "value": "288594172736732570239334030000",
                                    "address": "2757c80b7ec2e493a138fed45b906f9f5735a992b68dcbd2069fbdf418c8b25158f3ac7a816b"
                                },
                                "maturityHeight": 0
                            },
                            "satisfiedPolicy": {
                                "policy": {
                                    "type": "uc",
                                    "policy": {
                                        "timelock": 0,
                                        "publicKeys": [
                                            "ed25519:7931b69fe8888e354d601a778e31bfa97fa89dc6f625cd01cc8aa28046e557e7"
                                        ],
                                        "signaturesRequired": 1
                                    }
                                },
                                "signatures": [
                                    "f43380794a6384e3d24d9908143c05dd37aaac8959efb65d986feb70fe289a5e26b84e0ac712af01a2f85f8727da18aae13a599a51fb066d098591e40cb26902"
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
                            "value": "287594172736732570239334030000",
                            "address": "2757c80b7ec2e493a138fed45b906f9f5735a992b68dcbd2069fbdf418c8b25158f3ac7a816b"
                        }
                    ],
                    "minerFee": "0"
                }
            );

            let tx = serde_json::from_value::<V2Transaction>(j).unwrap();
            let hash = tx.input_sig_hash();
            let expected = Hash256::from_str("ef2f59bb25300bed9accbdcd95e1a2bd9f146ab6b474002670dc908ad68aacac").unwrap();
            assert_eq!(hash, expected);
        }

        fn test_v2_transaction_signing() {
            let j = json!(
                {
                    "siacoinInputs": [
                        {
                            "parent": {
                                "id": "f59e395dc5cbe3217ee80eff60585ffc9802e7ca580d55297782d4a9b4e08589",
                                "stateElement": {
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
                                    ]
                                },
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
                                    "f0a29ba576eb0dbc3438877ac1d3a6da4f3c4cbafd9030709c8a83c2fffa64f4dd080d37444261f023af3bd7a10a9597c33616267d5371bf2c0ade5e25e61903"
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
            let keypair = Keypair::from_private_bytes(
                &hex::decode("0100000000000000000000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();
            let sig_hash = tx.input_sig_hash();

            // test that we can correctly regenerate the signature
            let sig: Signature = keypair.sign(&sig_hash.0);
            assert_eq!(tx.siacoin_inputs[0].satisfied_policy.signatures[0], sig);
        }

        fn test_siacoin_output_id_new() {
            let txid = Hash256::from_str("31be0badc64d40fbcb91b63835c07d75ab49addd1fc1d839b8415e1e5ff38cb5").unwrap();
            let output_index = 0u32;
            let output_id = SiacoinOutputId::new(txid, output_index);
            let expected = SiacoinOutputId(
                Hash256::from_str("47b2ceee0a9e246d5f997129a250ecb3d0917f5e844989d520e246145349d292").unwrap(),
            );
            assert_eq!(output_id, expected);
        }
    }
}
