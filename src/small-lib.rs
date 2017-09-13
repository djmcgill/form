# ! [ cfg_attr ( feature = "rt" , feature ( asm ) ) ]

extern crate cortex_m_rt ;

use core::ops::Deref;

use bare_metal::Peripheral;
pub const NVIC_PRIO_BITS: u8 = 2;

#[doc(hidden)]
pub mod interrupt {
    pub const INTERRUPT_CONST: u8 = 3;
}

pub use cortex_m::peripheral::TPIU;
pub struct AC;
pub mod ac {
    pub mod ac2 {
        pub const AC2: AC = AC;    
        pub mod ac3 {
            pub const AC3: AC = AC;    
        }
    }
    pub const AC: AC = AC;
}
