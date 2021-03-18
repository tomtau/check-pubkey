mod rpc;
use std::str::FromStr;

use tendermint::net::Address;
use tendermint::chain::Id;
use tendermint::public_key::{PublicKey, PubKeyRequest};
use std::os::unix::net::UnixListener;
use crate::rpc::{Request, Response};
use std::io::Write;
use tmkms_light::utils::{print_pubkey, PubkeyDisplay};
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let address = Address::from_str(&args[1]).expect("listen on address");
    let chain_id = Id::from_str(&args[2]).expect("chain id");

    match address {
        Address::Unix { path } => {
            let listener = UnixListener::bind(path).expect("listen on");
            println!("listening on unix stream");
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        println!("got connection");
                        let request = Request::ReplyPing(Default::default()).encode().expect("request");
                        stream
                            .write_all(&request)
                            .expect("write ping failed");
                        let response = Response::read(&mut stream).expect("pong");
                        println!("got pong {:?}", response);
                        let request = Request::ShowPublicKey(PubKeyRequest {chain_id: chain_id.clone()}).encode().expect("request");
                        stream
                            .write_all(&request)
                            .expect("write show pubkey failed");
                        let response = Response::read(&mut stream).expect("pubkey");
                        println!("got pubkey {:?}", response);
                        match response {
                            Response::PublicKey(PublicKey::Ed25519(pk)) => {
                                print_pubkey(Some("crocnclconspub".to_owned()), Some(PubkeyDisplay::Bech32), pk);
                            }
                            _ => {
                                eprintln!("wrong response");
                            }
                        }
                                    
                    }
                    _ => {
                        eprintln!("error stream")
                    }
                }
            }
        }
        _ => {
            eprintln!("only unix path supported");
        }
    }
}
