use std::net::IpAddr;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AddressCfg {
    pub domain: String,
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrimaryServerCfg {
    pub address: AddressCfg,
    pub resource_root: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServersCfg {
    pub primary: PrimaryServerCfg,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cfg {
    pub servers: ServersCfg,
}
