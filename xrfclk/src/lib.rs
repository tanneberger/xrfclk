pub mod error;

use error::XRFClkError;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, warn};

pub struct LMKDevice {
    unix_spi_device_string: PathBuf,
    chip_name: Chip,
    number_of_bytes: u32,
    config: Arc<Config>,
}

pub struct LMXDevice {
    unix_spi_device_string: PathBuf,
    chip_name: Chip,
    config: Arc<Config>,
}

#[derive(Hash, Eq, PartialEq)]
pub enum Chip {
    LMX2594 = 0,
    LMK04832 = 1,
    LMK04208 = 2,
}

impl fmt::Display for Chip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LMX2594 => write!(f, "lmx2594"),
            Self::LMK04208 => write!(f, "lmk04208"),
            Self::LMK04832 => write!(f, "lmk04832"),
            // TODO: lmk04828
        }
    }
}

impl FromStr for Chip {
    type Err = error::XRFClkError;

    fn from_str(s: &str) -> Result<Chip, Self::Err> {
        match s {
            "lmx2594" => Ok(Chip::LMX2594),
            "lmk04208" => Ok(Chip::LMK04208),
            "lmk04832" => Ok(Chip::LMK04832),
            _ => Err(Self::Err::from(error::XRFClkErrorKind::InvalidChipString)),
        }
    }
}

impl<'de> Deserialize<'de> for Chip {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

fn cleanse_c_strings(s: &mut String) -> String {
    s.retain(|c| !c.is_whitespace());
    s.trim_matches(char::from(0)).to_string()
}

type Config = HashMap<Chip, HashMap<u64, HashMap<String, u32>>>;

pub fn load_config_from_file() -> Config {
    let included_string = include_str!("config.json");

    serde_json::from_str(included_string).expect("wrong json at compile time")
}

impl LMKDevice {
    pub fn from(
        chip_name: Chip,
        unix_spi_device_string: PathBuf,
        number_of_bytes: u32,
        config: Arc<Config>,
    ) -> Self {
        Self {
            unix_spi_device_string,
            chip_name,
            number_of_bytes,
            config,
        }
    }

    pub async fn write_registers(
        &self,
        register_values: &Vec<u32>,
    ) -> Result<(), error::XRFClkError> {
        debug!(
            "writing {} register values of chip: {} to: {}",
            register_values.len(),
            &self.chip_name,
            &self.unix_spi_device_string.display()
        );

        let mut file_handle = fs::File::open(&self.unix_spi_device_string)?;

        for value in register_values {
            let bytes = &value.to_be_bytes();

            if self.number_of_bytes == 3 {
                file_handle.write_all(&bytes[1..4])?;
            } else {
                file_handle.write_all(bytes)?;
            }
        }

        Ok(())
    }

    pub async fn set_clks(&self, frequency: u64) -> Result<(), error::XRFClkError> {
        debug!(
            "setting clocks of chip {} to frequency: {}",
            &self.chip_name, &frequency
        );

        let frequency_map = self.config.get(&self.chip_name).unwrap();

        if let Some(values) = frequency_map.get(&frequency) {
            let registers: Vec<u32> = values.clone().into_values().collect();
            self.write_registers(&registers).await
        } else {
            Err(error::XRFClkError::from(
                error::XRFClkErrorKind::InvalidFrequency,
            ))
        }
    }
}

impl LMXDevice {
    pub fn from(chip_name: Chip, unix_spi_device_string: PathBuf, config: Arc<Config>) -> Self {
        Self {
            unix_spi_device_string,
            chip_name,
            config,
        }
    }

    pub async fn write_registers(
        &self,
        register_values: &Vec<u32>,
    ) -> Result<(), error::XRFClkError> {
        debug!(
            "writing {} register values of chip: {} to: {}",
            register_values.len(),
            &self.chip_name,
            &self.unix_spi_device_string.display()
        );

        let mut file_handle = fs::File::open(&self.unix_spi_device_string)?;

        // Program RESET = 1 to reset registers
        let reset = 0x020000_u32.to_be_bytes();
        file_handle.write_all(&reset[1..])?;

        // Program RESET = 0 to remove reset
        let remove_reset = 0x0_u32.to_be_bytes();
        file_handle.write_all(&remove_reset[1..])?;

        for value in register_values {
            let bytes = &value.to_be_bytes();
            file_handle.write_all(&bytes[1..])?;
        }

        // Program register R0 one additional time with FCAL_EN = 1
        // to ensure that the VCO calibration runs from a stable state.

        let stable = 112_u32.to_be_bytes();
        file_handle.write_all(&stable[1..])?;

        Ok(())
    }

    pub async fn set_clks(&self, frequency: u64) -> Result<(), error::XRFClkError> {
        debug!(
            "setting clocks of chip {} to frequency: {}",
            &self.chip_name, &frequency
        );

        let frequency_map = self.config.get(&self.chip_name).unwrap();

        if let Some(values) = frequency_map.get(&frequency) {
            let registers: Vec<u32> = values.clone().into_values().collect();
            self.write_registers(&registers).await
        } else {
            Err(error::XRFClkError::from(
                error::XRFClkErrorKind::InvalidFrequency,
            ))
        }
    }
}

pub async fn spi_device_bind(
    device_string: &PathBuf,
    device_name: &str,
) -> Result<(), error::XRFClkError> {
    let bind_file = device_string.clone().join("driver_override");

    debug!(
        "binding spi device: device string: {:?} device name: {}",
        &bind_file, &device_name
    );

    let mut driver_override_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&bind_file)?;

    driver_override_file.write_all("spidev".as_bytes())?;

    let mut bind_file =
        fs::File::create(std::path::PathBuf::from("/sys/bus/spi/drivers/spidev/bind"))?;

    bind_file.write_all(device_name.as_bytes())?;

    Ok(())
}

pub async fn find_devices(
    config: Arc<Config>,
) -> Result<(Vec<LMKDevice>, Vec<LMXDevice>), error::XRFClkError> {
    debug!("finding devices on this system!");

    let mut lmx_devices = Vec::new();
    let mut lmk_devices = Vec::new();

    const LINUX_SPI_DEVICES: &str = "/sys/bus/spi/devices/";

    for file in fs::read_dir(LINUX_SPI_DEVICES)? {
        // file is of the form e.g. 'ti,lmx2594'
        debug!("processing spi device: {:?}", &file);

        let unwrapped_file = file?;
        let file_path = unwrapped_file.path().clone();
        let file_name = unwrapped_file.path().join("of_node/compatible");

        // processing the file name to figure out the hardware behind this driver
        let file_contents = match fs::read_to_string(&file_name) {
            Ok(value) => value,
            Err(e) => {
                warn!("cannot read spi device: {e}");
                continue;
            }
        };

        let mut chip_string = match file_contents.split_once(",") {
            Some((_, value)) => value.to_string(),
            None => {
                debug!("cannot split spi device string for {}", file_name.display());
                continue;
            }
        };

        chip_string = cleanse_c_strings(&mut chip_string);

        match Chip::from_str(&chip_string) {
            Ok(chip) => {
                // unbinding the file
                if file_path.join("driver").exists() {
                    debug!("bind file exists unbinding it!");
                    let mut unbind_file = fs::File::create(file_path.join("driver/unbind"))?;
                    unbind_file.write_all(chip.to_string().as_bytes())?;
                }

                debug!("creating bind file!");
                spi_device_bind(&file_path, &chip_string).await?;

                if chip == Chip::LMK04832 || chip == Chip::LMK04208 {
                    let mut bytes: [u8; 4] = [42u8; 4];
                    fs::File::open(file_path.clone().join("of_node/num_bytes"))?
                        .read_exact(&mut bytes)?;

                    lmk_devices.push(LMKDevice::from(
                        chip,
                        file_path,
                        u32::from_be_bytes(bytes),
                        config.clone(),
                    ))
                } else {
                    lmx_devices.push(LMXDevice::from(chip, file_path, config.clone()))
                }
            }
            Err(_) => {
                debug!(
                    "spi device not having valid chip string: |{}|",
                    &chip_string
                );
            }
        }
    }

    Ok((lmk_devices, lmx_devices))
}

pub async fn set_ref_clks(
    config: Arc<Config>,
    lmk_freq: u64,
    lmx_freq: u64,
) -> Result<(), error::XRFClkError> {
    let (lmk_devices, lmx_devices) = find_devices(config).await?;

    for lmk_device in lmk_devices {
        lmk_device.set_clks(lmk_freq).await?;
    }

    for lmx_device in lmx_devices {
        lmx_device.set_clks(lmx_freq).await?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::load_config_from_file;

    #[test]
    fn check_if_the_json_parses() {
        load_config_from_file();
    }
}
