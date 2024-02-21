mod my_ota;

use anyhow::{self};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;
use log::info;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("!!!IMAGE TWO!!!");

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs))?;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: "brisa".into(),
        password: "kalicanelo".into(),
        auth_method: AuthMethod::None,
        ..Default::default()
    }))?;

    wifi.start()?;
    wifi.connect()?;

    let config = wifi.get_configuration().unwrap();
    println!("Waiting for station {:?}", config);
    while !wifi.is_connected().unwrap() {
        // Get and print connetion configuration
    }

    println!("Connected");
    my_ota::my_ota::ota_update_handler(env!("VERSION"), env!("VERSION"))?;
    Ok(())

}