//! Envy configuration ui test #7 - prefix source (constant).

#![cfg(feature = "envy_configuration")]

use blockz::configuration::EasyConfiguration;
use blockz::prelude::*;

use serde::Deserialize;

const ENV_CONFIG_PREFIX: &str = "SOURCED_PREFIX_";

#[derive(Configuration, Deserialize, PartialEq)]
#[configuration(envy(prefix_source = "self::ENV_CONFIG_PREFIX.to_string()"))]
struct EnvConfig {
    server_port: u32,
}

#[tokio::main]
async fn main() {
    // set the required env variables
    std::env::set_var("SOURCED_PREFIX_SERVER_PORT", "1234");

    let conf1 = <EnvConfig as EasyConfiguration>::load().await.unwrap();
    let conf2 = <EnvConfig as Configuration>::load(()).await.unwrap();
    assert!(conf1 == conf2);
}