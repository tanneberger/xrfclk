use std::sync::Arc;
use tracing::{error, info, Level};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("loading static config!");
    let config = Arc::new(xrfclk::load_config_from_file());

    let lmk_freq = 12288;
    let lmx_freq = 10204;

    info!("configuring clocks");
    match xrfclk::set_ref_clks(config, lmk_freq, lmx_freq).await {
        Ok(_) => {
            info!("successfully configured all the clocks!");
        }
        Err(e) => {
            error!("error occurred: {e}");
        }
    }
}
