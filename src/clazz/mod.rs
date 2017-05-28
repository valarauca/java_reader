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

use std::borrow::Cow;

/// Pre-Class
///
/// Needs to have its constants propigated
#[derive(Debug)]
struct PreClass<'a> {
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
impl<'a> PreClass<'a> {

    #[inline(always)]
    fn len(&self) -> usize {
        self.constants.len()
    }

    /// Attempts to rsolve a constant utf8 string
    fn get_utf8(&self, index: u16) -> Option<String> {
        let index = index as usize;
        if index >= self.len() {
            return None;
        }
        self.constants[index].get_utf8()
    }

    ///Resolves the u16 -> CLASS -> u16 -> UTF8 -> Cow<'a,str> messiness
    fn get_class(&self, index: u16) -> Option<String> {
        let index = index as usize;
        if index >= self.len() {
            return None;
        }
        match self.constants[index].get_class() {
            Option::Some(class_index) => self.constants[class_index].get_utf8(),
            Option::None => None
        }
    }
    fn get_this_class(&self) -> Option<String> {
        self.get_class(self.this_class)
    }
    fn get_super_class(&self) -> Option<String> {
        self.get_class(self.super_class)
    }
    fn get_interfaces(&self) -> Vec<String> {
        if self.interfaces.len() == 0 {
            //defer allocation
            return Vec::with_capacity(0);
        }
        self.interfaces
            .iter()
            .map(|x| x.clone())
            .filter_map(|x| self.get_class(x))
            .collect()
    }
}
/*
 * Reads a class File
 *
 */
named!( parse_pre_class<PreClass>, do_parse!(
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

///Reads a class file and attempts to build it into a clear
/// easy to read structure
#[derive(Debug)]
pub struct Class {
    pub minor_version: u16,
    pub major_version: u16,
    pub access_flags: Vec<ClassAccessFlags>,
    pub this_class: String,
    pub super_class: String,
    pub interfaces: Vec<String>
}
impl Class {

    /// Parse a class file
    pub fn parse(buffer: &[u8]) -> Result<Class, String> {
        let preclass = match parse_pre_class(buffer) {
            IResult::Done(_, x) => x,
            IResult::Error(e) => return Err(format!("parsing error {:#?}", e)),
            IResult::Incomplete(_) => return Err(format!("File too short"))
        };
        let minor = preclass.minor_version.clone();
        let major = preclass.major_version.clone();
        let flags = ClassAccessFlags::and_mask(preclass.access_flags);
        let this_class = match preclass.get_this_class() {
            Option::Some(x) => x,
            Option::None => return Err(format!("Could not resolve this class's type description"))
        };
        let super_class = match preclass.get_this_class() {
            Option::Some(x) => x,
            Option::None => return Err(format!("Could not resolve this class's super class type description"))
        };
        let interfaces = match preclass.get_interfaces() {
            Option::Some(x) => x,
            Option::None => return Err(format!("Could not resolve interfaces"))
        };
        Ok(Class {
            minor_version: minor,
            major_version: major,
            access_flags: flags,
            this_class: this_class,
            super_class: super_class,
            interfaces:interfaces
        })
    }
}
