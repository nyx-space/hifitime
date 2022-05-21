#![no_main]
#![no_std]


use hal::pac::Peripherals;
use stm32f7xx_hal as hal;
use cortex_m_rt;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use crate::hal::{
    prelude::*, 
    pac, 
    serial::{self, Serial}, 
    can::Can,
    delay::Delay
};
use rtt_target as rtt;
use rand_core::RngCore;
use hifitime as ht;
use core::sync::atomic::{self, Ordering};
use core::fmt::Write;


const TEST_VALUES : u32 = 20;
const ITERS_PER_TEST : u32 = 200;
const FREQ : u32 = 48;

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr
        .sysclk(FREQ.MHz())
        .freeze();

    let mut delay = Delay::new(cp.SYST, clocks);
    let mut rng = p.RNG.init();
    let mut dcb = cp.DCB;
    dcb.enable_trace();
    cortex_m::peripheral::DWT::unlock();
    let mut dwt = cp.DWT;
    dwt.enable_cycle_counter();
    rtt::rtt_init_print!();

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();

    let tx = gpioa.pa9.into_alternate();
    let rx = gpiob.pb7.into_alternate();

    let serial = Serial::new(
        p.USART1,
        (tx, rx),
        clocks,
        Default::default() // 115200 bps
    );

    let (mut tx, mut rx) = serial.split();

    loop {
        if let Ok(val) =  rx.read() {
            break;
        }
    }




    rtt::rprintln!("Starting tests");
    writeln!(tx, "Starting tests");

    for test_index in 0..TEST_VALUES {
        let mut buf = [0u8; core::mem::size_of::<i64>()];
        rng.fill_bytes(&mut buf);
        let mut val = i64::from_be_bytes(buf);

        let start = cortex_m::peripheral::DWT::get_cycle_count();
        for iter in 0..ITERS_PER_TEST {
            let e = ht::Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33);
            e.as_tdb_seconds();
            e.as_jde_et_days();

            let f: ht::Epoch = e + val * ht::Unit::Second;
            f.as_tdb_seconds();
            f.as_jde_et_days();

            // Should avoid the loop from being optimized away
            atomic::compiler_fence(Ordering::SeqCst);

        }
        let end = cortex_m::peripheral::DWT::get_cycle_count();
        writeln!(tx, "TBD seconds and JDE ET : {} cycles for {} iterations at val = {}  @ {} MHz", end - start, ITERS_PER_TEST, val, FREQ);
        delay.delay_ms(5000_u16);
    }


    for test_index in 0..TEST_VALUES {
        let mut buf = [0u8; core::mem::size_of::<f64>()];
        rng.fill_bytes(&mut buf);
        let mut val = f64::from_be_bytes(buf);

        let start = cortex_m::peripheral::DWT::get_cycle_count();
        for iter in 0..ITERS_PER_TEST {
            let e = ht::Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33);
            e.as_tt_seconds();

            let f: ht::Epoch = e + val * ht::Unit::Second;
            f.as_tt_seconds();

            // Should avoid the loop from being optimized away
            atomic::compiler_fence(Ordering::SeqCst);

        }
        let end = cortex_m::peripheral::DWT::get_cycle_count();
        writeln!(tx, "TT : {} cycles for {} iterations at val = {}  @ {} MHz", end - start, ITERS_PER_TEST, val, FREQ);
        delay.delay_ms(5000_u16);
    }
    

    for test_index in 0..TEST_VALUES {
        let mut buf = [0u8; core::mem::size_of::<f64>()];
        rng.fill_bytes(&mut buf);
        let mut val = f64::from_be_bytes(buf);

        let start = cortex_m::peripheral::DWT::get_cycle_count();
        for iter in 0..ITERS_PER_TEST {
            let d: ht::Duration = ht::Unit::Second * val;
            d.in_seconds();

            // Should avoid the loop from being optimized away
            atomic::compiler_fence(Ordering::SeqCst);

        }
        let end = cortex_m::peripheral::DWT::get_cycle_count();
        writeln!(tx, "Duration to f64 seconds : {} cycles for {} iterations at val = {}  @ {} MHz", end - start, ITERS_PER_TEST, val, FREQ);
        delay.delay_ms(5000_u16);
    }
    
    for test_index in 0..TEST_VALUES {
        let mut buf1 = [0u8; core::mem::size_of::<i64>()];
        rng.fill_bytes(&mut buf1);
        let mut val1 = i64::from_be_bytes(buf1);
        let mut val2 = val1 as f64;

        let mut buf2 = [0u8; core::mem::size_of::<i64>()];
        rng.fill_bytes(&mut buf2);
        let mut val3 = i64::from_be_bytes(buf2);
        let mut val4 = val3 as f64;

        let start = cortex_m::peripheral::DWT::get_cycle_count();
        for iter in 0..ITERS_PER_TEST {
            assert_eq!(ht::Unit::Day * val1, ht::Unit::Day * val2);
            assert_eq!(ht::Unit::Hour * val3, ht::Unit::Hour * val4);

            // Should avoid the loop from being optimized away
            atomic::compiler_fence(Ordering::SeqCst);

        }
        let end = cortex_m::peripheral::DWT::get_cycle_count();
        writeln!(tx, "Duration add and assert day hour : {} cycles for {} iterations at vals = {:?}  @ {} MHz", end - start, ITERS_PER_TEST, (val1, val2, val3, val4), FREQ);
        delay.delay_ms(5000_u16);
    }

    for test_index in 0..TEST_VALUES {
        let mut buf1 = [0u8; core::mem::size_of::<i64>()];
        rng.fill_bytes(&mut buf1);
        let mut val1 = i64::from_be_bytes(buf1);
        let mut val2 = val1 as f64;

        let mut buf2 = [0u8; core::mem::size_of::<i64>()];
        rng.fill_bytes(&mut buf2);
        let mut val3 = i64::from_be_bytes(buf2);
        let mut val4 = val3 as f64;

        let start = cortex_m::peripheral::DWT::get_cycle_count();
        for iter in 0..ITERS_PER_TEST {
            assert_eq!(ht::Unit::Minute * val1, ht::Unit::Minute * val2);
            assert_eq!(ht::Unit::Second * val3, ht::Unit::Second * val4);

            // Should avoid the loop from being optimized away
            atomic::compiler_fence(Ordering::SeqCst);

        }
        let end = cortex_m::peripheral::DWT::get_cycle_count();
        writeln!(tx, "Duration add and assert minute second : {} cycles for {} iterations at vals = {:?}  @ {} MHz", end - start, ITERS_PER_TEST, (val1, val2, val3, val4), FREQ);
        delay.delay_ms(5000_u16);
    }

    for test_index in 0..TEST_VALUES {
        let mut buf1 = [0u8; core::mem::size_of::<i64>()];
        rng.fill_bytes(&mut buf1);
        let mut val1 = i64::from_be_bytes(buf1);
        let mut val2 = val1 as f64;

        let mut buf2 = [0u8; core::mem::size_of::<i64>()];
        rng.fill_bytes(&mut buf2);
        let mut val3 = i64::from_be_bytes(buf2);
        let mut val4 = val3 as f64;

        let start = cortex_m::peripheral::DWT::get_cycle_count();
        for iter in 0..ITERS_PER_TEST {
            assert_eq!(
                ht::Unit::Millisecond * val1,
                ht::Unit::Millisecond * val2
            );
            assert_eq!(
                ht::Unit::Nanosecond * val3,
                ht::Unit::Nanosecond * val4
            );

            // Should avoid the loop from being optimized away
            atomic::compiler_fence(Ordering::SeqCst);

        }
        let end = cortex_m::peripheral::DWT::get_cycle_count();
        writeln!(tx, "Duration add and assert subseconds : {} cycles for {} iterations at vals = {:?}  @ {} MHz", end - start, ITERS_PER_TEST, (val1, val2, val3, val4), FREQ);
        delay.delay_ms(5000_u16);
    }



    writeln!(tx, "Tests finished");


    


    loop {
        cortex_m::asm::nop();
    }
}

#[exception]
unsafe fn HardFault(e : &ExceptionFrame) -> ! {

    rtt::rprintln!("HardFault : {:?}", e);
    // https://interrupt.memfault.com/blog/cortex-m-fault-debug
    // https://stackoverflow.com/questions/44690439

    let ufsr_addr: usize = 0xE000ED2A;
    let ufsr_data : u16 = unsafe { core::ptr::read_volatile(ufsr_addr as *const u16) };
    rtt::rprintln!("UFSR : {:016b}", ufsr_data);

    let bfsr_addr: usize = 0xE000ED29;
    let bfsr_data : u8 = unsafe { core::ptr::read_volatile(bfsr_addr as *const u8) };
    rtt::rprintln!("BFSR : {:08b}", bfsr_data);

    let bfar_valid = bfsr_data & 0b10000000 != 0;
    if bfar_valid {
        let bfar_addr: usize = 0xE000ED38;
        let bfar_data : u32 = unsafe { core::ptr::read_volatile(bfar_addr as *const u32) };
        rtt::rprintln!("-> BFAR : {:#x}", bfar_data);
    }

    let mmfsr_addr: usize = 0xE000ED28;
    let mmfsr_data : u8 = unsafe { core::ptr::read_volatile(mmfsr_addr as *const u8) };
    rtt::rprintln!("MMFSR : {:08b}", mmfsr_data);

    let hfsr_addr: usize = 0xE000ED2C;
    let hfsr_data : u32 = unsafe { core::ptr::read_volatile(hfsr_addr as *const u32) };
    rtt::rprintln!("HFSR : {:032b}", hfsr_data);

    loop {
        cortex_m::asm::nop();
    }
        
}



#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rtt::rprintln!("Panic : {:?}", info);
    loop {
        cortex_m::asm::nop();
    } 
}

/*
23:19:43.075 Starting tests
23:24:52.475 Panic : PanicInfo { payload: Any { .. }, message: Some(assertion 
failed: `(left == right)`
23:24:52.475   left: `Duration { centuries: 1, nanoseconds:
2332740342346940416 }`,
23:24:52.475  right: `Duration { centuries: -32768, nanoseconds:
3155760000000000000 }`), location: Location { file: "src\\main.rs", line: 152,
col: 13 }, can_unwind: true }

*/
