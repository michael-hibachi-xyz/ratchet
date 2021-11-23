// Copyright 2015-2021 Swim Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bytes::BytesMut;
use ratchet_deflate::{Deflate, DeflateExtProvider};
use ratchet_rs::UpgradedClient;
use ratchet_rs::{Error, Message, PayloadType, ProtocolRegistry, WebSocketConfig};
use tokio::net::TcpStream;

const AGENT: &str = "Ratchet";

async fn subscribe(url: &str) -> Result<UpgradedClient<TcpStream, Deflate>, Error> {
    let stream = TcpStream::connect("127.0.0.1:9001").await.unwrap();
    stream.set_nodelay(true).unwrap();

    ratchet_rs::subscribe_with(
        WebSocketConfig::default(),
        stream,
        url,
        &DeflateExtProvider::default(),
        ProtocolRegistry::default(),
    )
    .await
}

async fn get_case_count() -> Result<u32, Error> {
    let stream = TcpStream::connect("127.0.0.1:9001").await.unwrap();
    stream.set_nodelay(true).unwrap();

    let mut websocket = subscribe("ws://localhost:9001/getCaseCount")
        .await
        .unwrap()
        .websocket;
    let mut buf = BytesMut::new();

    match websocket.read(&mut buf).await? {
        Message::Text => {
            let count = String::from_utf8(buf.to_vec()).unwrap();
            Ok(count.parse::<u32>().unwrap())
        }
        _ => panic!(),
    }
}

async fn update_reports() -> Result<(), Error> {
    let mut _websocket = subscribe(&format!(
        "ws://localhost:9001/updateReports?agent={}",
        AGENT
    ))
    .await
    .unwrap();
    Ok(())
}

async fn run_test(case: u32) -> Result<(), Error> {
    let mut websocket = subscribe(&format!(
        "ws://localhost:9001/runCase?case={}&agent={}",
        case, AGENT
    ))
    .await
    .unwrap()
    .websocket;

    let mut buf = BytesMut::new();

    loop {
        match websocket.read(&mut buf).await? {
            Message::Text => {
                let _s = String::from_utf8(buf.to_vec())?;
                websocket.write(&mut buf, PayloadType::Text).await?;
                buf.clear();
            }
            Message::Binary => {
                websocket.write(&mut buf, PayloadType::Binary).await?;
                buf.clear();
            }
            Message::Ping | Message::Pong => {}
            Message::Close(_) => break Ok(()),
        }
    }
}

#[tokio::main]
async fn main() {
    let total = get_case_count().await.unwrap();

    for case in 1..=total {
        if let Err(e) = run_test(case).await {
            println!("{}", e);
        }
    }

    update_reports().await.unwrap();
}
