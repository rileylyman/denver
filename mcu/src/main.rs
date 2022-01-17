#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m::delay::Delay;
use cortex_m_rt::entry; // The runtime
use rfm69::Rfm69;
use stm32_hal2::{
    self,
    clocks::Clocks,
    gpio::{Pin, PinMode, Port},
    pac::{self, SPI1},
    spi::{BaudRate, Spi, SpiConfig},
};

use defmt::{debug, error, info};
use defmt_rtt as _;

fn send(rfm: &mut Rfm69<Pin, Spi<SPI1>, Delay>, msg: &str) {
    if let Err(e) = rfm.send(msg.as_bytes()) {
        // error!("{}", e);
        panic!("{:?}", e);
    }
}

// This marks the entrypoint of our application. The cortex_m_rt creates some
// startup code before this, but we don't need to worry about this
#[entry]
fn main() -> ! {
    // Get handles to the hardware objects. These functions can only be called
    // once, so that the borrowchecker can ensure you don't reconfigure
    // something by accident.
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut dp = pac::Peripherals::take().unwrap();

    // this line is required if you want to take advantage of ST-Link
    // stm32_hal2::debug_workaround();

    defmt::println!("Hello, world!");

    let clock_cfg = Clocks::default();
    clock_cfg.setup().unwrap();

    let mut cs = Pin::new(Port::A, 4, PinMode::Output);
    cs.set_high();

    let _spi1_sck = Pin::new(Port::A, 5, PinMode::Alt(5));
    let _spi1_miso = Pin::new(Port::A, 6, PinMode::Alt(5));
    let _spi1_mosi = Pin::new(Port::A, 7, PinMode::Alt(5));
    let spi1 = Spi::new(dp.SPI1, SpiConfig::default(), BaudRate::Div32);

    let mut delay = Delay::new(cp.SYST, clock_cfg.systick());

    info!("After delay");
    let mut rfm = Rfm69::new(spi1, cs, delay);

    debug!("Gonna check.");
    defmt::flush();
    info!("{:?}", rfm.read_all_regs().unwrap_or([0xab; 79]));

    let msg = "Hello from me.";

    let mut led = Pin::new(Port::B, 3, PinMode::Output);
    loop {
        led.set_low();
        send(&mut rfm, msg);
        led.set_high();
        send(&mut rfm, msg);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!(
        "Panic occurred: payload={:?}",
        info.payload().downcast_ref::<&str>()
    );
    cortex_m::asm::udf()
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
