mod my_ota;

use anyhow::{self};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;
use log::{error, info};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    println!("Starting version {}", VERSION);
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs))?;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: "brisa".try_into().unwrap(),
        password: "kalicanelo".try_into().unwrap(),
        auth_method: AuthMethod::None,
        ..Default::default()
    }))?;
    wifi.start()?;
    wifi.connect()?;

    let config = wifi.get_configuration().unwrap();
    println!("Waiting for WiFi connection {:?}", config);
    while !wifi.is_connected().unwrap() || !wifi.is_up().unwrap() {
        // Get and print connetion configuration
    }
    println!("WiFi connection established");
    loop {
        match my_ota::my_ota::do_update_if_available(VERSION, "device_id") {
            Ok(did_update) => {
                info!("Update was successful");
                if did_update.unwrap() {
                    esp_idf_hal::delay::FreeRtos::delay_ms(1000);
                    info!("Restarting device after firmware update");
                    unsafe {
                        esp_idf_sys::esp_restart();
                    }
                } else {
                    info!("Did not update firmware");
                }
            }
            Err(e) => {
                error!("Error fetching or installing update {}", e);
            }
        }
    }

    Ok(())
}