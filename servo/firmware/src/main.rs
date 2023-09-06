#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;

use stm32f0xx_hal as hal;

use hal::{
    pac,
    prelude::*,
    pwm,
    time::{Hertz, KiloHertz},
    timers::*,
};
use stm32f0xx_hal::usb::{Peripheral, UsbBus};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use core::fmt::Write;

/*
 * Spin the servo back and forth
 * and print Hello World! every second
 */

#[entry]
fn main() -> ! {
    let mut dp = pac::Peripherals::take().unwrap();

    stm32f0xx_hal::usb::remap_pins(&mut dp.RCC, &mut dp.SYSCFG);
    let mut rcc = dp
        .RCC
        .configure()
        .hsi48()
        .enable_crs(dp.CRS)
        .sysclk(48.mhz())
        .pclk(24.mhz())
        .freeze(&mut dp.FLASH);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let channel = cortex_m::interrupt::free(move |cs| gpioa.pa6.into_alternate_af1(cs));

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: gpioa.pa12,
    };

    let usb_bus = UsbBus::new(usb);

    let mut serial = SerialPort::new(&usb_bus);
    let mut buf: [u8; 64] = [0; 64];

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();

    let mut pwm = pwm::tim3(dp.TIM3, channel, &mut rcc, 50.hz());

    // should be 64000
    //let max_duty = pwm.get_max_duty();

    let min = 1500;
    let mut curr = min;
    let max = 8200;

    let mut timer = Timer::tim1(dp.TIM1, KiloHertz(1), &mut rcc);
    let mut tim2 = Timer::tim2(dp.TIM2, Hertz(1), &mut rcc);

    let mut up = true;

    pwm.enable();
    loop {
        if let Ok(()) = timer.wait() {
            if up {
                curr += 5;
            } else {
                curr -= 5;
            }
            if curr > max {
                up = false;
                curr = max;
            }
            if curr < min {
                up = true;
                curr = min;
            }

            pwm.set_duty(curr);
        }
        if let Ok(()) = tim2.wait() {
            write!(FixedWriter(&mut buf, 0), "Hello World!\r\n").ok();
            serial.write(&buf).ok();
        }
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }
    }
}

// Writes up to the capacity of the buffer but not more, without erroring out
struct FixedWriter<'a>(&'a mut [u8], usize);
impl<'a> Write for FixedWriter<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for c in s.chars() {
            self.1 += 1;
            if self.1 < self.0.len() {
                self.0[self.1] = c as u8;
            }
        }
        Ok(())
    }
}
