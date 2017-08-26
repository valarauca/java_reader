
//! Manages alignment of java ops.
//!
//! Some opcode have alignment restrictions when deserializing.
//! I assume this is a legacy result of RISC alignment restirctions.
//!
//! ---
//!
//! Position is stored in a thread local

#![allow(dead_code)]

use super::super::nom::{IResult, Needed};

use std::sync::atomic::{AtomicUsize, Ordering};


/*
 * Everything is stored in a thread local to make it thread safe
 */
thread_local!(static POSITION: AtomicUsize = AtomicUsize::new(0));

/// Increment the position forward by `x` value
#[inline(always)]
pub fn move_pos(x: usize) {
    POSITION.with(|f| {
        f.fetch_add(x, Ordering::Relaxed);
    });
}

/// Get the current position
#[inline(always)]
pub fn get_pos() -> usize {
    POSITION.with(|f| f.load(Ordering::Relaxed))
}

/// Ensure position is zero (this is always done when parsing starts
#[inline(always)]
pub fn set_pos_init(x: usize) {
    POSITION.with(|f| f.store(x, Ordering::Relaxed));
}

/// Handle aligning the opcodes
///
/// Also updating the `move_pos` counter.
#[inline(always)]
pub fn align_code<'a>(buffer: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let pos = POSITION.with(|f| f.load(Ordering::Relaxed)) + 1;
    let buffer_len = buffer.len();
    if buffer_len <= 4 {
        return IResult::Incomplete(Needed::Unknown);
    }
    match pos & 0b00000011usize {
        3 => {
            //this is account for preceeding opcode
            move_pos(2);
            IResult::Done(&buffer[0..1], &buffer[1..])
        },
        2 => {
            //this is account for preceeding opcode
            move_pos(3);
            IResult::Done(&buffer[0..2], &buffer[2..])
        },
        1 => {
            //this is account for preceeding opcode
            move_pos(4);
            IResult::Done(&buffer[0..3], &buffer[3..])
        },
        _ => IResult::Done(&buffer[0..0], buffer)
    }
}

