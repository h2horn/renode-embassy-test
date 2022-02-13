#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![macro_use]

use defmt_rtt as _; // global logger
use panic_probe as _;

pub use defmt::*;

use arrayvec::ArrayString;
use core::sync::atomic::{AtomicUsize, Ordering};
use embassy::blocking_mutex::kind::Noop;
use embassy::channel::mpsc::{self, Channel, Sender, Receiver};
use embassy::executor::Spawner;
use embassy::util::Forever;
use embassy_nrf::gpio::{NoPin, Input, Output, Pull, Level, OutputDrive};
use embassy_nrf::peripherals::{UARTE0, P0_06, P1_06};
use embassy_nrf::uarte::UarteRx;
use embassy_nrf::{interrupt, uarte, Peripherals};
use core::fmt::Write;

enum LedState {
    On,
    Off,
}

static LED_QUEUE: Forever<Channel<Noop, LedState, 1>> = Forever::new();
static UART_QUEUE: Forever<Channel<Noop, ArrayString<32>, 8>> = Forever::new();

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    let irq = interrupt::take!(UARTE0_UART0);
    let uart = uarte::Uarte::new(p.UARTE0, irq, p.P0_08, p.P0_12, NoPin, NoPin, uarte::Config::default());
    let (mut tx, rx) = uart.split();

    let c = UART_QUEUE.put(Channel::new());
    let (s, mut r) = mpsc::split(c);

    let led_queue = LED_QUEUE.put(Channel::new());
    let (sender, receiver) = mpsc::split(led_queue);

    let button = Input::new(p.P1_06, Pull::Up);
    let led = Output::new(p.P0_06, Level::Low, OutputDrive::Standard);

    s.send(ArrayString::from("Hello World!\n").unwrap()).await.unwrap();

    //outb.set_high();
    //button.wait_for_low().await;
    s.send(ArrayString::from("Low!\n").unwrap()).await.unwrap();

    // Spawn a task responsible purely for reading

    spawner.spawn(led_task(led, receiver)).unwrap();
    spawner.spawn(button_task(button, sender, s.clone())).unwrap();
    spawner.spawn(reader(rx, s)).unwrap();

    // Continue reading in this main task and write
    // back out the buffer we receive from the read
    // task.
    loop {
        if let Some(buf) = r.recv().await {
            info!("writing...");
            unwrap!(tx.write(buf.as_bytes()).await);
        }
    }
}

#[embassy::task]
async fn led_task(
    mut led: Output<'static, P0_06>,
    mut receiver: Receiver<'static, Noop, LedState, 1>,
) {
    loop {
        match receiver.recv().await.unwrap() {
            LedState::On => led.set_high(),
            LedState::Off => led.set_low(),
        };
    }
}

#[embassy::task]
async fn button_task(
    mut button: Input<'static, P1_06>,
    sender: Sender<'static, Noop, LedState, 1>,
    tx: Sender<'static, Noop, ArrayString<32>, 8>,
) {
    let mut trigger_count = 0;
    let mut string : ArrayString<32> = ArrayString::new();

    loop {
        button.wait_for_low().await;

        trigger_count += 1;
        let _ = sender.send(LedState::On).await;
        core::writeln!(string, "Button pressed: {}", trigger_count).unwrap();
        let _ = tx.send(string).await;

        button.wait_for_high().await;

        let _ = sender.send(LedState::Off).await;
        core::writeln!(string, "Button released: {}", trigger_count).unwrap();
        let _ = tx.send(string).await;
    }
}

#[embassy::task]
async fn reader(mut rx: UarteRx<'static, UARTE0>, s: Sender<'static, Noop, ArrayString<32>, 8>) {
    let mut buf = [0; 32];
    loop {
        info!("reading...");
        unwrap!(rx.read(&mut buf).await);
        unwrap!(s.send(ArrayString::from_byte_string(&buf).unwrap()).await);
    }
}

defmt::timestamp! {"{=u64}", {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        let n = COUNT.load(Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Relaxed);
        n as u64
    }
}
