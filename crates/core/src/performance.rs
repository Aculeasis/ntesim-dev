//! Operand observations for the explicitly enabled benchmark build.
use serde::Serialize;
use std::cell::RefCell;

#[derive(Default, Serialize)]
pub struct GcdOperands {
    zero: u64,
    one: u64,
    u32: u64,
    u64: u64,
    wide: u64,
    samples: Vec<[u64; 2]>,
}

impl GcdOperands {
    fn record(&mut self, a: u128, b: u128) {
        if a == 0 || b == 0 {
            self.zero += 1;
        } else if a == 1 || b == 1 {
            self.one += 1;
        } else if (a | b) <= u128::from(u64::MAX) {
            if (a | b) <= u128::from(u32::MAX) {
                self.u32 += 1;
            } else {
                self.u64 += 1;
            }
            if self.samples.len() < 4096 && (self.u32 + self.u64) % 127 == 1 {
                self.samples.push([a as u64, b as u64]);
            }
        } else {
            self.wide += 1;
        }
    }
}

#[derive(Default, Serialize)]
pub struct ArithmeticOperands {
    fraction: GcdOperands,
    modifiers: GcdOperands,
}

thread_local! {
    static OPERANDS: RefCell<ArithmeticOperands> = RefCell::default();
}

pub(crate) fn fraction_gcd(a: u128, b: u128) {
    OPERANDS.with_borrow_mut(|operands| operands.fraction.record(a, b));
}

pub(crate) fn modifier_gcd(a: u128, b: u128) {
    OPERANDS.with_borrow_mut(|operands| operands.modifiers.record(a, b));
}

pub fn take_arithmetic_operands() -> ArithmeticOperands {
    OPERANDS.with_borrow_mut(std::mem::take)
}
