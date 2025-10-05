
use anyhow::{Ok, Result};
use esp_idf_svc::{
  hal::{gpio::PinDriver, peripherals::Peripherals},
  sys::link_patches,
};
use esp_idf_hal::{
    units::Hertz,
    ledc::{
        config::TimerConfig,
        LedcChannel, LedcTimer, LedcDriver, LedcTimerDriver, LEDC
    }
};
use esp_idf_hal::prelude::*;
use std::{thread::sleep, time::Duration};


const URL_GAS_RECIVE: &'static str = "https://script.google.com/macros/s/AKfycbzVrJa80ZfQUyTxWgsJMkbTNdFpBfamwyeFIaKuuSQm/exec?param=recieve";

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();

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

    let duty_min = (max_duty_servo as f32 * (500.0 / period_us)) as u32;
    let duty_mid = (max_duty_servo as f32 * (1500.0 / period_us)) as u32;
    let duty_max = (max_duty_servo as f32 * (2500.0 / period_us)) as u32;
    loop {

        fan_vin.set_high()?;
        println!("high");

        pwm_fan.set_duty(max_duty_fan * 30 / 100)?;


        std::thread::sleep(Duration::from_secs(60));

        fan_vin.set_low()?;
        println!("low");

        std::thread::sleep(Duration::from_secs(60));


        /*pwm_servo.set_duty(duty_min)?;
        std::thread::sleep(Duration::from_secs(5));

        pwm_servo.set_duty(duty_mid)?;
        std::thread::sleep(Duration::from_secs(5));

        pwm_servo.set_duty(duty_max)?;
        std::thread::sleep(Duration::from_secs(5));*/
    }
}