
#![allow(dead_code)]

mod ops;
pub use self::ops::{JOp, parse_java_op};
mod tls_pos;
use self::tls_pos::set_pos_init;

/// Attempts to read all opcodes in a buffer
///
/// If this returns _none_ that signifies an error occured
///
/// Don't call this function on incomplete bytecode sections
/// that start at an artibrary offset, java bytecode parsing
/// as to have _some state_ about its offset into the file.
pub fn read_all_ops<R: AsRef<[u8]>>(buffer: R) -> Vec<JOp> {
    use super::nom::IResult;
    set_pos_init(0);
    let mut buffer: &[u8] = buffer.as_ref();
    let mut ret = Vec::new();
    loop {
        buffer = match parse_java_op(buffer) {
            IResult::Done(x,y) => {
                ret.push(y);
                x
            },
            IResult::Error(_) => return Vec::new(),
            IResult::Incomplete(_) => break
        };
    }
    ret
}
