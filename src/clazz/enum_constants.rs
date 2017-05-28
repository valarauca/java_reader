#![allow(dead_code)]

use super::super::nom::{
    be_u8,
    be_u16,
    IResult,
    ErrorKind
};
use std::ops::BitAnd;
/*
 * Used to define Enumerators
 *
 */
macro_rules! EnumBuilder {
    (@U8
        ENUM_NAME: $NAME: ident;
        VALUES { $($VARNAME: ident => $VARVAL: expr),*};
        NOM_PARSER: $PARSE: ident;
        ERROR_CODE: $CODE: expr;
        EXTERIOR_PARSER: $EXTERIOR_PARSE: ident;
        CONVERT { $($VARTYPE: ty),* };
    ) => {
        #[repr(u8)]
        #[derive(Copy,Clone,Debug,PartialEq,Eq)]
        pub enum $NAME {
            $($VARNAME = $VARVAL),*
        }
        impl $NAME {
            pub fn and_mask(val: u16) -> Vec<$NAME> {
                let mut ret_val = Vec::new();
                $(
                    if ($VARVAL & val.clone()) > 0 {
                        ret_val.push($NAME::$VARNAME);
                    }
                 )*
                 ret_val
            }
        }
        impl AsRef<u8> for $NAME {
            #[inline(always)]
            fn as_ref(&self) -> &u8 {
                unsafe { ::std::mem::transmute(self) }
            }
        }
        $(
            impl Into<$VARTYPE> for $NAME {
                #[inline(always)]
                fn into(self) -> $VARTYPE {
                    let x: u8 = self as u8;
                    x as $VARTYPE
                }
            }
         )*
        #[inline(always)]
        pub fn $EXTERIOR_PARSE<'a>(buffer: &'a [u8]) -> IResult<&'a [u8], $NAME> {
            match $PARSE(buffer) {
                IResult::Done(rem, var) => {
                    let mut ret_val = IResult::Error(ErrorKind::Custom($CODE));
                    $(
                        if $VARVAL == var.clone() {
                            ret_val = IResult::Done(rem, $NAME::$VARNAME);
                        }
                     )*
                    ret_val
                },
                IResult::Error(e) => IResult::Error(e),
                IResult::Incomplete(n) => IResult::Incomplete(n)
            }
        }
    };
    (@U16
        ENUM_NAME: $NAME: ident;
        VALUES { $($VARNAME: ident => $VARVAL: expr),*};
        NOM_PARSER: $PARSE: ident;
        ERROR_CODE: $CODE: expr;
        EXTERIOR_PARSER: $EXTERIOR_PARSE: ident;
        CONVERT { $($VARTYPE: ty),* };
    ) => {
        #[repr(u16)]
        #[derive(Copy,Clone,Debug,PartialEq,Eq)]
        pub enum $NAME {
            $($VARNAME = $VARVAL),*
        }
        impl $NAME {
            pub fn and_mask(val: u16) -> Vec<$NAME> {
                let mut ret_val = Vec::new();
                $(
                    if ($VARVAL & val.clone()) > 0 {
                        ret_val.push($NAME::$VARNAME);
                    }
                 )*
                 ret_val
            }
        }
        impl AsRef<u16> for $NAME {
            #[inline(always)]
            fn as_ref(&self) -> &u16 {
                unsafe { ::std::mem::transmute(self) }
            }
        }
        $(
            impl Into<$VARTYPE> for $NAME {
                #[inline(always)]
                fn into(self) -> $VARTYPE {
                    let x: u16 = self as u16;
                    x as $VARTYPE
                }
            }
         )*
        #[inline]
        pub fn $EXTERIOR_PARSE<'a>(buffer: &'a [u8]) -> IResult<&'a [u8], $NAME> {
            match $PARSE(buffer) {
                IResult::Done(rem, var) => {
                    let mut ret_val = IResult::Error(ErrorKind::Custom($CODE));
                    $(
                        if $VARVAL == var.clone() {
                            ret_val = IResult::Done(rem, $NAME::$VARNAME);
                        }
                     )*
                    ret_val
                },
                IResult::Error(e) => IResult::Error(e),
                IResult::Incomplete(n) => IResult::Incomplete(n)
            }
        }
    };
}


/*
 * Class Access Flags
 *
 */
EnumBuilder! {@U16
    ENUM_NAME: ClassAccessFlags;
    VALUES {
        Public => 0x0001,
        Final => 0x0010,
        Super => 0x0020,
        Interface => 0x0200,
        Abstract => 0x0400,
        Synthetic => 0x1000,
        Annotation => 0x2000,
        Enum => 0x4000
    };
    NOM_PARSER: be_u16;
    ERROR_CODE: 1u32;
    EXTERIOR_PARSER: parse_class_access_flag;
    CONVERT {
        u16, u32, u64, usize
    };
}




/*
 * Class Structure Pool Tags
 *
 */
EnumBuilder! {@U8
    ENUM_NAME: PoolTag;
    VALUES {
        Class => 7, 
        FieldRef => 9,
        MethodRef => 10,
        InterfaceMethodRef => 11,
        Str => 8,
        Integer => 3,
        Float => 4,
        Long => 5,
        Double => 6,
        NameAndType => 12,
        Utf8 => 1,
        MethodHandle => 15,
        MethodType => 16,
        InvokeDynamic => 18
    };
    NOM_PARSER: be_u8;
    ERROR_CODE: 2u32;
    EXTERIOR_PARSER: parse_const_pool_tag;
    CONVERT {
        u8, u16, u32, u64, usize
    };
}

/*
 * Method Descriptors
 *
 */
EnumBuilder! {@U8
    ENUM_NAME: MethodDescriptor;
    VALUES {
        GetField => 1,
        GetStatic => 2,
        PutField => 3,
        PutStatic => 4,
        InvokeVirtual => 5,
        InvokeStatic => 6,
        InvokeSpecial => 7,
        NewInvokeSpecial => 8,
        InvokeInterface => 9
    };
    NOM_PARSER: be_u8;
    ERROR_CODE: 3u32;
    EXTERIOR_PARSER: parse_method_descriptor;
    CONVERT {
        u8, u16, u32, u64, usize
    };
}


/*
 * Field Access Flags
 *
 */
EnumBuilder! {@U16
    ENUM_NAME: FieldAccessFlags;
    VALUES {
        Public => 0x0001,
        Private => 0x0002,
        Protected => 0x0004,
        Static => 0x0008,
        Final => 0x0010,
        Volatile => 0x0040,
        Synthetic => 0x1000,
        Enum => 0x4000
    };
    NOM_PARSER: be_u16;
    ERROR_CODE: 6u32;
    EXTERIOR_PARSER: parse_field_access_flag;
    CONVERT {
        u16, u32, u64, usize
    };
}

/*
 * Method Access Flags
 *
 */
EnumBuilder! {@U16
    ENUM_NAME: MethodAccessFlags;
    VALUES {
        Public => 0x0001,
        Private => 0x0002,
        Protected => 0x0004,
        Static => 0x0008,
        Final => 0x0010,
        Synchronized => 0x0020,
        Bridge => 0x0040,
        Varargs => 0x0080,
        Native => 0x0100,
        Abstract => 0x0400,
        Strict => 0x0800,
        Synthetic => 0x1000
    };
    NOM_PARSER: be_u16;
    ERROR_CODE: 7u32;
    EXTERIOR_PARSER: parse_method_access_flag;
    CONVERT {
        u16, u32, u64, usize
    };
}
