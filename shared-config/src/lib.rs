use std::net::IpAddr;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerCfg {
    pub domain: String,
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServersCfg {
    pub primary: ServerCfg,
    pub api: ServerCfg,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cfg {
    pub servers: ServersCfg,
}
