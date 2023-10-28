// ENS160 Register address
// This 2-byte register contains the part number in little endian of the ENS160.
pub const ENS160_PART_ID_REG: u8 = 0x00;
// This 1-byte register sets the Operating Mode of the ENS160.
pub const ENS160_OPMODE_REG: u8 = 0x10;
// This 1-byte register configures the action of the INTn pin.
pub const ENS160_CONFIG_REG: u8 = 0x11;
// This 1-byte register allows some additional commands to be executed on the ENS160.
#[allow(dead_code)]
pub const ENS160_COMMAND_REG: u8 = 0x12;
// This 2-byte register allows the host system to write ambient temperature data to ENS160 for compensation.
pub const ENS160_TEMP_IN_REG: u8 = 0x13;
// This 2-byte register allows the host system to write relative humidity data to ENS160 for compensation.
#[allow(dead_code)]
pub const ENS160_RH_IN_REG: u8 = 0x15;
// This 1-byte register indicates the current STATUS of the ENS160.
pub const ENS160_DATA_STATUS_REG: u8 = 0x20;
// This 1-byte register reports the calculated Air Quality Index according to the UBA.
pub const ENS160_DATA_AQI_REG: u8 = 0x21;
// This 2-byte register reports the calculated TVOC concentration in ppb.
pub const ENS160_DATA_TVOC_REG: u8 = 0x22;
// This 2-byte register reports the calculated equivalent CO2-concentration in ppm, based on the detected VOCs and hydrogen.
pub const ENS160_DATA_ECO2_REG: u8 = 0x24;
// This 2-byte register reports the temperature used in its calculations (taken from TEMP_IN, if supplied).
pub const ENS160_DATA_T_REG: u8 = 0x30;
// This 2-byte register reports the relative humidity used in its calculations (taken from RH_IN if supplied).
#[allow(dead_code)]
pub const ENS160_DATA_RH_REG: u8 = 0x32;
// This 1-byte register reports the calculated checksum of the previous DATA_ read transaction (of n-bytes).
#[allow(dead_code)]
pub const ENS160_DATA_MISR_REG: u8 = 0x38;
// This 8-byte register is used by several functions for the Host System to pass data to the ENS160.
#[allow(dead_code)]
pub const ENS160_GPR_WRITE_REG: u8 = 0x40;
// This 8-byte register is used by several functions for the ENS160 to pass data to the Host System.
#[allow(dead_code)]
pub const ENS160_GPR_READ_REG: u8 = 0x48;
