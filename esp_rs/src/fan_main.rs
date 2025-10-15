
use anyhow::Result;
use esp_idf_svc::{
  hal::{gpio::PinDriver, peripherals::Peripherals},
};
use esp_idf_hal::{
    delay::{self},
    i2c::{I2cConfig, I2cDriver},
    ledc::{
        config::TimerConfig, LedcDriver, LedcTimerDriver
    },
    units::Hertz
};
use esp_idf_hal::prelude::*;
use std::time::Duration;
use bme280::i2c::BME280;

use serde::{Deserialize, Serialize};

use core::convert::TryInto;

use embedded_svc::{
    http::{client::Client as HttpClient, Method},
    utils::io,
    io::Write,
    wifi::{AuthMethod, ClientConfiguration},
};

use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use log::{error, info};

use core::f32::consts::{PI, E};

use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
enum Power {
    On,
    Off,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct ParamsSent {
    power: Power,
    targetTemp: u32,
    angle: f32,
}
impl Default for Power {
    fn default() -> Self {
        Power::Off
    }
}

const URL_GAS_RECIVE: &'static str = "https://script.google.com/macros/s/AKfycbxAZuuMf1YV7tr9ESOnFpejUin488bMCvWWBuS4zBLB89zmvKsiwr19o9ySxy1oQMhXTw/exec?param=receive";
const URL_GAS: &'static str = "https://script.google.com/macros/s/AKfycbxAZuuMf1YV7tr9ESOnFpejUin488bMCvWWBuS4zBLB89zmvKsiwr19o9ySxy1oQMhXTw/exec";

const SSID: &'static str = "pelu's Nothing Phone";
const PASSWORD: &'static str = "kws8b8tj";

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

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
            buffer_size_tx: Some(4096),
            ..Default::default()
        }
    )?);

    let timer_fan = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &TimerConfig::default()
            .frequency(Hertz(25 * 1000).into())
            .resolution(esp_idf_hal::ledc::Resolution::Bits8),
    )?;
    let timer_servo = LedcTimerDriver::new(
        peripherals.ledc.timer1,
        &TimerConfig::default()
            .frequency(Hertz(50).into())
            .resolution(esp_idf_hal::ledc::Resolution::Bits16),
    )?;

    let mut fan_vin = PinDriver::output(peripherals.pins.gpio14)?;
    let mut servo_vin = PinDriver::output(peripherals.pins.gpio12)?;
    let mut pwm_fan = LedcDriver::new(
        peripherals.ledc.channel0,
        &timer_fan,
        peripherals.pins.gpio33,
    )?;
    let mut pwm_servo = LedcDriver::new(
        peripherals.ledc.channel1,
        &timer_servo,
        peripherals.pins.gpio13,
    )?;

    let max_duty_fan = pwm_fan.get_max_duty();
    let max_duty_servo = pwm_servo.get_max_duty();

    let period_us = 20_000f32;

    //let duty_min = (max_duty_servo as f32 * (500.0 / period_us)) as u32;
    let duty_mid = (max_duty_servo as f32 * (1500.0 / period_us)) as u32;
    //let duty_max = (max_duty_servo as f32 * (2500.0 / period_us)) as u32;
    
    
    let sda = peripherals.pins.gpio22;
    let scl = peripherals.pins.gpio21;
    let config = I2cConfig::new()
        .baudrate(100.kHz().into())
        .sda_enable_pullup(true)
        .scl_enable_pullup(true);
    let i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;

    let mut bmp280 = BME280::new_primary(i2c);
    let mut delay = delay::Ets;
    bmp280.init(&mut delay).unwrap();

    

    let mut datas: ParamsSent = Default::default();

    loop {

        let bmp = bmp280.measure(&mut delay).unwrap();
         println!(
            "Relative Humidity = {:3.2} %,   Temperature = {:3.2} °C,   Pressure = {:4.2} hPa",
            bmp.humidity,
            bmp.temperature,
            bmp.pressure / 100_f32
        );

        use std::result::Result::Ok;
        match get(&mut client) {
            Ok(d) => {
                println!("ok");
                datas = d.clone();
                println!("{:?} : {:?} : {:?}", datas.power, datas.targetTemp, datas.angle);
            }
            Err(e) => {
                println!("err: {:?}", e);
            }
        }

        if datas.power == Power::On {
            fan_vin.set_high()?;
            servo_vin.set_high()?;

            let duty_servo = (max_duty_servo as f32 * ((1500. + (datas.angle / (PI / 4.) * 1000.)) / period_us)) as u32;
            
            let wind_speed = {
                let e = (bmp.humidity / 100.) * 6.105 * E.powf((17.27 * bmp.temperature) / (237.7 + bmp.temperature));
                let v = (datas.targetTemp as f32 - bmp.temperature - 0.33 * e + 4.) / 0.7;
                v.max(0.0)
            };
            println!("wind: {}", wind_speed);
            let duty_fan = {
                let q_max = 146.9 / 3600.;
                let v_max = q_max / 0.01;

                let max = max_duty_fan as f32;
                let ratio = wind_speed / v_max;
                let mut duty = max * ratio;
                if duty > max {
                    duty = max;
                } else if duty < max * 0.2 {
                    duty = max * 0.2;
                }

                duty
            };

            println!("pwm servo: {}", duty_servo);
            pwm_servo.set_duty(duty_servo)?;
            println!("pwm fan: {}", duty_fan);
            pwm_fan.set_duty(duty_fan as u32)?;
            
            post(&mut client, bmp.temperature, bmp.humidity, wind_speed).unwrap();

        } else {

            pwm_servo.set_duty(duty_mid)?;
            fan_vin.set_low()?;
            servo_vin.set_low()?;
        }

        std::thread::sleep(Duration::from_millis(50));
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
fn get(client: &mut HttpClient<EspHttpConnection>) -> anyhow::Result<ParamsSent> {
    let headers = [("accept", "application/json")];

    let request = client.request(Method::Get, URL_GAS_RECIVE, &headers)?;
    info!("-> GET {URL_GAS_RECIVE}");
    let mut response = request.submit()?;

    let status = response.status();
    info!("<- {status}");
    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    info!("Read {bytes_read} bytes");
    use std::result::Result::Ok;
    match std::str::from_utf8(&buf[0..bytes_read]) {
        Ok(body_string) => info!(
            "Response body (truncated to {} bytes): {:?}",
            buf.len(),
            body_string
        ),
        Err(e) => error!("Error decoding response body: {e}"),
    };

    if let Ok(body_str) = std::str::from_utf8(&buf[..bytes_read]) {
        info!("Body ({} bytes): {}", bytes_read, body_str);
    } else {
        error!("Body is not valid UTF-8");
    }

    let value: serde_json::Value = serde_json::from_slice(&buf[..bytes_read])?;

    //println!("value: {:?}", value);

    let params = serde_json::from_value(value)?;
    Ok(params)
}


fn post(
    client: &mut HttpClient<EspHttpConnection>,
    temperature: f32,
    humidity: f32,
    wind_speed: f32,
) -> anyhow::Result<()> {

    let _ = SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d: Duration| d.as_secs())
        .unwrap_or(0);
    let json = json!({
        "method": "post",
        "content": "sensor",
        "params": {
            "temperature": temperature,
            "humidity": humidity,
            "wind_speed": wind_speed,
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