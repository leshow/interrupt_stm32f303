#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         // extern crate panic_itm; // logs messages over ITM; requires ITM support
                         // extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m_semihosting::hio;
use hal::{
    delay::Delay,
    gpio::{self, gpioe},
    hal::digital::v2::*,
    prelude::*,
    stm32::{self, interrupt, Peripherals},
};
use stm32f3xx_hal as hal;

// from core
use core::fmt::Write;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32::Peripherals::take().unwrap();

    let mut flash = peripherals.FLASH.constrain();
    let mut rcc = peripherals.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioa = peripherals.GPIOA.split(&mut rcc.ahb);
    let btn = gpioa
        .pa0
        .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);

    let mut gpioe = peripherals.GPIOE.split(&mut rcc.ahb);
    let mut n = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    loop {
        if btn.is_high().unwrap() {
            n.set_high().unwrap();
        } else {
            n.set_low().unwrap();
        }
        delay.delay_ms(1_00_u16);
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    if let Ok(mut hstdout) = hio::hstdout() {
        writeln!(hstdout, "{:#?}", ef).ok();
    }
    loop {}
}
