use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub enum MemoryAccess {
    #[serde(rename = "r")]
    READ,
    #[serde(rename = "w")]
    WRITE,
}

#[derive(Deserialize)]
pub enum RAMStyle {
    #[serde(rename = "block")]
    BLOCK,
}

#[derive(Deserialize)]
pub enum Sign {
    Unsigned,
    Signed,
}

#[derive(Deserialize)]
pub struct MMIOConfig {
    a_width: usize,
    a_depth: usize,
    b_width: usize,
    prefix: String,
    access: MemoryAccess,
    address: *const u8,
    ram_style: RAMStyle,
}

#[derive(Deserialize)]
pub struct RegisterConfig {
    access: MemoryAccess,
    addr_width: usize,
    data_width: usize,
    sign: Sign,
    init: i32,
    base_addr: usize,
}

pub type BRAMSConfig = HashMap<String, MMIOConfig>;
pub type RegistersConfig = HashMap<String, RegisterConfig>;
