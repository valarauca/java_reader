#![allow(dead_code)]

use super::super::nom::{
    IResult,
    ErrorKind,
    Needed,
    be_u16
};
use std::borrow::Cow;

/*
 * Finite state machine to decode Java's "UTF-8" implementation
 *
 * This defines an array about ~256bytes large which should fit nicely
 * in L1
 *
 *
 */

const CODE: &[u8;256] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // Errors
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    // 192 - 223 2 bytes
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 
    // 224 - 239 3 bytes (with a 6 thrown in)
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 6, 3, 3, 
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
];


///Wraps an unsafe method
#[inline(always)]
fn get_code(x: u8) -> u8 {
    let index: usize = x.clone() as usize;
    unsafe { CODE.get_unchecked(index) }.clone()
}

/// Get a buffer
named!(get_buffer, do_parse!(
    sized: be_u16 >>
    buffer: take!(sized as usize) >>
    (buffer)
));

/// Decodes a Java UTF8 string to a standard Rust string
///
/// Reserves error codes
///
/// 4 (malformed character)
///
/// 5 (Illegal codepoint)
pub fn decode_java_utf8<'a>(buffer: &'a [u8]) -> IResult<&'a [u8], Cow<'a, str>> {
    match get_buffer(buffer) {
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(n) => IResult::Incomplete(n),
        IResult::Done(rem, buff) => {
            match ::std::str::from_utf8(buff) {
                Ok(x) => return IResult::Done(rem, Cow::Borrowed(x)),
                _ => { }
            };
            match decode(buff) {
                IResult::Error(e) => IResult::Error(e),
                IResult::Incomplete(n) => IResult::Incomplete(n),
                IResult::Done(_,var) => IResult::Done(rem, var)
            }
        }
    }
}

/*
 * Happily re-allocates the string
 */
fn decode<'a>(buffer: &'a [u8]) -> IResult<&'a [u8], Cow<'a, str>> {
    use std::char;

    let len = buffer.len();
    let mut ret = String::with_capacity(len);
    let mut index = 0;
    loop {
        if index == len {
            return IResult::Done(&buffer[0..0],Cow::Owned(ret));
        }
        let var = unsafe{buffer.get_unchecked(index)}.clone();
        index += 1;
        match get_code(var) {
            0 => {
                if var == 0 {
                    return IResult::Error(ErrorKind::Custom(5));
                }
                match char::from_u32(var as u32) {
                    Option::Some(c) => ret.push(c),
                    Option::None => return IResult::Error(ErrorKind::Custom(4))
                }
            },
            2 => {
                let x = var as u32;
                if index+1 > len {
                    return IResult::Incomplete(Needed::Size(1));
                }
                let y = unsafe{buffer.get_unchecked(index)}.clone() as u32;
                index += 1;
                let c = ((x&0x1Fu32) << 6) + (y&0x3Fu32);
                match char::from_u32(c) {
                    Option::Some(c) => ret.push(c),
                    Option::None => return IResult::Error(ErrorKind::Custom(4))
                }
            },
            3 => {
                let x = var as u32;
                if index+2 > len {
                    return IResult::Incomplete(Needed::Size(2));
                }
                let y = unsafe{buffer.get_unchecked(index)}.clone() as u32;
                index += 1;
                let z = unsafe{buffer.get_unchecked(index)}.clone() as u32;
                index += 1;
                let c = ((x&0xFu32) << 12) + ((y&0x3F) << 6) + (z&0x3F);
                match char::from_u32(c) {
                    Option::Some(c) => ret.push(c),
                    Option::None => return IResult::Error(ErrorKind::Custom(4))
                }
            },
            6 => {
                if index+5 > len {
                    return IResult::Incomplete(Needed::Size(5));
                }
                let v = unsafe{buffer.get_unchecked(index)}.clone() as u32;
                index += 1;
                let w = unsafe{buffer.get_unchecked(index)}.clone() as u32;
                index += 1;
                index += 1;
                let y = unsafe{buffer.get_unchecked(index)}.clone() as u32;
                index += 1;
                let z = unsafe{buffer.get_unchecked(index)}.clone() as u32;
                index += 1;
                let c = 0x10000u32 + ((v&0x0F) << 16) + ((w&0x3F) << 10) + ((y&0xF) << 6) + (z & 0x3F);
                match char::from_u32(c) {
                    Option::Some(c) => ret.push(c),
                    Option::None => return IResult::Error(ErrorKind::Custom(4))
                }
            },
            _ => return IResult::Error(ErrorKind::Custom(5))
        };
    }
}

#[test]
fn test_decode() {

    let (_,var) = decode(b"Hello World!").unwrap();
    assert_eq!(var, "Hello World!");
}





