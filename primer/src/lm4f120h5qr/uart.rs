//! Implements support for the LM4F120 UARTs

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

use core::intrinsics::{volatile_store, volatile_load};
use core::ptr::Unique;
use super::registers;
use super::gpio;

// ****************************************************************************
//
// Public Types
//
// ****************************************************************************

/// This chip has 8 UARTs
#[derive(PartialEq, Clone, Copy)]
pub enum UartId {
    Uart0,
    Uart1,
    Uart2,
    Uart3,
    Uart4,
    Uart5,
    Uart6,
    Uart7,
}

/// Controls a single UART
/// Only supports 8/N/1 - who needs anything else?
pub struct Uart {
    id: UartId,
    baud: u32,
    reg: Unique<registers::UartRegisters>,
}

// ****************************************************************************
//
// Private Types
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Public Data
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Public Functions
//
// ****************************************************************************

impl Uart {
    pub fn new(id: UartId, baud: u32) -> Uart {
        let mut uart = Uart {
            id: id,
            baud: baud,
            reg: get_uart_registers(id),
        };
        uart.init();
        uart
    }

    pub fn init(&mut self) -> () {
        // Do GPIO pin muxing
        gpio::enable_uart(self.id);
        // Enable UART module in RCGUART register p306
        unsafe {
            let mut reg: usize = volatile_load(registers::SYSCTL_RCGCUART_R);
            reg |= match self.id {
                UartId::Uart0 => 1 << 0,
                UartId::Uart1 => 1 << 1,
                UartId::Uart2 => 1 << 2,
                UartId::Uart3 => 1 << 3,
                UartId::Uart4 => 1 << 4,
                UartId::Uart5 => 1 << 5,
                UartId::Uart6 => 1 << 6,
                UartId::Uart7 => 1 << 7,
            };
            volatile_store(registers::SYSCTL_RCGCUART_R, reg);
        }
        // Disable UART and all features
        unsafe {
            self.reg.get_mut().ctl = 0;
        }
        // Calculate the baud rate values
        unsafe {
            // baud_div = CLOCK_RATE / (16 * baud_rate);
            // baud_int = round(baud_div * 64)
            let baud_int: u32 = (((66000000 * 8) / self.baud) + 1) / 2;
            // Store the upper and lower parts of the divider
            self.reg.get_mut().ibrd = (baud_int / 64) as usize;
            self.reg.get_mut().fbrd = (baud_int % 64) as usize;
        }
        // Calculate the UART Line Control register value
        unsafe {
            // 8N1
            self.reg.get_mut().lcrh = registers::UART_LCRH_WLEN_8;
        }
        // Clear the flags
        unsafe {
            self.reg.get_mut().rf = 0;
        }
        // Enable
        unsafe {
            self.reg.get_mut().ctl = registers::UART_CTL_RXE | registers::UART_CTL_TXE |
                                     registers::UART_CTL_UARTEN;
        }
    }

    fn putc(&mut self, value: u8) {
        unsafe {
            while (self.reg.get_mut().rf & registers::UART_FR_TXFF) != 0 {
                asm!("NOP");
            }
            self.reg.get_mut().data = value as usize;
        }
    }
}

impl ::core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for byte in s.bytes() {
            self.putc(byte)
        }
        Ok(())
    }
}

// ****************************************************************************
//
// Private Functions
//
// ****************************************************************************

fn get_uart_registers(uart: UartId) -> Unique<registers::UartRegisters> {
    match uart {
        UartId::Uart0 => unsafe { Unique::new(registers::UART0_DR_R as *mut _) },
        UartId::Uart1 => unsafe { Unique::new(registers::UART1_DR_R as *mut _) },
        UartId::Uart2 => unsafe { Unique::new(registers::UART2_DR_R as *mut _) },
        UartId::Uart3 => unsafe { Unique::new(registers::UART3_DR_R as *mut _) },
        UartId::Uart4 => unsafe { Unique::new(registers::UART4_DR_R as *mut _) },
        UartId::Uart5 => unsafe { Unique::new(registers::UART5_DR_R as *mut _) },
        UartId::Uart6 => unsafe { Unique::new(registers::UART6_DR_R as *mut _) },
        UartId::Uart7 => unsafe { Unique::new(registers::UART7_DR_R as *mut _) },
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
