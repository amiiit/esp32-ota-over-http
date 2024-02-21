pub mod my_ota {
    use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
    use esp_idf_svc::ota::EspOta;
    use log::info;
    use embedded_svc::http::Method;
    use esp_idf_sys::EspError;

    fn fetch_target_firmware_url(device_id: &str) -> anyhow::Result<String, EspError> {
        const BUF_MAX: usize = 2 * 1024;

        let mut client = EspHttpConnection::new(&Configuration {
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
            buffer_size: Some(BUF_MAX),
            ..Default::default()
        })
            .expect("creation of EspHttpConnection should have worked");

        let device_specific_url = format!("https://storage.googleapis.com/devices/{}/target", device_id);
        info!("Will request target version via device specific URL: {}", device_specific_url);
        let _resp = client.initiate_request(Method::Get, &device_specific_url, &[]);
        const TARGET_VERSION_MAX_LENGTH: usize = 64;
        let mut target_version_body: [u8; TARGET_VERSION_MAX_LENGTH] = [0; TARGET_VERSION_MAX_LENGTH];
        client.initiate_response()?;
        let mut content_length: usize = 0;

        if let Some(len) = client.header("Content-Length") {
            content_length = len.parse().unwrap();
        } else {
            info!("reading content length for target version http request failed");
        }
        if client.status() != 200 {
            info!("Error fetching target for device from {}", device_specific_url);
            return anyhow::Error();
        }

        let target_version_bytesread = match client.read(&mut target_version_body)  {
            Ok(n) => n,
            Err(err) => {
                info!("Error reading body for target URL {:?}", err);
                Err(anyhow::Error());
            }
        }

        let result = match std::string::String::from_utf8(target_version_body.to_vec()) {
            Ok(s) => s,
            Err(err) => {
                info!("Error parding response into string {:?}", err);
                Err(anyhow::Error())
            }
        };

        Ok(result)
    }
    pub fn ota_update_handler(current_version: &str, device_id: &str) -> anyhow::Result<()> {


        let mut firmware_update_ok = false;

        info!("Start processing OTA update");

        let mut content_length: usize = 0;
        const BUF_MAX: usize = 2 * 1024;
        let mut body: [u8; BUF_MAX] = [0; BUF_MAX];

        let firmware_url = "https://storage.googleapis.com/devices/image.bin";
        info!("Will use firmware from: {}", firmware_url);

        let mut ota = EspOta::new().expect("EspOta::new should have been successful");

        let mut ota_update = ota
            .initiate_update()
            .expect("initiate ota update should have worked");

        info!("EspHttpConnection created");
        let _resp = client.initiate_request(embedded_svc::http::Method::Get, firmware_url, &[]);

        info!("after client.initiate_request()");

        client.initiate_response()?;

        if let Some(len) = client.header("Content-Length") {
            content_length = len.parse().unwrap();
        } else {
            info!("reading content length for firmware update http request failed");
        }

        info!("Content-length: {:?}", content_length);
        info!(">>>>>>>>>>>>>>>> initiating OTA update");

        let mut bytes_read_total = 0;

        loop {
            esp_idf_hal::delay::FreeRtos::delay_ms(10);
            let n_bytes_read = match client.read(&mut body) {
                Ok(n) => n,
                Err(err) => {
                    info!("ERROR reading firmware batch {:?}", err);
                    break;
                }
            };
            bytes_read_total += n_bytes_read;

            if !body.is_empty() {
                match ota_update.write(&body) {
                    Ok(_) => {}
                    Err(err) => {
                        info!("ERROR failed to write update with: {:?}", err);
                        break;
                    }
                }
            } else {
                info!("ERROR firmware image with zero length");
                break;
            }

            if body.len() > n_bytes_read {
                break;
            }
        }

        if bytes_read_total == content_length {
            firmware_update_ok = true;
        }

        let _confirmation_msg = if firmware_update_ok {
            ota_update.complete().unwrap();
            info!("completed firmware update");
            info!("Successfully completed firmware update")
        } else {
            ota_update.abort().unwrap();
            info!("ERROR firmware update failed");
            info!("Firmare update failed")
        };

        esp_idf_hal::delay::FreeRtos::delay_ms(1000);
        info!("restarting device after firmware update");
        unsafe {
            esp_idf_sys::esp_restart();
        }
    }
}

