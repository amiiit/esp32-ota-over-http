use anyhow::{self, Error};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("WiFi Starting");
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

    while !wifi.is_connected().unwrap() {
        // Get and print connetion configuration
        let config = wifi.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }

    println!("Connected");
    Ok(())

}