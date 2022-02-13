#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

pub use defmt::*;

use arrayvec::ArrayString;
use core::sync::atomic::{AtomicUsize, Ordering};
use embassy::blocking_mutex::raw::NoopRawMutex;
use embassy::channel::mpsc::{self, Channel, Sender, Receiver};
use embassy::executor::Spawner;
use embassy::util::Forever;
use embassy_nrf::gpio::{Input, Output, Pull, Level, OutputDrive};
use embassy_nrf::peripherals::{UARTE0, P0_06, P1_06};
use embassy_nrf::uarte::UarteRx;
use embassy_nrf::{interrupt, uarte, Peripherals};
use core::fmt::Write;

#[derive(Debug)]
enum LedState {
    On,
    Off,
}

static LED_QUEUE: Forever<Channel<NoopRawMutex, LedState, 8>> = Forever::new();
static UART_QUEUE: Forever<Channel<NoopRawMutex, ArrayString<32>, 8>> = Forever::new();

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let irq = interrupt::take!(UARTE0_UART0);
    let uart = uarte::Uarte::new(p.UARTE0, irq, p.P0_08, p.P0_12, config);
    let (mut tx, rx) = uart.split();

    let c = UART_QUEUE.put(Channel::new());
    let (s, mut r) = mpsc::split(c);

    let led_queue = LED_QUEUE.put(Channel::new());
    let (sender, receiver) = mpsc::split(led_queue);

    let button = Input::new(p.P1_06, Pull::Up);
    let led = Output::new(p.P0_06, Level::Low, OutputDrive::Standard);

    s.send(ArrayString::from("Hello World!\n").unwrap()).await.unwrap();

    spawner.spawn(led_task(led, receiver, s.clone())).unwrap();
    spawner.spawn(button_task(button, sender.clone(), s.clone())).unwrap();
    spawner.spawn(reader(rx, sender, s)).unwrap();

    loop {
        if let Some(buf) = r.recv().await {
            let _ = tx.write(buf.as_bytes()).await;
        }
    }
}

#[embassy::task]
async fn led_task(
    mut led: Output<'static, P0_06>,
    mut receiver: Receiver<'static, NoopRawMutex, LedState, 8>,
    tx: Sender<'static, NoopRawMutex, ArrayString<32>, 8>,
) {
    let mut string : ArrayString<32> = ArrayString::new();
    loop {
        if let Some(state) = receiver.recv().await {
            match state {
                LedState::On => led.set_high(),
                LedState::Off => led.set_low(),
            }
            core::writeln!(string, "LED: {:?}", state).ok();
            let _ = tx.send(string).await;
            string.clear();
        }
    }
}

#[embassy::task]
async fn button_task(
    mut button: Input<'static, P1_06>,
    sender: Sender<'static, NoopRawMutex, LedState, 8>,
    tx: Sender<'static, NoopRawMutex, ArrayString<32>, 8>,
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
async fn reader(
    mut rx: UarteRx<'static, UARTE0>,
    sender: Sender<'static, NoopRawMutex, LedState, 8>,
    tx: Sender<'static, NoopRawMutex, ArrayString<32>, 8>
) {
    let mut buf = [0; 1];
    let mut string = ArrayString::<32>::new();
    loop {
        unwrap!(rx.read(&mut buf).await);
        match buf[0] {
            b'0' => unwrap!(sender.send(LedState::Off).await),
            b'1' => unwrap!(sender.send(LedState::On).await),
            b'\n' => { unwrap!(tx.send(string).await); string.clear(); },
            c => string.push(c as char),
        }
        if string.is_full() {
            unwrap!(tx.send(string).await);
            string.clear();
        }
    }
}

defmt::timestamp! {"{=u64}", {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        let n = COUNT.load(Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Relaxed);
        n as u64
    }
}
