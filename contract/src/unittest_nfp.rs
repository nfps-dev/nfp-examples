#[cfg(test)]
mod tests {
    use std::any::Any;

    use cosmwasm_std::testing::*;
    use cosmwasm_std::{
        from_binary, to_binary, Addr, Api, Binary, OwnedDeps,
        Response, StdError, StdResult,
    };
    use crate::contract::{execute, instantiate, query,};
    use crate::msg::{
        ContractStatus, ExecuteAnswer, ExecuteMsg, InstantiateConfig,
        InstantiateMsg, Mint, QueryAnswer, QueryMsg,
        KeyValuePair, ViewerInfo, ViewerInfoAddrOpt,
    };
    use crate::nfp::{RawData, KEY_CLEARED_PACKAGES};
    use crate::token::{Extension, Metadata,};

    // Helper functions

    fn init_helper_default() -> (
        StdResult<Response>,
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("instantiator", &[]);
        let init_msg = InstantiateMsg {
            name: "sec721".to_string(),
            symbol: "S721".to_string(),
            admin: Some("admin".to_string()),
            entropy: "We're going to need a bigger boat".to_string(),
            royalty_info: None,
            config: None,
            post_init_callback: None,
        };

        (instantiate(deps.as_mut(), env, info, init_msg), deps)
    }

    fn init_helper_with_config(
        public_token_supply: bool,
        public_owner: bool,
        enable_sealed_metadata: bool,
        unwrapped_metadata_is_private: bool,
        minter_may_update_metadata: bool,
        owner_may_update_metadata: bool,
        enable_burn: bool,
        minter_may_put_token_storage: bool,
        minter_may_put_global_storage: bool,
    ) -> (
        StdResult<Response>,
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies();

        let env = mock_env();
        let init_config: InstantiateConfig = from_binary(&Binary::from(
            format!(
                "{{\"public_token_supply\":{},
                \"public_owner\":{},
                \"enable_sealed_metadata\":{},
                \"unwrapped_metadata_is_private\":{},
                \"minter_may_update_metadata\":{},
                \"owner_may_update_metadata\":{},
                \"enable_burn\":{},
                \"minter_may_put_token_storage\":{},
                \"minter_may_put_global_storage\":{}}}",
                public_token_supply,
                public_owner,
                enable_sealed_metadata,
                unwrapped_metadata_is_private,
                minter_may_update_metadata,
                owner_may_update_metadata,
                enable_burn,
                minter_may_put_token_storage,
                minter_may_put_global_storage,
            )
            .as_bytes(),
        ))
        .unwrap();
        let info = mock_info("instantiator", &[]);
        let init_msg = InstantiateMsg {
            name: "sec721".to_string(),
            symbol: "S721".to_string(),
            admin: Some("admin".to_string()),
            entropy: "We're going to need a bigger boat".to_string(),
            royalty_info: None,
            config: Some(init_config),
            post_init_callback: None,
        };

        (instantiate(deps.as_mut(), env, info, init_msg), deps)
    }

    fn extract_error_msg<T: Any>(error: StdResult<T>) -> String {
        match error {
            Ok(_response) => panic!("Expected error, but had Ok response"),
            Err(err) => match err {
                StdError::GenericErr { msg, .. } => msg,
                _ => panic!("Unexpected error result {:?}", err),
            },
        }
    }

    fn extract_log(resp: StdResult<Response>) -> String {
        match resp {
            Ok(response) => response.attributes[0].value.clone(),
            Err(_err) => "These are not the logs you are looking for".to_string(),
        }
    }

    #[test]
    fn test_storage_owner() {
        let (init_result, mut deps) =
            init_helper_with_config(false, false, false, false, true, false, false, false, false);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let execute_msg = ExecuteMsg::StorageOwnerPut {
            data: [
                KeyValuePair {
                    key: "test".to_string(),
                    value: Some("data".to_string()),
                }
            ].to_vec(),
            owner: None,
            token_id: None,
            padding: None,
        };
        let exec_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            execute_msg,
        );
        let exec_answer: ExecuteAnswer = from_binary(&exec_result.unwrap().data.unwrap()).unwrap();
        println!("{:?}", exec_answer);

        let viewer = ViewerInfo {
            address: "alice".to_string(),
            viewing_key: "key".to_string(),
        };

        let query_msg = QueryMsg::StorageOwnerGet { 
            keys: vec!["test".to_string()], 
            viewer,
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        let error = extract_error_msg(query_result);
        println!("{}", error);
        assert!(error.contains("unauthorized"));

        let execute_msg = ExecuteMsg::SetViewingKey {
            key: "key".to_string(),
            padding: None,
        };
        let _handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            execute_msg,
        );
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::StorageOwnerGet { data } => {
                assert_eq!(data[0],
                    KeyValuePair {
                        key: "test".to_string(),
                        value: Some("data".to_string()),
                    }
                );
            }
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn test_package_version() {
        let (init_result, mut deps) =
        init_helper_with_config(
            false, 
            false, 
            false, 
            false, 
            true, 
            false, 
            false, 
            true, 
            true
        );
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let execute_msg = ExecuteMsg::SetContractStatus {
            level: ContractStatus::Normal,
            padding: None,
        };
        let _handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );

        let alice = Addr::unchecked("alice".to_string());
        let _alice_raw = deps.api.addr_canonicalize(alice.as_str()).unwrap();
        let admin = Addr::unchecked("admin".to_string());
        let _admin_raw = deps.api.addr_canonicalize(admin.as_str()).unwrap();
        let pub1 = Metadata {
            token_uri: None,
            extension: Some(Extension {
                name: Some("NFT1".to_string()),
                description: Some("pub1".to_string()),
                image: Some("uri1".to_string()),
                ..Extension::default()
            }),
        };
        let priv2 = Metadata {
            token_uri: None,
            extension: Some(Extension {
                name: Some("NFT2".to_string()),
                description: Some("priv2".to_string()),
                image: Some("uri2".to_string()),
                ..Extension::default()
            }),
        };
        let mints = vec![
            Mint {
                token_id: None,
                owner: Some(alice.to_string()),
                public_metadata: Some(pub1.clone()),
                private_metadata: None,
                royalty_info: None,
                serial_number: None,
                transferable: None,
                memo: None,
            },
            Mint {
                token_id: Some("NFT2".to_string()),
                owner: None,
                public_metadata: None,
                private_metadata: Some(priv2.clone()),
                royalty_info: None,
                serial_number: None,
                transferable: None,
                memo: None,
            },
            Mint {
                token_id: Some("NFT3".to_string()),
                owner: Some(alice.to_string()),
                public_metadata: None,
                private_metadata: None,
                royalty_info: None,
                transferable: None,
                serial_number: None,
                memo: None,
            },
            Mint {
                token_id: None,
                owner: Some(admin.to_string()),
                public_metadata: None,
                private_metadata: None,
                royalty_info: None,
                transferable: None,
                serial_number: None,
                memo: Some("has id 3".to_string()),
            },
        ];
        let execute_msg = ExecuteMsg::BatchMintNft {
            mints: mints.clone(),
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        let minted_vec = vec![
            "0".to_string(),
            "NFT2".to_string(),
            "NFT3".to_string(),
            "3".to_string(),
        ];
        let exec_answer: ExecuteAnswer =
            from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
        match exec_answer {
            ExecuteAnswer::BatchMintNft { token_ids } => {
                assert_eq!(token_ids, minted_vec);
            }
            _ => panic!("unexpected"),
        }

        // upload public package
        let execute_msg = ExecuteMsg::UploadPackageVersion { 
            package_id: "script1.js".to_string(), 
            data: RawData {
                bytes: to_binary("publicdata").unwrap(),
                content_type: Some("image-svg/xml".to_string()),
                content_encoding: Some("gzip".to_string()),
                metadata: None,
            }, 
            tags: Some(vec![
                "v1".to_string(), 
                "latest".to_string()
            ]), 
            metadata: Some("some metadata".to_string()), 
            access: "public".to_string(), 
            padding: None 
        };
        let exec_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        let exec_answer: ExecuteAnswer =
            from_binary(&exec_result.unwrap().data.unwrap()).unwrap();
        println!("{:?}", exec_answer);
        match exec_answer {
            ExecuteAnswer::UploadPackageVersion { index } => {
                assert_eq!(index, 0_u32);
            }
            _ => panic!("unexpected"),
        }

        // alice viewing key
        let execute_msg = ExecuteMsg::SetViewingKey {
            key: "key".to_string(),
            padding: None,
        };
        let _exec_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            execute_msg,
        );

        // bob viewing key
        let execute_msg = ExecuteMsg::SetViewingKey {
            key: "key123".to_string(),
            padding: None,
        };
        let _exec_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            execute_msg,
        );

        // query package version
        let query_msg = QueryMsg::PackageVersion { 
            package_id: "script1.js".to_string(), 
            tag: Some("latest".to_string()), 
            index: None, 
            token_id: "0".to_string(), 
            viewer: None,
            //viewer: Some(ViewerInfoAddrOpt {
            //    address: None,
            //    viewing_key: "key".to_string()
            //}),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        println!("{:?}", query_answer);
        match query_answer {
            QueryAnswer::PackageVersion { package } => {
                assert_eq!(package.unwrap().access, "public");
            }
            _ => panic!("unexpected"),
        }

        // query package version -- public should ignore token_id and viewer
        let query_msg = QueryMsg::PackageVersion { 
            package_id: "script1.js".to_string(), 
            tag: Some("latest".to_string()), 
            index: None, 
            token_id: "no token".to_string(), 
            viewer: Some(ViewerInfoAddrOpt {
                address: None,
                viewing_key: "some key".to_string()
            }),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        println!("{:?}", query_answer);
        match query_answer {
            QueryAnswer::PackageVersion { package } => {
                assert_eq!(package.unwrap().access, "public");
            }
            _ => panic!("unexpected"),
        }

        // try to upload new version of package with different type --should error
        let execute_msg = ExecuteMsg::UploadPackageVersion { 
            package_id: "script1.js".to_string(), 
            data: RawData {
                bytes: to_binary("publicdata").unwrap(),
                content_type: Some("image-svg/xml".to_string()),
                content_encoding: Some("gzip".to_string()),
                metadata: None,
            }, 
            tags: Some(vec![
                "v1".to_string(), 
                "latest".to_string()
            ]), 
            metadata: Some("some metadata".to_string()), 
            access: "owners".to_string(), 
            padding: None 
        };
        let exec_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        let error = extract_error_msg(exec_result);
        assert!(error.contains("access of new package version MUST be public"));

        // upload a 2nd version that is public
        let execute_msg = ExecuteMsg::UploadPackageVersion { 
            package_id: "script1.js".to_string(), 
            data: RawData {
                bytes: to_binary("publicdata2").unwrap(),
                content_type: Some("image-svg/xml".to_string()),
                content_encoding: Some("gzip".to_string()),
                metadata: None,
            }, 
            tags: Some(vec![
                "v2".to_string(), 
                "latest".to_string()
            ]), 
            metadata: Some("some metadata".to_string()), 
            access: "public".to_string(), 
            padding: None 
        };
        let exec_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        let exec_answer: ExecuteAnswer =
            from_binary(&exec_result.unwrap().data.unwrap()).unwrap();
        println!("{:?}", exec_answer);
        match exec_answer {
            ExecuteAnswer::UploadPackageVersion { index } => {
                assert_eq!(index, 1_u32);
            }
            _ => panic!("unexpected"),
        }

        // query package version -- should give second package with "latest"
        let query_msg = QueryMsg::PackageVersion { 
            package_id: "script1.js".to_string(), 
            tag: Some("latest".to_string()), 
            index: None, 
            token_id: "no token".to_string(), 
            viewer: Some(ViewerInfoAddrOpt {
                address: None,
                viewing_key: "some key".to_string()
            }),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        println!("{:?}", query_answer);
        match query_answer {
            QueryAnswer::PackageVersion { package } => {
                assert!(package.unwrap().tags.unwrap().contains(&"v2".to_string()));
            }
            _ => panic!("unexpected"),
        }

        // upload a new owners package
        let execute_msg = ExecuteMsg::UploadPackageVersion { 
            package_id: "script2.js".to_string(), 
            data: RawData {
                bytes: to_binary("ownersdata").unwrap(),
                content_type: Some("image-svg/xml".to_string()),
                content_encoding: Some("gzip".to_string()),
                metadata: None,
            }, 
            tags: Some(vec![
                "v1".to_string(), 
                "latest".to_string()
            ]), 
            metadata: Some("some metadata".to_string()), 
            access: "owners".to_string(), 
            padding: None 
        };
        let exec_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        let exec_answer: ExecuteAnswer =
            from_binary(&exec_result.unwrap().data.unwrap()).unwrap();
        println!("{:?}", exec_answer);
        match exec_answer {
            ExecuteAnswer::UploadPackageVersion { index } => {
                assert_eq!(index, 0_u32);
            }
            _ => panic!("unexpected"),
        }

        // query with non-owner, bob
        let query_msg = QueryMsg::PackageVersion { 
            package_id: "script2.js".to_string(), 
            tag: Some("latest".to_string()), 
            index: None, 
            token_id: "0".to_string(), 
            viewer: Some(ViewerInfoAddrOpt {
                address: Some("bob".to_string()),
                viewing_key: "key123".to_string()
            }),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        assert!(query_result.is_err());

        // query with owner, alice, wrong token
        let query_msg = QueryMsg::PackageVersion { 
            package_id: "script2.js".to_string(), 
            tag: Some("latest".to_string()), 
            index: None, 
            token_id: "no token".to_string(), 
            viewer: Some(ViewerInfoAddrOpt {
                address: Some("alice".to_string()),
                viewing_key: "key".to_string()
            }),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        assert!(query_result.is_err());

        // query with alice, token owner
        let query_msg = QueryMsg::PackageVersion { 
            package_id: "script2.js".to_string(), 
            tag: Some("latest".to_string()), 
            index: None, 
            token_id: "0".to_string(), 
            viewer: Some(ViewerInfoAddrOpt {
                address: Some("alice".to_string()),
                viewing_key: "key".to_string()
            }),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        println!("{:?}", query_answer);
        match query_answer {
            QueryAnswer::PackageVersion { package } => {
                assert_eq!(package.unwrap().access, "owners".to_string());
            }
            _ => panic!("unexpected"),
        }

        // upload a new cleared package
        let execute_msg = ExecuteMsg::UploadPackageVersion { 
            package_id: "script3.js".to_string(), 
            data: RawData {
                bytes: to_binary("cleareddata").unwrap(),
                content_type: Some("image-svg/xml".to_string()),
                content_encoding: Some("gzip".to_string()),
                metadata: None,
            }, 
            tags: Some(vec![
                "v1".to_string(), 
                "latest".to_string()
            ]), 
            metadata: Some("some metadata".to_string()), 
            access: "cleared".to_string(), 
            padding: None 
        };
        let exec_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        let exec_answer: ExecuteAnswer =
            from_binary(&exec_result.unwrap().data.unwrap()).unwrap();
        println!("{:?}", exec_answer);
        match exec_answer {
            ExecuteAnswer::UploadPackageVersion { index } => {
                assert_eq!(index, 0_u32);
            }
            _ => panic!("unexpected"),
        }

        // query with alice, token owner, not cleared
        let query_msg = QueryMsg::PackageVersion { 
            package_id: "script3.js".to_string(), 
            tag: Some("latest".to_string()), 
            index: None, 
            token_id: "0".to_string(), 
            viewer: Some(ViewerInfoAddrOpt {
                address: Some("alice".to_string()),
                viewing_key: "key".to_string()
            }),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        assert!(query_result.is_err());

        // set cleared for token "0"
        let execute_msg = ExecuteMsg::StorageTokenPut { 
            data: vec![
                KeyValuePair {
                    key: KEY_CLEARED_PACKAGES.to_string(),
                    value: Some("[\"script3.js\"]".to_string()),
                }
            ], 
            token_id: "0".to_string(), 
            padding: None 
        };
        let _exec_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );

        // query with alice, token owner, cleared
        let query_msg = QueryMsg::PackageVersion { 
            package_id: "script3.js".to_string(), 
            tag: Some("latest".to_string()), 
            index: None, 
            token_id: "0".to_string(), 
            viewer: Some(ViewerInfoAddrOpt {
                address: Some("alice".to_string()),
                viewing_key: "key".to_string()
            }),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        println!("{:?}", query_answer);
        match query_answer {
            QueryAnswer::PackageVersion { package } => {
                assert_eq!(package.unwrap().access, "cleared".to_string());
            }
            _ => panic!("unexpected"),
        }

        // query package_info without auth
        let query_msg = QueryMsg::PackageInfo { package_id: "script1.js".to_string(), page: Some(0), page_size: Some(100), viewer: None };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::PackageInfo { version_count, .. } => {
                assert_eq!(version_count, 2_u32);
            }
            _ => panic!("unexpected"),
        }
        
        let query_msg = QueryMsg::PackageInfo { package_id: "script2.js".to_string(), page: Some(0), page_size: Some(100), viewer: None };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        assert!(query_result.is_err());

        let query_msg = QueryMsg::PackageInfo { package_id: "script3.js".to_string(), page: Some(0), page_size: Some(100), viewer: None };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        assert!(query_result.is_err());

        // query package_info with auth
        let query_msg = QueryMsg::PackageInfo { 
            package_id: "script1.js".to_string(), 
            page: Some(0), 
            page_size: Some(100), 
            viewer: Some(ViewerInfo { address: "alice".to_string(), viewing_key: "key".to_string() }) 
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::PackageInfo { version_count, .. } => {
                assert_eq!(version_count, 2_u32);
            }
            _ => panic!("unexpected"),
        }
        
        let query_msg = QueryMsg::PackageInfo { 
            package_id: "script2.js".to_string(), 
            page: Some(0), 
            page_size: Some(100), 
            viewer: Some(ViewerInfo { address: "alice".to_string(), viewing_key: "key".to_string() })  
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::PackageInfo { version_count, .. } => {
                assert_eq!(version_count, 1_u32);
            }
            _ => panic!("unexpected"),
        }

        let query_msg = QueryMsg::PackageInfo { 
            package_id: "script3.js".to_string(), 
            page: Some(0), 
            page_size: Some(100), 
            viewer: Some(ViewerInfo { address: "alice".to_string(), viewing_key: "key".to_string() })  
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::PackageInfo { version_count, .. } => {
                assert_eq!(version_count, 1_u32);
            }
            _ => panic!("unexpected"),
        }

        // query package_info with auth, not owner
        let query_msg = QueryMsg::PackageInfo { 
            package_id: "script1.js".to_string(), 
            page: Some(0), 
            page_size: Some(100), 
            viewer: Some(ViewerInfo { address: "bob".to_string(), viewing_key: "key123".to_string() }) 
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::PackageInfo { version_count, .. } => {
                assert_eq!(version_count, 2_u32);
            }
            _ => panic!("unexpected"),
        }
        
        let query_msg = QueryMsg::PackageInfo { 
            package_id: "script2.js".to_string(), 
            page: Some(0), 
            page_size: Some(100), 
            viewer: Some(ViewerInfo { address: "bob".to_string(), viewing_key: "key123".to_string() })  
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::PackageInfo { version_count, .. } => {
                assert_eq!(version_count, 0_u32);
            }
            _ => panic!("unexpected"),
        }

        let query_msg = QueryMsg::PackageInfo { 
            package_id: "script3.js".to_string(), 
            page: Some(0), 
            page_size: Some(100), 
            viewer: Some(ViewerInfo { address: "bob".to_string(), viewing_key: "key123".to_string() })  
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg.clone());
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::PackageInfo { version_count, .. } => {
                assert_eq!(version_count, 0_u32);
            }
            _ => panic!("unexpected"),
        }

    }
}