#![allow(dead_code)]
use super::nom::be_u16;

mod enum_constants;
pub use self::enum_constants::{
    MethodAccessFlags,
    FieldAccessFlags,
    MethodDescriptor,
    ClassAccessFlags
};
use self::enum_constants::parse_class_access_flag;

mod const_pool;
pub use self::const_pool::PoolMembers;
use self::const_pool::parse_constant_pool;

mod javautf8;
mod attributes;
pub use self::attributes::AttributeInfo;
use self::attributes::parse_attribute;

mod fields;
pub use self::fields::FieldInfo;
use self::fields::parse_field;

mod methods;
pub use self::methods::MethodInfo;
use self::methods::parse_method;


/// Pre-Class
///
/// Needs to have its constants propigated
#[derive(Debug)]
pub struct PreClass<'a> {
    pub minor_version: u16,
    pub major_version: u16,
    pub constants: Vec<PoolMembers<'a>>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo<'a>>,
    pub methods: Vec<MethodInfo<'a>>,
    pub attributes: Vec<AttributeInfo<'a>>
}

/*
 * Reads a class File
 *
 */
named!(pub parse_pre_class<PreClass>, do_parse!(
    tag!(b"\xCA\xFE\xBA\xBE") >>
    min: be_u16 >>
    maj: be_u16 >>
    constpool: parse_constant_pool >>
    acces: be_u16 >> 
    this: be_u16 >>
    sup: be_u16 >>
    ifaces: be_u16 >>
    interface: count!( be_u16, ifaces as usize) >>
    field_count: be_u16 >>
    fields: count!( parse_field, field_count as usize) >>
    method_count: be_u16 >>
    methods: count!( parse_method, method_count as usize) >>
    attri_count: be_u16 >>
    attri: count!( parse_attribute, attri_count as usize) >>
    (PreClass {
        minor_version: min,
        major_version: maj,
        constants: constpool,
        access_flags: acces,
        this_class: this,
        super_class: sup,
        interfaces: interface,
        fields: fields,
        methods: methods,
        attributes: attri
    })
));
        
