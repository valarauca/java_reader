#![allow(dead_code)]

use super::super::nom::{
    be_u8,
    be_u16,
    be_f32,
    be_i32,
    be_u32,
    IResult,
};
use super::enum_constants::{
    PoolTag,
    parse_const_pool_tag,
    MethodDescriptor,
    parse_method_descriptor
};
use super::javautf8::decode_java_utf8;
use std::borrow::Cow;


macro_rules! build_is {
    (@1 $NAME: ident; $KIND: ident) => {
        #[inline(always)]
        pub fn $NAME(&self) -> bool {
            match self {
                &PoolMembers::$KIND(_) => true,
                _ => false
            }
        }
    };
    (@2 $NAME: ident; $KIND: ident) => {
        #[inline(always)]
        pub fn $NAME(&self) -> bool {
            match self {
                &PoolMembers::$KIND(_,_) => true,
                _ => false
            }
        }
    };
}

/// Items within the constant pool
#[derive(Clone,Debug)]
pub enum PoolMembers<'a> {
    ///Contains the index of its name
    ClassInfo(u16),
    ///Contains the index of its class, and name_and_type
    FieldRef(u16, u16),
    ///See above
    MethodRef(u16, u16),
    /// See above
    InterfaceMethodRef(u16, u16),
    /// Pointer to its UTF8 value
    Str(u16),
    /// Literal i32
    Integer(i32),
    /// Literal f32
    Float(f32),
    Long(i64),
    Double(f64),
    NameAndType(u16, u16),
    Utf8(Cow<'a, str>),
    /// handle to a method value is a 
    MethodHandle(u8, u16),
    /// Describes a method. Value is a index in the constant pool
    /// that points to a PoolMembers::
    MethodType(u16),
    InvokeDynamic(u16, u16)
}
impl<'a> PoolMembers<'a> {
    #[inline(always)]
    fn is_double_long(&self) -> bool {
        match self {
            &PoolMembers::Double(_) |
            &PoolMembers::Long(_) => true,
            _ => false
        }
    }
    build_is!(@1 is_class_info; ClassInfo);
    build_is!(@2 is_field_ref; FieldRef);
    build_is!(@2 is_method_ref; MethodRef);
    build_is!(@2 is_interface_method_ref; InterfaceMethodRef);
    build_is!(@1 is_str; Str);
    build_is!(@1 is_integer; Integer);
    build_is!(@1 is_float; Float);
    build_is!(@1 is_long; Long);
    build_is!(@1 is_double; Double);
    build_is!(@2 is_name_and_type; NameAndType);
    build_is!(@1 is_utf8; Utf8);
    build_is!(@2 is_method_handle; MethodHandle);
    build_is!(@1 is_method_type; MethodType);
    build_is!(@2 is_invoke_dynamic; InvokeDynamic);
}


/*
 * Parse A value in the constant's pool
 *
 */
named!(parse_pool_tag<PoolMembers>, switch!( parse_const_pool_tag, 
        PoolTag::Class => do_parse!(
            v: be_u16 >>
            (PoolMembers::ClassInfo(v))) |
        PoolTag::Str => do_parse!(
            v: be_u16 >>
            (PoolMembers::Str(v))) |
        PoolTag::FieldRef => do_parse!(
            c: be_u16 >>
            n: be_u16 >>
            (PoolMembers::FieldRef(c,n))) |
        PoolTag::MethodRef => do_parse!(
            c: be_u16 >>
            n: be_u16 >>
            (PoolMembers::MethodRef(c, n))) |
        PoolTag::InterfaceMethodRef => do_parse!(
            c: be_u16 >>
            n: be_u16 >>
            (PoolMembers::InterfaceMethodRef(c, n))) |
        PoolTag::Integer => do_parse!(
            v: be_i32 >>
            (PoolMembers::Integer(v))) |
        PoolTag::Float => do_parse!(
            v: be_f32 >>
            (PoolMembers::Float(v))) |
        PoolTag::Long => do_parse!(
            hig: be_u32 >>
            low: be_u32 >>
            (PoolMembers::Long((((hig as u64) << 32) + (low as u64)) as i64)))|
        PoolTag::Double => do_parse!(
            hig: be_u32 >>
            low: be_u32 >>
            (PoolMembers::Double(unsafe { ::std::mem::transmute(((hig as u64) << 32) + (low as u64)) })))|
        PoolTag::NameAndType => do_parse!(
            n: be_u16 >>
            d: be_u16 >>
            (PoolMembers::NameAndType(n,d))) |
        PoolTag::Utf8 => do_parse!(
            buf: decode_java_utf8 >>
            (PoolMembers::Utf8(buf))) |
        PoolTag::MethodHandle => do_parse!(
            d: be_u8 >>
            i: be_u16 >>
            (PoolMembers::MethodHandle(d, i))) |
        PoolTag::MethodType => do_parse!(
            i: be_u16 >>
            (PoolMembers::MethodType(i))) |
        PoolTag::InvokeDynamic => do_parse!(
            b: be_u16 >>
            n: be_u16 >>
            (PoolMembers::InvokeDynamic(b,n)))
));

/// Parse a constant pool
pub fn parse_constant_pool<'a>(buffer: &'a [u8]) -> IResult<&'a [u8], Vec<PoolMembers>> {
    match be_u16(buffer) {
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(n) => IResult::Incomplete(n),
        IResult::Done(mut rem, pool_count) =>{
            let pool_count = (pool_count as usize) - 1;
            let mut pool = Vec::with_capacity(pool_count);
            loop {
                if pool.len() == pool_count {
                    break;
                }
                let (rm, val) = match parse_pool_tag(rem) {
                    IResult::Error(e) => return IResult::Error(e),
                    IResult::Incomplete(n) => return IResult::Incomplete(n),
                    IResult::Done(x,y) => (x,y)
                };
                rem = rm;
                pool.push(val.clone());
                if val.is_double_long() {
                    pool.push(val.clone())
                }
            }
            IResult::Done(rem, pool)
        }
    }
}
