#![allow(dead_code)]
use super::nom::{
    be_u16,
    IResult,
    ErrorKind,
    Needed
};

mod enum_constants;
pub use self::enum_constants::{
    MethodAccessFlags,
    FieldAccessFlags,
    MethodDescriptor,
    ClassAccessFlags
};
use self::enum_constants::parse_class_access_flag;

mod const_pool;
pub use self::const_pool::{
    PoolMembers,
    ConstantsPool
};
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

use std::borrow::Cow;

/// Pre-Class
///
/// Needs to have its constants propigated
#[derive(Debug)]
pub struct Class<'a> {
    minor_version: u16,
    major_version: u16,
    constants: ConstantsPool<'a>,
    access_flags: u16,
    this_class: u16,
    super_class: u16,
    interfaces: Vec<u16>,
    fields: Vec<FieldInfo<'a>>,
    methods: Vec<MethodInfo<'a>>,
    attributes: Vec<AttributeInfo<'a>>
}
/*
 * Reads a class File
 *
 */
named!( parse_pre_class<Class>, do_parse!(
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
    (Class {
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


///Common Errors
///
///If you see any with well formed jars please submit a bug report
#[derive(Clone,Copy,Debug)]
pub enum Fault {
    
    /// Your class file failed to parse
    ParseError,
    ClassLookUpFailure,
    UTF8LookupFailure
}

impl<'a> Class<'a> {

    /// Attempt to parse a class from a buffer of bytes
    pub fn parse(buffer: &'a [u8]) -> Result<Class<'a>, Fault> {
        match parse_pre_class(buffer) {
            IResult::Done(_, item) => Ok(item),
            IResult::Error(_) |
            IResult::Incomplete(_) => Err(Fault::ParseError)
        }
    }

    /// What is this class's name
    pub fn get_this_class<'b>(&'b self) -> Result<Cow<'b, str>, Fault> {
        match self.constants.get_class_name(self.this_class.clone()) {
            Option::Some(var) => Ok(var),
            Option::None => Err(Fault::ClassLookUpFailure)
        }
    }

    /// What is the super class name
    pub fn get_super_class<'b>(&'b self) -> Result<Cow<'b, str>, Fault> {
        match self.constants.get_class_name(self.super_class.clone()) {
            Option::Some(var) => Ok(var),
            Option::None => Err(Fault::ClassLookUpFailure)
        }
    }

    pub fn get_interfaces_count(&self) -> usize {
        self.interfaces.len()
    }
    pub fn get_fields_count(&self) -> usize {
        self.fields.len()
    }
    pub fn get_methods_count(&self) -> usize {
        self.methods.len()
    }
    
    ///Attempts to resolve an interface with its index
    pub fn get_interfaces<'b>(&'b self) -> Result<Vec<Cow<'b, str>>,Fault> {
        let mut retvec = Vec::with_capacity(0);
        for iface in self.interfaces.iter() {
            match self.constants.get_class_name(iface.clone()) {
                Option::Some(var) => retvec.push(var),
                Option::None => return Err(Fault::ClassLookUpFailure)
            }
        }
        Ok(retvec)
    }

    /// Reads all fields
    ///
    /// Returns a tuple of `(Name,Descriptor)` for each field
    pub fn get_fields<'b>(&'b self) -> Result<Vec<(Cow<'b, str>,Cow<'b,str>)>,Fault> {
        let mut retvec = Vec::with_capacity(0);
        for field in self.fields.iter() {
            let name = match self.constants.get_utf8(field.name_index.clone()) {
                Option::Some(var) => var,
                Option::None => return Err(Fault::UTF8LookupFailure)
            };
            let desc = match self.constants.get_utf8(field.descriptor_index.clone()) {
                Option::Some(var) => var,
                Option::None => return Err(Fault::UTF8LookupFailure)
            };
            retvec.push( (name,desc) );
        }
        Ok(retvec)
    }
    
    /// Reads all methods
    ///
    /// Returns a tuple of `(Name,Descriptor)` for each field
    pub fn get_methods<'b>(&'b self) -> Result<Vec<(Cow<'b, str>,Cow<'b,str>)>,Fault> {
        let mut retvec = Vec::with_capacity(0);
        for method in self.methods.iter() {
            let name = match self.constants.get_utf8(method.name_index.clone()) {
                Option::Some(var) => var,
                Option::None => return Err(Fault::UTF8LookupFailure)
            };
            let desc = match self.constants.get_utf8(method.descriptor_index.clone()) {
                Option::Some(var) => var,
                Option::None => return Err(Fault::UTF8LookupFailure)
            };
            retvec.push( (name,desc) );
        }
        Ok(retvec)
    }
}

