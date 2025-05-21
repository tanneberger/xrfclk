use pynq::Clock;
use tracing::{error, info, Level};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("loading and flashing bitstream file!");

    let filename = "./rust_bitstream.bit";

    let clocks: [Clock; 2] = [Clock { div0: 1, div1: 1 }; 2];

    match pynq::load_bitstream_from_file(filename, &clocks) {
        Ok(value) => {
            info!("load bitstream successful {value}")
        }
        Err(err_code) => {
            error!("received error from load bitstream call {err_code}");
        }
    }
}
