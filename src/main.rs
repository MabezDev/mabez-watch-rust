
#![feature(proc_macro)]
#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_std]
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx_hal as hal;
extern crate heapless;

use hal::dma::{CircBuffer, Event, dma1};
use hal::prelude::*;
use hal::serial::Serial;
use hal::stm32f103xx;
use rtfm::{app, Threshold};
use heapless::RingBuffer;
use rtfm::atomic;

/* Our includes */
mod msgmgr;

use msgmgr::MessageManager;
use msgmgr::Message;

const CB_HALF_LEN: usize = 64; /* Buffer size of DMA Half */
const MSG_PAYLOAD_SIZE: usize = 256; /* Body Of payload */
const MSG_COUNT: usize = 8; /* Number of message to store */

app! {
    device: stm32f103xx,

    resources: {
        static BUFFER: [[u8; CB_HALF_LEN]; 2] = [[0; CB_HALF_LEN]; 2];
        static CB: CircBuffer<[u8; CB_HALF_LEN], dma1::C6>;
        static STDOUT: cortex_m::peripheral::ITM;
        static MSG_PAYLOADS: [[u8; MSG_PAYLOAD_SIZE]; MSG_COUNT] = [[0; MSG_PAYLOAD_SIZE]; MSG_COUNT];
        static MMGR: MessageManager;
        static RB: RingBuffer<u8, [u8; 128]> = RingBuffer::new();
    },

    init: {
        resources: [BUFFER, MSG_PAYLOADS, RB],
    },

    tasks: {
        DMA1_CHANNEL6: {
            path: rx,
            resources: [CB, STDOUT, MMGR],
        },
        SYS_TICK: {
            path: sys_tick,
            resources: [STDOUT, MMGR],
        },
    }
}

fn init(p: init::Peripherals, r: init::Resources) -> init::LateResources {

    let mut itm = p.core.ITM;
    iprintln!(&mut itm.stim[0], "Hello, world!");

    /* Enable SYS_TICK IT */
    let mut syst = p.core.SYST;
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload(1_000_000); // 1_000_000 / 80_000_000, where 80_000_000 is HCLK
    syst.enable_interrupt();
    syst.enable_counter();

    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);

    /* USART2 Pins */
    let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rx = gpioa.pa3;

    /* USART 2 initialization */
    let serial = Serial::usart2(
        p.device.USART2,
        (tx, rx),
        &mut afio.mapr,
        9_600.bps(),
        clocks,
        &mut rcc.apb1,
    );

    let (_tx, rx) = serial.split();

    let mut channels = p.device.DMA1.split(&mut rcc.ahb);
    
    channels.6.listen(Event::HalfTransfer);
    channels.6.listen(Event::TransferComplete);

    /* Define out block of message - surely there must be a nice way to to this? */
    let msgs: [msgmgr::Message; 8] = [ 
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[0] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[1] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[2] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[3] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[4] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[5] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[6] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[7] },
    ]; 

    let rb: &'static mut RingBuffer<u8, [u8; 128]> = r.RB; /* Static RB for Msg recieving */
    
    /* Pass messages to the Message Manager */
    let mmgr = MessageManager::new(msgs, rb);

    init::LateResources {
        CB: rx.circ_read(channels.6, r.BUFFER),
        STDOUT : itm,
        MMGR: mmgr,
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi(); /* Wait for interrupts */
    }
}

fn rx(_t: &mut Threshold, mut r: DMA1_CHANNEL6::Resources) {
    let out = &mut r.STDOUT.stim[0];
    let mut mgr = r.MMGR;
    r.CB
        .peek(|_buf, _half| {
            mgr.write(_buf);
        })
        .unwrap();
}

fn sys_tick(_t: &mut Threshold, mut r: SYS_TICK::Resources){
    let out = &mut r.STDOUT.stim[0];
    let mut mgr = r.MMGR;
    // mgr.process(); // TODO IMPLEMENT - probably can be interrupted
    atomic(_t, |cs| { // dont interrrupt the printint process, so we run it atomically
        mgr.print_rb(out);
    });
    
}
