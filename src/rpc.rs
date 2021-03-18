//! Remote Procedure Calls
//! Copyright (c) 2018-2021 Iqlusion Inc. (licensed under the Apache License, Version 2.0)
//! Modifications Copyright (c) 2021, Foris Limited (licensed under the Apache License, Version 2.0)


use prost::Message as _;
use std::convert::TryFrom;
use std::io::Read;
use tendermint::public_key::{PubKeyRequest, PublicKey};
use tendermint_p2p::secret_connection::DATA_MAX_SIZE;
use tendermint_proto::{
    privval::{
        message::Sum, Message as PrivMessage, PingRequest, PingResponse
    },
};

/// Requests to the KMS
#[derive(Debug)]
pub enum Request {
    ShowPublicKey(PubKeyRequest),

    // PingRequest is a PrivValidatorSocket message to keep the connection alive.
    ReplyPing(PingRequest),
}

impl Request {
   /// Encode response to bytes
   pub fn encode(self) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();

    let msg = match self {
        Request::ReplyPing(pr) => Sum::PingRequest(pr),
        Request::ShowPublicKey(pkr) => Sum::PubKeyRequest(pkr.into()),
    };

    PrivMessage { sum: Some(msg) }
        .encode_length_delimited(&mut buf)
        .map_err(|e| {
            format!("failed to encode response: {}", e)
        })?;
    Ok(buf)
}

}

/// Responses from the KMS
#[derive(Debug)]
pub enum Response {
    /// Signature response
    Ping(PingResponse),
    PublicKey(PublicKey),
}

impl Response {

    /// Read a request from the given readable
    pub fn read(conn: &mut impl Read) -> Result<Self, String> {
        let msg = read_msg(conn)?;

        // Parse Protobuf-encoded request message
        let msg = PrivMessage::decode_length_delimited(msg.as_ref())
            .map_err(|e| format!("malformed message packet: {}", e))?
            .sum;

        match msg {
            Some(Sum::PingResponse(req)) => {
                
                Ok(Response::Ping(req))
            }
            Some(Sum::PubKeyResponse(req)) => {
                
                Ok(Response::PublicKey(PublicKey::try_from(req.pub_key.expect("pubkey")).expect("pubkey")))
            }
            _ => {
                Err("unexpected response".to_owned())
            }
        }
    }
}

/// Read a message from a Secret Connection
// TODO(tarcieri): extract this into Secret Connection
fn read_msg(conn: &mut impl Read) -> Result<Vec<u8>, String> {
    let mut buf = vec![0; DATA_MAX_SIZE];
    let buf_read = conn
        .read(&mut buf)
        .map_err(|e| format!("read msg failed: {}", e))?;
    buf.truncate(buf_read);
    Ok(buf)
}
