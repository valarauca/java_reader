
#[macro_use]
extern crate nom;
use nom::IResult;

mod clazz;

use self::clazz::{
    PreClass,
    parse_pre_class
};

use std::io::prelude::*;
use std::fs::File;


fn get_args() -> Vec<String> {
    ::std::env::args().skip(1).collect()
}

fn main() {
    let args = get_args();    
    let mut f = File::open(&args[0]).unwrap();
    let mut v = Vec::<u8>::with_capacity(4096);
    f.read_to_end(&mut v).unwrap();
    let out = match parse_pre_class(v.as_slice()) {
        IResult::Done(_, x) => x,
        IResult::Error(e) => panic!("{:#?}", e),
        IResult::Incomplete(n) => panic!("{:?}",n)
    };
    println!("{:#?}",out);
}
