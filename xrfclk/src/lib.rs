pub mod error;

use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
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
    LMK04828 = 3,
}

impl fmt::Display for Chip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LMX2594 => write!(f, "lmx2594"),
            Self::LMK04208 => write!(f, "lmk04208"),
            Self::LMK04832 => write!(f, "lmk04832"),
            Self::LMK04828 => write!(f, "lmk04828"),
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
            "lmk04828" => Ok(Chip::LMK04828),
            _ => Err(Self::Err::from(error::XRFClkErrorKind::InvalidChipString)),
        }
    }
}

impl Chip {
    pub fn is_lmk(&self) -> bool {
        *self == Self::LMK04828 || *self == Self::LMK04832 || *self == Self::LMK04208
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

type RawConfig = HashMap<Chip, HashMap<u64, HashMap<String, String>>>;
type Config = HashMap<Chip, HashMap<u64, HashMap<String, u32>>>;

pub fn load_config_from_file() -> Config {
    let included_string = include_str!("config.json");

    let raw_config: RawConfig =
        serde_json::from_str(included_string).expect("wrong json at compile time");
    let mut config: Config = Config::new();

    for (chip, chip_values) in raw_config {
        let chip_entry: &mut HashMap<u64, HashMap<String, u32>> = config.entry(chip).or_default();
        for (freq, register_values) in chip_values {
            let freq_entry = chip_entry.entry(freq).or_default();
            for (reg, mut value) in register_values {
                value.remove(0);
                value.remove(0);
                freq_entry.insert(reg, u32::from_str_radix(&value, 16).unwrap());
            }
        }
    }

    config
}

pub fn generate_device_path(device_name: String) -> PathBuf {
    PathBuf::from(format!("/dev/{}", device_name.replace("spi", "spidev")))
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
        register_values: &HashMap<String, u32>,
    ) -> Result<(), error::XRFClkError> {
        debug!(
            "writing {} register values of chip {} to {}",
            register_values.len(),
            &self.chip_name,
            &self.unix_spi_device_string.display()
        );

        let mut file_handle = fs::OpenOptions::new()
            .write(true)
            .create(false)
            .open(&self.unix_spi_device_string)?;

        for value in register_values.values() {
            // makes sure to save the number in big endian
            let bytes: [u8; 4] = value.to_be_bytes();

            if self.number_of_bytes == 3 {
                file_handle.write_all(&bytes[1..4])?;
            } else {
                file_handle.write_all(&bytes)?;
            }
            file_handle.flush()?;
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
            self.write_registers(&values).await
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
        register_values: &HashMap<String, u32>,
    ) -> Result<(), error::XRFClkError> {
        debug!(
            "writing {} register values of chip {} to {}",
            register_values.len(),
            &self.chip_name,
            &self.unix_spi_device_string.display()
        );

        let mut file_handle = fs::OpenOptions::new()
            .write(true)
            .create(false)
            .open(&self.unix_spi_device_string)?;

        // Program RESET = 1 to reset registers
        let reset = 0x020000_u32.to_be_bytes();
        file_handle.write_all(&reset[1..])?;
        file_handle.flush()?;

        // Program RESET = 0 to remove reset
        let remove_reset = 0x0_u32.to_be_bytes();
        file_handle.write_all(&remove_reset[1..])?;
        file_handle.flush()?;

        for value in register_values.values() {
            let bytes = &value.to_be_bytes();
            file_handle.write_all(&bytes[1..])?;
            file_handle.flush()?;
        }

        // Program register R0 one additional time with FCAL_EN = 1
        // to ensure that the VCO calibration runs from a stable state.
        let stable = register_values
            .get("R112")
            .expect("Register 112 not specified for this device")
            .to_be_bytes();

        file_handle.write_all(&stable[1..])?;
        file_handle.flush()?;

        Ok(())
    }

    pub async fn set_clks(&self, frequency: u64) -> Result<(), error::XRFClkError> {
        debug!(
            "setting clocks of chip {} to frequency {}",
            &self.chip_name, &frequency
        );

        let frequency_map = self.config.get(&self.chip_name).unwrap();

        if let Some(values) = frequency_map.get(&frequency) {
            self.write_registers(&values).await
        } else {
            Err(error::XRFClkError::from(
                error::XRFClkErrorKind::InvalidFrequency,
            ))
        }
    }
}

pub async fn spi_device_bind(
    device_string: &Path,
    chip: &String,
) -> Result<(), error::XRFClkError> {
    let bind_file = device_string.to_path_buf().join("driver_override");

    debug!(
        "binding spi device: device string: {} device name: {}",
        &bind_file.display(),
        &chip
    );

    let mut driver_override_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&bind_file)?;

    driver_override_file.write_all("spidev".as_bytes())?;
    driver_override_file.flush()?;

    let mut bind_file = fs::OpenOptions::new()
        .write(true)
        .read(false)
        .open(std::path::PathBuf::from("/sys/bus/spi/drivers/spidev/bind"))?;

    bind_file.write_all(chip.as_bytes())?;
    bind_file.flush()?;

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
        let spi_name = unwrapped_file
            .file_name()
            .into_string()
            .map_err(|_| error::XRFClkError::from(error::XRFClkErrorKind::InvalidChipString))?;

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

                    let mut unbind_file = fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(file_path.join("driver/unbind"))?;

                    unbind_file.write_all(spi_name.as_bytes())?;
                    unbind_file.flush()?;
                }

                debug!("creating bind file! using spi dev: {}", &spi_name);
                spi_device_bind(&file_path, &spi_name).await?;

                let device_path = generate_device_path(spi_name);

                if chip.is_lmk() {
                    let mut bytes: [u8; 4] = [42u8; 4];
                    fs::File::open(file_path.clone().join("of_node/num_bytes"))?
                        .read_exact(&mut bytes)?;

                    lmk_devices.push(LMKDevice::from(
                        chip,
                        device_path,
                        u32::from_be_bytes(bytes),
                        config.clone(),
                    ))
                } else {
                    lmx_devices.push(LMXDevice::from(chip, device_path, config.clone()))
                }
            }
            Err(_) => {
                debug!("spi device not having valid chip string: {}", &chip_string);
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
