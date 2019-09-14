#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         // extern crate panic_itm; // logs messages over ITM; requires ITM support
                         // extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m::interrupt::Mutex;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m_semihosting::{hio, hprintln};
use hal::{
    delay::Delay,
    gpio::{self, gpioe},
    hal::digital::v2::*,
    prelude::*,
    stm32::{self, interrupt, Peripherals},
};
use stm32f3xx_hal as hal;

// from core
use core::{cell::RefCell, fmt::Write};

static BTN_GPIO: Mutex<RefCell<Option<gpioe::PE9<gpio::Output<gpio::PushPull>>>>> =
    Mutex::new(RefCell::new(None));
static BUTTON: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32::Peripherals::take().unwrap();

    let mut flash = peripherals.FLASH.constrain();
    let mut rcc = peripherals.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let _delay = Delay::new(cp.SYST, clocks);

    let mut gpioa = peripherals.GPIOA.split(&mut rcc.ahb);
    let _btn = gpioa
        .pa0
        .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);

    let mut gpioe = peripherals.GPIOE.split(&mut rcc.ahb);
    let n = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    cortex_m::interrupt::free(|cs| {
        BTN_GPIO.borrow(cs).replace(Some(n));
    });

    // // set source of interrupt to be PA0 (RTRM 12.1.3)
    let syscfg = &peripherals.SYSCFG;
    syscfg
        .exticr1
        .modify(|_, w| unsafe { w.exti0().bits(0b000) });
    // set interrupt to fire on rising edge of PA0 signal (RTRM - 14.3.2)
    let exti = &peripherals.EXTI;
    exti.imr1.modify(|_, w| w.mr0().set_bit());
    exti.rtsr1.modify(|_, w| w.tr0().set_bit());

    // Enable EXTI0 Interrupt
    let mut nvic = cp.NVIC;
    nvic.enable(stm32::Interrupt::EXTI0);

    loop {}
}

#[interrupt]
fn EXTI0() {
    cortex_m::interrupt::free(|cs| {
        let mut btn = BUTTON.borrow(cs).borrow_mut();
        *btn = !(*btn);
        let mut btn_gpio = BTN_GPIO.borrow(cs).borrow_mut();
        if let Some(ref mut btn_gpio) = *btn_gpio {
            if *btn {
                btn_gpio.set_high().unwrap();
            } else {
                btn_gpio.set_low().unwrap();
            }
        }
    });
    hprintln!("Interrupt caught").unwrap();

    // clear interrupt
    let exti = unsafe { Peripherals::steal() };
    exti.EXTI.pr1.modify(|_, w| w.pr0().set_bit());
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    if let Ok(mut hstdout) = hio::hstdout() {
        writeln!(hstdout, "{:#?}", ef).ok();
    }
    loop {}
}
