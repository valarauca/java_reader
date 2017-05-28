
use super::super::nom::{
    be_u16,
    be_u32,
};

use super::PreClass;

#[derive(Copy,Clone,Debug)]
pub enum AttributeType {
    ConstantValue,
    Code,
    StackMapTable,
    Exceptions,
    InnerClasses,
    EnclosingMethods,
    Synthetic,
    Signature,
    SourceFile,
    SourceDebugExtension,
    LineNumberTable,
    LocalVariableTable,
    LocalVaraibleTypeTable,
    Deprecated,
    RuntimeVisibleAnnotations,
    RuntimeInvisibleAnnotations,
    RuntimeVisibleParameterAnnocations,
    RuntimeInvisibleParameterAnnotations,
    AnnotationDefault,
    BootstrapMethods
}
named!(parse_attr_kind<AttributeType>, switch!(peek!(1),
        b"A" => do_parse!(
            tag!(b"AnnotationDefault") >> (AttributeType::AnnotationDefault)
        )|
        b"B" => do_parse!(
            tag!(b"BootstrapMethods") >> (AttributeType::BootstrapMethods)
        )|
        b"C" => switch!(peek!(3),
            b"Con" => do_parse!(tag!(b"ConstantValue") >> (AttributeType::ConstantValue)) |
            b"Cod" => do_parse!(tag!(b"Code") >> (AttributeType::Code))) |
        b"D" => do_parse!(
            tag!(b"Deprecated") >> (AttributeType::Deprecated)
        )|
        b"E" => switch!(peek!(2),
            b"Ex" => do_parse!(tag!(b"Exceptions") >> (AttributeType::Exceptions)) |
            b"En" => do_parse!(tag!(b"EnclosingMethods") >> (AttributeType::EnclosingMethods))
        )|
        b"I" => do_parse!(
            tag!(b"InnerClasses") >> (AttributeType::InnerClasses)
        )|
        b"L" => switch!(peek!(2),
            b"Li" => do_parse!(tag!(b"LineNumberTable") >> (AttributeType::LineNumberTable)) |
            b"Lo" => alt_complete!(
                do_parse!(tag!(b"LocalVariableTable") >> (AttributeType::LocalVariableTable)) |
                do_parse!(tag!(b"LocalVaraibleTypeTable") >> (AttributeType::LocalVaraibleTypeTable))|
            )
        )|
        b"R" => switch!(peek!(8),
            b"RuntimeV" => alt_complete!(
                do_parse!(tag!(b"RuntimeVisibleAnnotations") >> (AttributeType::RuntimeVisibleAnnotations)) |
                do_parse!(tag!(b"RuntimeVisibleParameterAnnotations") >> (AttributeType::RuntimeVisibleParameterAnnotations))|
            ) |
            b"RuntimeI" => alt_complete!(
                do_parse!(tag!(b"RuntimeInvisibleParameterAnnotations") >> (AttributeType::RuntimeInvisibleParameterAnnotations)) |
                do_parse!(tag!(b"RuntimeInvisibleAnnotations") >> (AttributeType::RuntimeInvisibleAnnotations))|
            )
        )|
        b"S" => alt_complete!(
            do_parse!(tag!(b"Signature") >> (AttributeType::Signature)) |
            do_parse!(tag!(b"SourceFile") >> (AttributeType::SourceFile)) |
            do_parse!(tag!(b"SourceDebugExtension") >> (AttributeType::SourceDebugExtension)) |
            do_parse!(tag!(b"StackMapTable") >> (AttributeType::StackMapTable)) |
            do_parse!(tag!(b"Synthetic") >> (AttributeType::Synthetic))
        )
));


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

