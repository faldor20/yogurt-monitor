use std::error::Error;
use std::{io, time};

use anyhow::Error;
use ds18b20::{Ds18b20, Resolution};

use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::timer;
use one_wire_bus::{OneWire, OneWireResult, OneWireError};
use std::fmt::Debug;
use std::io::Write;
use std::time::Duration;
use futures::prelude::*;
use smol::{Async, Timer};
fn main() {
    println!("Hello, world!");
}
#[derive(Debug )]
enum mainError<E> {
    OneWireError(OneWireError<E>),
    NoSensorError

} 

impl<E> From<OneWireError<E>> for mainError<E> where E:Debug{
    fn from(err: OneWireError<E>) -> mainError<E> {
        mainError::OneWireError(err)
    }
}

fn get_temperature_probes<E, P>(
    delay: &mut (impl DelayUs<u16>),
    one_wire_bus: &mut OneWire<P>,
) -> Result<Ds18b20,mainError<E>>
where
    P: OutputPin<Error = E> + InputPin<Error = E>,
    E: Debug,
{
    let mut search_state = None;

    if let Some((device_address, state)) =
        one_wire_bus.device_search(search_state.as_ref(), false, delay)?
    {
        search_state = Some(state);
        // You will generally create the sensor once, and save it for later
        let sensor = Ds18b20::new(device_address)?;
        Ok(sensor)
    } else {
        Err(mainError::NoSensorError)
    }
}

async fn get_temperature<P, E>(
    delay: &mut (impl DelayUs<u16>),
    one_wire_bus: &mut OneWire<P>,
    sensor: Ds18b20,
) ->    OneWireResult<(), E>
where
    P: OutputPin<Error = E> + InputPin<Error = E>,
    E: Debug,
{
    // initiate a temperature measurement for all connected devices
    ds18b20::start_simultaneous_temp_measurement(one_wire_bus, delay)?;

    // wait until the measurement is done. This depends on the resolution you specified
    // If you don't know the resolution, you can obtain it from reading the sensor data,
    // or just wait the longest time, which is the 12-bit resolution (750ms)
    let delaymillis=ds18b20::Resolution::Bits9.max_measurement_time_millis();
    Timer::after(Duration::from_millis(delaymillis as u64)).await;

    // contains the read temperature, as well as config info such as the resolution used
    let sensor_data = sensor.read_data(one_wire_bus, delay)?;
    Ok(())
}

fn setup_config<P, E>(
    device: Ds18b20,
    delay: &mut (impl  DelayUs<u16>),
    one_wire_bus: &mut OneWire<P>,
    resolution: Resolution,
) -> OneWireResult<(), E>
where
    P: OutputPin<Error = E> + InputPin<Error = E>,
    E: Debug,
{
    // read the initial config values (read from EEPROM by the device when it was first powered)
    let initial_data = device.read_data(one_wire_bus, delay)?;
    println!("Initial data: {:?}", initial_data);

    // set new alarm values and resolutions
    device.set_config(18, 24, resolution, one_wire_bus, delay)?;

    // confirm the new config is now in the scratchpad memory
    let new_data = device.read_data(one_wire_bus, delay)?;
    println!("New data: {:?}", new_data);

    // save the config to EEPROM to save it permanently
    device.save_to_eeprom(one_wire_bus, delay)?;

    // read the values from EEPROM back to the scratchpad to verify it was saved correctly
    device.recall_from_eeprom(one_wire_bus, delay)?;
    let eeprom_data = device.read_data(one_wire_bus, delay)?;
    println!("EEPROM data: {:?}", eeprom_data);

    Ok(())
}
