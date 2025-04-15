mod config;

use std::path::Path;
use pynq::mmio::Mmio;


struct PLInterface {
    bram_config: config::MMIOConfig,
    dsp_register_config: config::RegistersConfig,
    cfg_register_config: config::RegistersConfig
}

impl PLInterface {
    fn from_files(bram_config_file: Path, dsp_register_config_file: Path, cfg_register_config_file: Path) -> PLInterface {
        
    }
}

