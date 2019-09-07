#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         // extern crate panic_itm; // logs messages over ITM; requires ITM support
                         // extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m_semihosting::{hio, hprintln};
use hal::{
    delay::Delay,
    hal::digital::v2::{InputPin, OutputPin},
    prelude::*,
    stm32::{self, interrupt, Interrupt, GPIOA, GPIOE, NVIC},
};
use lazy_static::lazy_static;
use stm32f3xx_hal as hal;

// from core
use core::cell::RefCell;
use core::fmt::Write;

// lazy_static! {
//     static ref MUTEX_GPIOA: Mutex<RefCell<Option<GPIOA>>> = Mutex::new(RefCell::new(None));
//     static ref MUTEX_EXTI: Mutex<RefCell<Option<stm32::EXTI>>> = Mutex::new(RefCell::new(None));
// }

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

    // // set source of interrupt to be PA0 (RTRM 12.1.3)
    let syscfg = &peripherals.SYSCFG;
    syscfg
        .exticr1
        .modify(|_, w| unsafe { w.exti0().bits(0b000) });
    // set interrupt to fire on rising edge of PA0 signal (RTRM - 14.3.2)
    let exti = &peripherals.EXTI;
    exti.imr1.modify(|_, w| w.mr0().set_bit());
    exti.rtsr1.modify(|_, w| w.tr0().set_bit());

    // // 7. Enable EXTI0 Interrupt
    let mut nvic = cp.NVIC;
    nvic.enable(stm32::Interrupt::EXTI0);

    loop {
        // if btn.is_high().unwrap() {
        //     n.set_high().unwrap();
        // } else {
        //     n.set_low().unwrap();
        // }
        // delay.delay_ms(1_00_u16);
    }
}

#[interrupt]
fn EXTI0() {
    hprintln!("Interrupt caught").unwrap();
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    if let Ok(mut hstdout) = hio::hstdout() {
        writeln!(hstdout, "{:#?}", ef).ok();
    }

    loop {}
}
