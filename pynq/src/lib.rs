pub mod dma;
pub mod leds;
pub mod mmio;
pub mod overlay;
pub mod xlnk;

pub mod bitstream;

use std::fs::File;
use std::io::{Read, Write};

use crate::mmio::Mmio;

#[derive(Copy, Clone, Debug)]
pub struct Clock {
    pub div0: u32,
    pub div1: u32,
}

pub fn load_bitstream_from_file(filename: &str, clocks: &[Clock]) -> Result<usize, String> {
    let mut buf = Vec::new();
    load_bitstream_data(filename, &mut buf);
    configure_clocks(clocks);
    set_partial_bitstream(false);
    write_bitstream_data(&buf);
    Ok(buf.len())
}

pub fn load_bitstream_from_mem(program: &Vec<u8>, clocks: &[Clock]) -> Result<usize, String> {
    configure_clocks(clocks);
    set_partial_bitstream(false);
    write_bitstream_data(program);
    Ok(program.len())
}

fn configure_clocks(clocks: &[Clock]) {
    assert!(clocks.len() > 0, "Need to enable at least 1 clock!");
    assert!(
        clocks.len() <= 4,
        "Only four clocks FCLK(0-3) can be configured!"
    );
    let disabled = Clock { div0: 0, div1: 0 };
    let base_addr = 0xf8000000;
    fn offset(ii: usize) -> usize {
        (0x170 / 4) + (ii * (0x10 / 4))
    }
    let mut mem = Mmio::map(base_addr, 0x170 + 0x10 * 4);
    for (ii, clk) in clocks.iter().enumerate() {
        mem[offset(ii)] = calc_divs(clk, mem[offset(ii)]);
    }
    for ii in clocks.len()..4 {
        mem[offset(ii)] = calc_divs(&disabled, mem[offset(ii)]);
    }
}

fn calc_divs(clk: &Clock, old: u32) -> u32 {
    (old & !((0x3f << 20) | (0x3f << 8))) | (((clk.div1 & 0x3f) << 20) | ((clk.div0 & 0x3f) << 8))
}

fn set_partial_bitstream(enabled: bool) {
    let partial_bitstream = "/sys/devices/soc0/amba/f8007000.devcfg/is_partial_bitstream";
    let mut file = File::create(partial_bitstream).expect("Failed to open partial bitstream file!");
    file.write(if enabled { b"1" } else { b"0" }).unwrap();
}

fn load_bitstream_data(filename: &str, buf: &mut Vec<u8>) {
    let mut file = File::open(filename).expect("Failed to open bitstream file!");
    file.read_to_end(buf)
        .expect("Failed to read bitstream file!");
}

fn write_bitstream_data(buf: &[u8]) {
    let mut file = File::create("/dev/xdevcfg").unwrap();
    file.write_all(buf)
        .expect("Failed to write bitstream to FPGA");
}
