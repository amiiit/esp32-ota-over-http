pub mod my_ota {
    use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
    use esp_idf_svc::ota::EspOta;
    use log::info;
    use embedded_svc::http::Method;

    const BUF_MAX: usize = 2048;
    const TARGET_VERSION_MAX_LENGTH: usize = 64;

    fn fetch_target_firmware_version(device_id: &str) -> anyhow::Result<String, anyhow::Error> {
        let mut client = EspHttpConnection::new(&Configuration {
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
            buffer_size: Some(BUF_MAX),
            ..Default::default()
        })?;

        let device_specific_url = format!("https://storage.googleapis.com/devices/{}/target", device_id);
        info!("Requesting target version via device-specific URL: {}", device_specific_url);

        let resp = client.initiate_request(Method::Get, &device_specific_url, &[])?;
        client.initiate_response()?;

        // let content_length = client.header("Content-Length")?;


        if client.status() != 200 {
            info!("Error fetching target for device from {}", device_specific_url);
            return Err(anyhow::Error::msg("Failed to fetch target version"));
        }

        let mut target_version_body = [0; TARGET_VERSION_MAX_LENGTH];
        client.read(&mut target_version_body)?;

        match std::str::from_utf8(&target_version_body) {
            Ok(body) => Ok(String::from(body)),
            Err(e) => Err(anyhow::Error::new(e))
        }
    }

    pub fn do_update_if_available(current_version: &str, device_id: &str) -> Result<Option<bool>, anyhow::Error> {
        let target_version = fetch_target_firmware_version(device_id)?;
        if current_version == target_version {
            Ok(Some(false))
        } else {
            ota_update_handler(target_version)?;
            return Ok(Some(true));
        }
    }

    pub fn ota_update_handler(target_version: String) -> anyhow::Result<()> {
        info!("Start processing OTA update");

        const BUF_MAX: usize = 2 * 1024;
        let mut body = [0; BUF_MAX];

        let firmware_url = format!("https://storage.googleapis.com/devices/images/{}/image.bin", target_version);
        info!("Using firmware from: {}", firmware_url);

        let mut ota = EspOta::new()?;
        let mut ota_update = ota.initiate_update()?;

        let mut client = EspHttpConnection::new(&Configuration {
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
            buffer_size: Some(BUF_MAX),
            ..Default::default()
        })?;

        let _resp = client.initiate_request(Method::Get, &firmware_url, &[]);
        client.initiate_response()?;

        let content_length = client.header("Content-Length")
            .map(|len| len.parse().unwrap_or_default())
            .unwrap_or_default();

        info!("Content-length: {:?}", content_length);
        info!("Initiating OTA update");

        let mut bytes_read_total = 0;

        loop {
            esp_idf_hal::delay::FreeRtos::delay_ms(10);
            let n_bytes_read = client.read(&mut body)?;

            bytes_read_total += n_bytes_read;

            if !body[..n_bytes_read].is_empty() {
                ota_update.write(&body[..n_bytes_read])?;
            } else {
                info!("ERROR: Firmware image with zero length");
                break;
            }

            if body.len() > n_bytes_read {
                break;
            }
        }

        if bytes_read_total == content_length {
            ota_update.complete()?;
            info!("Successfully completed firmware update");
        } else {
            ota_update.abort()?;
            info!("ERROR: Firmware update failed");
            return Err(anyhow::Error::msg("Firmware update failed"));
        }


        Ok(())
    }
}
