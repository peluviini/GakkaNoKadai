
use anyhow::Result;
use esp_idf_sys::link_patches;
use esp_idf_hal::{
    delay::BLOCK, i2c::{I2cConfig, I2cDriver}, peripherals::Peripherals, prelude::*
};
use std::thread;
use std::time::Duration;

use core::convert::TryInto;

use embedded_svc::{
    http::{client::Client as HttpClient, Method},
    io::Write,
    utils::io,
    wifi::{AuthMethod, ClientConfiguration},
};

use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use log::{error, info};

use core::f32::consts::PI;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use std::io::Read;

const SSID: &'static str = "pelu's Nothing Phone";
const PASSWORD: &'static str = "kws8b8tj";
const URL_GAS: &'static str = "https://script.google.com/macros/s/AKfycby8D0VcTwNRlZ0MPskfAP5FgP_SFfzporLFQiRYREyPhuzRJfLr_QK-Cspy_Tf0hzIiUA/exec";

fn main() -> Result<()> {
    link_patches();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let mut wifi = BlockingWifi::wrap(
    EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
    sys_loop,
    )?;

    use esp_idf_svc::http::client::Configuration;
    connect_wifi(&mut wifi)?;
    let mut client = HttpClient::wrap(EspHttpConnection::new(
        &Configuration {
            use_global_ca_store: true,
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
            ..Default::default()
            
        }
    )?);

    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let config = I2cConfig::new()
        .baudrate(400.kHz().into())
        .sda_enable_pullup(true)
        .scl_enable_pullup(true);
    let mut i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;

    let amg_addr: u8 = 0x69;

    loop {
        let mut buf = [0u8; 128];

        i2c.write_read(amg_addr, &[0x80], &mut buf, BLOCK)?;

        let mut temps: [f32; 64] = [0.0; 64];
        for i in 0..64 {
            let lo = buf[2 * i] as u16;
            let hi = buf[2 * i + 1] as u16;
            let mut raw = ((hi << 8) | lo) & 0x0FFF;
            if (raw & 0x800) != 0 {
                raw = raw.wrapping_sub(0x1000);
                let raw_signed = (raw as i16) as f32;
                temps[i] = raw_signed * 0.25;
            } else {
                temps[i] = (raw as f32) * 0.25;
            }
        }

        let angle = angle_from_column_weighted(&temps).unwrap();

        println!("post an angle: {}", angle);
        post(&mut client, angle).unwrap();

        for row in 0..8 {
            for col in 0..8 {
                let idx = row * 8 + col;
                print!("{:6.2} ", temps[idx]);
            }
            println!();
        }
        println!("----------------------------");

        thread::sleep(Duration::from_millis(800));  
    }
}
fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    use embedded_svc::wifi::Configuration;
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: PASSWORD.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected");

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}
fn angle_from_column_weighted(temps: &[f32; 64]) -> Option<f32> {
    let mut col_sums = [0.0_f32; 8];
    for r in 0..8 {
        for c in 0..8 {
            col_sums[c] += temps[r * 8 + c];
        }
    }
    let total: f32 = col_sums.iter().sum();
    if total.abs() < 1e-6 {
        return None;
    }
    let mut weighted = 0.0_f32;
    for c in 0..8 {
        let center = (c as f32) + 0.5;
        weighted += center * col_sums[c];
    }
    let col_avg = weighted / total;

    let left = -PI / 3.0_f32;
    let right =  PI / 3.0_f32;
    let frac = col_avg / 8.0;
    let angle = left + frac * (right - left);
    Some(angle)
}

fn post(client: &mut HttpClient<EspHttpConnection>, angle: f32) -> anyhow::Result<()> {

    let ts = SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d: Duration| d.as_secs())
        .unwrap_or(0);
    let json = json!({
        "method": "post",
        "content": "sensor",
        "params": {
            "angle": angle
        }
    });
    let payload = &serde_json::to_vec(&json).unwrap();

    let content_length_header = format!("{}", payload.len());
    let headers = [
        ("content-type", "application/json"),
        ("content-length", &*content_length_header),
    ];

    let mut request = client.post(URL_GAS, &headers)?;
    request.write_all(payload)?;
    request.flush()?;
    info!("-> POST {}", URL_GAS);
    
    let mut response = request.submit()?;
    let status = response.status();
    info!("<- status: {}", status);

    let mut body_vec: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 512];
    loop {
        let n = response.read(&mut tmp)?;
        if n == 0 {
            break;
        }
        body_vec.extend_from_slice(&tmp[..n]);
    }

    match std::str::from_utf8(&body_vec) {
        Ok(body_string) => info!("Response body: {}", body_string),
        Err(e) => error!("Error decoding response body: {}", e),
    };

    Ok(())
}