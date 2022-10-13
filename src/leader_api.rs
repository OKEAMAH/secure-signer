use crate::keys::{bls_key_gen, list_bls_keys};
use crate::attest::{epid_remote_attestation};

use anyhow::{Result, Context, bail};
use serde_derive::{Deserialize, Serialize};
use warp::{reply, Filter, http::Response, http::StatusCode};
use std::collections::HashMap;



#[derive(Deserialize, Serialize, Debug)]
pub struct KeyGenResponseInner {
    pub status: String,
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KeyGenResponse {
    pub data: [KeyGenResponseInner; 1],
}

/// Runs all the logic to generate and save a new BLS key. Returns a `KeyGenResponse` on success.
pub async fn bls_key_gen_service() -> Result<impl warp::Reply, warp::Rejection> {
    match bls_key_gen() {
        Ok(pk) => {
            let pk_hex = hex::encode(pk.compress());
            let data = KeyGenResponseInner { status: "imported".to_string(), message: pk_hex};
            let resp = KeyGenResponse { data: [data] };
            Ok(reply::with_status(reply::json(&resp), StatusCode::OK))
        }
        Err(e) => {
            let mut resp = HashMap::new();
            resp.insert("error", e.to_string());
            Ok(reply::with_status(reply::json(&resp), StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}

/// Generates a new BLS private key in Enclave. To remain compatible with web3signer POST /eth/v1/keystores, the JSON body is not parsed. The BLS public key is returned 
pub fn bls_key_gen_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("portal"))
        .and(warp::path("v1"))
        .and(warp::path("keystores"))
        .and_then(bls_key_gen_service)
}


#[derive(Debug)]
#[derive(Deserialize, Serialize)]
pub struct ListKeysResponseInner {
    pub pubkey: String,
}

#[derive(Debug)]
#[derive(Deserialize, Serialize)]
pub struct ListKeysResponse {
    pub data: Vec<ListKeysResponseInner>,
}

impl ListKeysResponse {
    pub fn new(keys: Vec<String>) -> ListKeysResponse {
        let inners = keys.iter().map(|pk| {
            ListKeysResponseInner {
                pubkey: format!("0x{}", pk),
            }
        }).collect();

        ListKeysResponse {
            data: inners
        }
    }
}

pub async fn list_bls_keys_service() -> Result<impl warp::Reply, warp::Rejection> {
    match list_bls_keys() {
        Ok(pks) => {
            let resp = ListKeysResponse::new(pks);
            Ok(reply::with_status(reply::json(&resp), warp::http::StatusCode::OK))
        }
        Err(e) => {
            let mut resp = HashMap::new();
            resp.insert("error", e.to_string());
            Ok(reply::with_status(reply::json(&resp), warp::http::StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}

/// Returns the hex-encoded BLS public keys that have their corresponding secret keys safeguarded in Enclave memory. 
pub fn list_bls_keys_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("portal"))
        .and(warp::path("v1"))
        .and(warp::path("keystores"))
        .and_then(list_bls_keys_service)
}