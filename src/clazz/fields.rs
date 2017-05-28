
use super::super::nom::be_u16;
use super::enum_constants::{
    FieldAccessFlags,
    parse_field_access_flag
};
use super::attributes::{
    AttributeInfo,
    parse_attribute
};

/// Holds Information about a field
#[derive(Debug)]
pub struct FieldInfo<'a> {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo<'a>>
}

named!(pub parse_field<FieldInfo>, do_parse!(
    flags: be_u16 >>
    name: be_u16 >>
    desc: be_u16 >>
    attr: be_u16 >>
    attr_vec: count!( parse_attribute, attr as usize) >>
    (FieldInfo {
        access_flags: flags,
        name_index: name,
        descriptor_index: desc,
        attributes: attr_vec
    })
));
