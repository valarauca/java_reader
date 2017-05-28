
use super::super::nom::{
    be_u16,
    be_u32,
};


///Represents an attribute
#[derive(Debug)]
pub struct AttributeInfo<'a>{
    pub name_index: u16,
    pub data: &'a [u8]
}
named!(pub parse_attribute<AttributeInfo>, do_parse!(
    index: be_u16 >>
    len: be_u32 >>
    buffer: take!(len as usize) >>
    (AttributeInfo {
        name_index: index,
        data: buffer
    })
));

