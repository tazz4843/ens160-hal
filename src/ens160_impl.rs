use super::registers::*;
use super::{AirQualityIndex, Command, ECo2, OperationMode, Status};
use crate::InterruptConfig;
#[cfg(feature = "blocking")]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::{I2c, SevenBitAddress};

/// A driver for the `ENS160` sensor connected with I2C to the host.
pub struct Ens160<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Ens160<I2C> {
    /// Creates a new sensor driver.
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Releases the underlying I2C bus and destroys the driver.
    pub fn release(self) -> I2C {
        self.i2c
    }
}

#[cfg(feature = "blocking")]
impl<I2C, E> Ens160<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Resets the device.
    pub fn reset(&mut self) -> Result<(), E> {
        self.write_register([ENS160_OPMODE_REG, OperationMode::Reset as u8])
    }

    /// Switches the device to idle mode.
    ///
    /// Only in idle mode operations with `ENS160_COMMAND_REG` can be performed.
    pub fn idle(&mut self) -> Result<(), E> {
        self.write_register([ENS160_OPMODE_REG, OperationMode::Idle as u8])
    }

    /// Switches the device to deep sleep mode.
    ///
    /// This function can be used to conserve power when the device is not in use.
    pub fn deep_sleep(&mut self) -> Result<(), E> {
        self.write_register([ENS160_OPMODE_REG, OperationMode::Sleep as u8])
    }

    /// Switches the device to operational mode.
    ///
    /// Call this function when you want the device to start taking measurements.
    pub fn operational(&mut self) -> Result<(), E> {
        self.write_register([ENS160_OPMODE_REG, OperationMode::Standard as u8])
    }

    /// Clears the command register of the device.
    pub fn clear_command(&mut self) -> Result<(), E> {
        self.write_register([ENS160_COMMAND_REG, Command::Nop as u8])?;
        self.write_register([ENS160_COMMAND_REG, Command::Clear as u8])?;
        Ok(())
    }

    /// Returns the part ID of the sensor.
    pub fn part_id(&mut self) -> Result<u16, E> {
        self.read_register::<2>(ENS160_PART_ID_REG)
            .map(u16::from_le_bytes)
    }

    /// Returns the firmware version of the sensor.
    pub fn firmware_version(&mut self) -> Result<(u8, u8, u8), E> {
        self.write_register([ENS160_COMMAND_REG, Command::GetAppVersion as u8])?;
        let buffer = self.read_register::<3>(ENS160_GPR_READ_REG)?;
        Ok((buffer[0], buffer[1], buffer[2]))
    }

    /// Returns the current status of the sensor.
    pub fn status(&mut self) -> Result<Status, E> {
        self.read_register::<1>(ENS160_DATA_STATUS_REG)
            .map(|v| Status(v[0]))
    }

    /// Returns the current Air Quality Index (AQI) reading from the sensor.
    ///
    /// The AQI is calculated based on the current sensor readings.
    pub fn airquality_index(&mut self) -> Result<AirQualityIndex, E> {
        self.read_register::<1>(ENS160_DATA_AQI_REG)
            .map(|v| AirQualityIndex::from(v[0] & 0x07))
    }

    /// Returns the Total Volatile Organic Compounds (TVOC) measurement from the sensor.
    ///
    /// The TVOC level is expressed in parts per billion (ppb) in the range 0-65000.
    pub fn tvoc(&mut self) -> Result<u16, E> {
        self.read_register::<2>(ENS160_DATA_TVOC_REG)
            .map(u16::from_le_bytes)
    }

    /// Returns the Equivalent Carbon Dioxide (eCO2) measurement from the sensor.
    ///
    /// The eCO2 level is expressed in parts per million (ppm) in the range 400-65000.
    pub fn eco2(&mut self) -> Result<ECo2, E> {
        self.read_register::<2>(ENS160_DATA_ECO2_REG)
            .map(u16::from_le_bytes)
            .map(ECo2::from)
    }

    /// Returns the temperature (in °C) and relative humidity (in %) values used in the calculations.
    ///
    /// The units are scaled by 100. For example, a temperature value of 2550 represents 25.50 °C,
    /// and a humidity value of 5025 represents 50.25% RH.
    ///
    /// These values can be set using [`Ens160::set_temp_and_hum()`].
    pub fn temp_and_hum(&mut self) -> Result<(i16, u16), E> {
        let buffer = self.read_register::<4>(ENS160_DATA_T_REG)?;
        let temp = u16::from_le_bytes([buffer[0], buffer[1]]);
        let rh = u16::from_le_bytes([buffer[2], buffer[3]]);

        let temp = temp as i32 * 100 / 64 - 27315;
        let hum = rh as u32 * 100 / 512;

        Ok((temp as i16, hum as u16))
    }

    /// Sets the temperature value used in the device's calculations.
    ///
    /// Unit is scaled by 100. For example, a temperature value of 2550 should be used for 25.50 °C.
    pub fn set_temp(&mut self, ambient_temp: i16) -> Result<(), E> {
        let temp = ((ambient_temp as i32 + 27315) * 64 / 100) as u16;
        let temp = temp.to_le_bytes();
        let tbuffer = [ENS160_TEMP_IN_REG, temp[0], temp[1]];
        self.write_register(tbuffer)
    }

    /// Sets the relative humidity value used in the device's calculations.
    ///
    /// Unit is scaled by 100. For example, a humidity value of 5025 should be used for 50.25% RH.
    pub fn set_hum(&mut self, relative_humidity: u16) -> Result<(), E> {
        let rh = (relative_humidity as u32 * 512 / 100) as u16;
        let rh = rh.to_le_bytes();
        let hbuffer = [ENS160_RH_IN_REG, rh[0], rh[1]];
        self.write_register(hbuffer)
    }

    /// Sets interrupt configuration.
    pub fn set_interrupt_config(&mut self, config: InterruptConfig) -> Result<(), E> {
        self.write_register([ENS160_CONFIG_REG, config.finish().0])
    }

    fn read_register<const N: usize>(&mut self, register: u8) -> Result<[u8; N], E> {
        let mut write_buffer = [0u8; 1];
        write_buffer[0] = register;
        let mut buffer = [0u8; N];
        self.i2c
            .write_read(self.address, &write_buffer, &mut buffer)?;
        Ok(buffer)
    }

    fn write_register<const N: usize>(&mut self, buffer: [u8; N]) -> Result<(), E> {
        self.i2c.write(self.address, &buffer)
    }
}

#[cfg(feature = "async")]
impl<I2C, E> Ens160<I2C>
where
    I2C: I2c<SevenBitAddress, Error = E>,
{
    /// Resets the device.
    pub async fn reset(&mut self) -> Result<(), E> {
        self.write_register([ENS160_OPMODE_REG, OperationMode::Reset as u8])
            .await
    }

    /// Switches the device to idle mode.
    ///
    /// Only in idle mode operations with `ENS160_COMMAND_REG` can be performed.
    pub async fn idle(&mut self) -> Result<(), E> {
        self.write_register([ENS160_OPMODE_REG, OperationMode::Idle as u8])
            .await
    }

    /// Switches the device to deep sleep mode.
    ///
    /// This function can be used to conserve power when the device is not in use.
    pub async fn deep_sleep(&mut self) -> Result<(), E> {
        self.write_register([ENS160_OPMODE_REG, OperationMode::Sleep as u8])
            .await
    }

    /// Switches the device to operational mode.
    ///
    /// Call this function when you want the device to start taking measurements.
    pub async fn operational(&mut self) -> Result<(), E> {
        self.write_register([ENS160_OPMODE_REG, OperationMode::Standard as u8])
            .await
    }

    /// Clears the command register of the device.
    pub async fn clear_command(&mut self) -> Result<(), E> {
        self.write_register([ENS160_COMMAND_REG, Command::Nop as u8])
            .await?;
        self.write_register([ENS160_COMMAND_REG, Command::Clear as u8])
            .await?;
        Ok(())
    }

    /// Returns the part ID of the sensor.
    pub async fn part_id(&mut self) -> Result<u16, E> {
        self.read_register::<2>(ENS160_PART_ID_REG)
            .await
            .map(u16::from_le_bytes)
    }

    /// Returns the firmware version of the sensor.
    pub async fn firmware_version(&mut self) -> Result<(u8, u8, u8), E> {
        self.write_register([ENS160_COMMAND_REG, Command::GetAppVersion as u8])
            .await?;
        let buffer = self.read_register::<3>(ENS160_GPR_READ_REG).await?;
        Ok((buffer[0], buffer[1], buffer[2]))
    }

    /// Returns the current status of the sensor.
    pub async fn status(&mut self) -> Result<Status, E> {
        self.read_register::<1>(ENS160_DATA_STATUS_REG)
            .await
            .map(|v| Status(v[0]))
    }

    /// Returns the current Air Quality Index (AQI) reading from the sensor.
    ///
    /// The AQI is calculated based on the current sensor readings.
    pub async fn airquality_index(&mut self) -> Result<AirQualityIndex, E> {
        self.read_register::<1>(ENS160_DATA_AQI_REG)
            .await
            .map(|v| AirQualityIndex::from(v[0] & 0x07))
    }

    /// Returns the Total Volatile Organic Compounds (TVOC) measurement from the sensor.
    ///
    /// The TVOC level is expressed in parts per billion (ppb) in the range 0-65000.
    pub async fn tvoc(&mut self) -> Result<u16, E> {
        self.read_register::<2>(ENS160_DATA_TVOC_REG)
            .await
            .map(u16::from_le_bytes)
    }

    /// Returns the Equivalent Carbon Dioxide (eCO2) measurement from the sensor.
    ///
    /// The eCO2 level is expressed in parts per million (ppm) in the range 400-65000.
    pub async fn eco2(&mut self) -> Result<ECo2, E> {
        self.read_register::<2>(ENS160_DATA_ECO2_REG)
            .await
            .map(u16::from_le_bytes)
            .map(ECo2::from)
    }

    /// Returns the temperature (in °C) and relative humidity (in %) values used in the calculations.
    ///
    /// The units are scaled by 100. For example, a temperature value of 2550 represents 25.50 °C,
    /// and a humidity value of 5025 represents 50.25% RH.
    ///
    /// These values can be set using [`Ens160::set_temp_and_hum()`].
    pub async fn temp_and_hum(&mut self) -> Result<(i16, u16), E> {
        let buffer = self.read_register::<4>(ENS160_DATA_T_REG).await?;
        let temp = u16::from_le_bytes([buffer[0], buffer[1]]);
        let rh = u16::from_le_bytes([buffer[2], buffer[3]]);

        let temp = temp as i32 * 100 / 64 - 27315;
        let hum = rh as u32 * 100 / 512;

        Ok((temp as i16, hum as u16))
    }

    /// Sets the temperature value used in the device's calculations.
    ///
    /// Unit is scaled by 100. For example, a temperature value of 2550 should be used for 25.50 °C.
    pub async fn set_temp(&mut self, ambient_temp: i16) -> Result<(), E> {
        let temp = ((ambient_temp as i32 + 27315) * 64 / 100) as u16;
        let temp = temp.to_le_bytes();
        let tbuffer = [ENS160_TEMP_IN_REG, temp[0], temp[1]];
        self.write_register(tbuffer).await
    }

    /// Sets the relative humidity value used in the device's calculations.
    ///
    /// Unit is scaled by 100. For example, a humidity value of 5025 should be used for 50.25% RH.
    pub async fn set_hum(&mut self, relative_humidity: u16) -> Result<(), E> {
        let rh = (relative_humidity as u32 * 512 / 100) as u16;
        let rh = rh.to_le_bytes();
        let hbuffer = [ENS160_RH_IN_REG, rh[0], rh[1]];
        self.write_register(hbuffer).await
    }

    /// Sets interrupt configuration.
    pub async fn set_interrupt_config(&mut self, config: InterruptConfig) -> Result<(), E> {
        self.write_register([ENS160_CONFIG_REG, config.finish().0])
            .await
    }

    async fn read_register<const N: usize>(&mut self, register: u8) -> Result<[u8; N], E> {
        let mut write_buffer = [0u8; 1];
        write_buffer[0] = register;
        let mut buffer = [0u8; N];
        self.i2c
            .write_read(self.address, &write_buffer, &mut buffer)
            .await?;
        Ok(buffer)
    }

    async fn write_register<const N: usize>(&mut self, buffer: [u8; N]) -> Result<(), E> {
        self.i2c.write(self.address, &buffer).await
    }
}
