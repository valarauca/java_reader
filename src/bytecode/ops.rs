
#![allow(non_snake_case)]

use super::super::nom::{le_u8, le_u32, le_u16, le_i32, IResult, ErrorKind, le_i16, Needed};

use super::tls_pos::{move_pos, align_code};

named!(parse_npairs<(i32,u32)>, do_parse!(
    match_var: le_i32 >>
    offset: le_u32 >>
    ( (match_var, offset) )
));

fn slen(high: i32, low: i32) -> usize {
    ((high - low) + 1) as usize
}

macro_rules! build_it_all {
    (@SIMP $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            ({
                move_pos(1);
                JOp::$b
            })
        ));
    };
    (@U8 $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var: le_u8 >>
            ({
                move_pos(2);
                JOp::$b(var)
            })
        ));
    };
    (@U8WIDE $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!(b"\xC4") >>
            tag!($a) >>
            var: le_u16 >>
            ({
                move_pos(4);
                JOp::$b(var)
            })
        ));
    };
    (@U16 $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var: le_u16 >>
            ({
                move_pos(3);
                JOp::$b(var)
            })
        ));
    };
    (@U1600 $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var: le_u16 >>
            tag!(b"\x00\x00") >>
            ({
                move_pos(5);
                JOp::$b(var)
            })
        ));
    };
    (@U32 $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var: le_u32 >>
            ({
                move_pos(5);
                JOp::$b(var)
            })
        ));
    };
    (@NPAIRS $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            call!(align_code) >>
            default_code: le_u32 >>
            pairs: le_u32 >>
            data: many_m_n!(pairs as usize, pairs as usize, parse_npairs) >>
            ({
                move_pos(8 + (data.len() * 8));
                JOp::$b(default_code, data)
            })
        ));
    };
    (@IINC $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!(b"\xC4") >>
            tag!($a) >>
            var0: le_u16 >>
            var1: le_i16 >>
            ({
                move_pos(6);
                JOp::$b(var0, var1)
            })
        ));
    };
    (@@U8 $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var0: le_u8 >>
            var1: le_u8 >>
            ({
                move_pos(3);
                JOp::$b(var0, var1)
            })
        ));
    };
    (@@U16U80 $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var0: le_u16 >>
            var1: le_u8 >>
            tag!(b"\x00") >>
            ({
                move_pos(5);
                JOp::$b(var0, var1)
            })
        ));
    };
    (@@U16U8 $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var0: le_u16 >>
            var1: le_u8 >>
            ({
                move_pos(4);
                JOp::$b(var0, var1)
            })
        ));
    };

    //exit condition
    (@TABLESWITCH $a: expr, $b: ident, $c: ident) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            call!(align_code) >>
            default: le_i32 >>
            low: le_i32 >>
            high: le_i32 >>
            data: many_m_n!(slen(high,low),slen(high,low), le_u32) >>
            ({
                move_pos(12 + (data.len() * 4));
                JOp::$b(default, low, high, data)
            })
        ));
    };
}


    build_it_all!(@SIMP       b"\x32", AALoad, parser_AALoad);
    build_it_all!(@SIMP       b"\x53", AAStore, parser_AAStore);
    build_it_all!(@SIMP       b"\x01", AConstNull, parser_AConstNull);
    build_it_all!(@U8         b"\x19", ALoad, parser_ALoad);
    build_it_all!(@U8WIDE     b"\x19", ALoadWide, parser_ALoadWide);
    build_it_all!(@SIMP       b"\x2A", ALoad0, parser_ALoad0);
    build_it_all!(@SIMP       b"\x2B", ALoad1, parser_ALoad1);
    build_it_all!(@SIMP       b"\x2C", ALoad2, parser_ALoad2);
    build_it_all!(@SIMP       b"\x2D", ALoad3, parser_ALoad3);
    build_it_all!(@U16        b"\xBD", ANewArray, parser_ANewArray);
    build_it_all!(@SIMP       b"\xB0", AReturn, parser_AReturn);
    build_it_all!(@SIMP       b"\xBE", ArrayLength, parser_ArrayLength);
    build_it_all!(@U8         b"\x3A", AStore, parser_AStore);
    build_it_all!(@U8WIDE     b"\x3A", AStoreWide, parser_AStoreWide);
    build_it_all!(@SIMP       b"\x4B", AStore0, parser_AStore0);
    build_it_all!(@SIMP       b"\x4C", AStore1, parser_AStore1);
    build_it_all!(@SIMP       b"\x4D", AStore2, parser_AStore2);
    build_it_all!(@SIMP       b"\x4E", AStore3, parser_AStore3);
    build_it_all!(@SIMP       b"\xBF", AThrow, parser_AThrow);
    build_it_all!(@SIMP       b"\x33", BaLoad, parser_BaLoad);
    build_it_all!(@SIMP       b"\x54", BaStore, parser_BaStore);
    build_it_all!(@U8         b"\x10", BiPush, parser_BiPush);
    build_it_all!(@SIMP       b"\xCA", BreakPoint, parser_BreakPoint);
    build_it_all!(@SIMP       b"\x34", CaLoad, parser_CaLoad);
    build_it_all!(@SIMP       b"\x55", CaStore, parser_CaStore);
    build_it_all!(@U16        b"\xC0", CheckCast, parser_CheckCast);
    build_it_all!(@SIMP       b"\x90", D2F, parser_D2F);
    build_it_all!(@SIMP       b"\x8E", D2I, parser_D2I);
    build_it_all!(@SIMP       b"\x8F", D2L, parser_D2L);
    build_it_all!(@SIMP       b"\x63", DAdd, parser_DAdd);
    build_it_all!(@SIMP       b"\x31", DALoad, parser_DALoad);
    build_it_all!(@SIMP       b"\x52", DAStore, parser_DAStore);
    build_it_all!(@SIMP       b"\x98", DcmpG, parser_DcmpG);
    build_it_all!(@SIMP       b"\x97", DcmpL, parser_DcmpL);
    build_it_all!(@SIMP       b"\x0E", DConst0, parser_DConst0);
    build_it_all!(@SIMP       b"\x0F", DConst1, parser_DConst1);
    build_it_all!(@SIMP       b"\x6F", DDiv, parser_DDiv);
    build_it_all!(@U8         b"\x18", DLoad, parser_DLoad);
    build_it_all!(@U8WIDE     b"\x18", DLoadWide, parser_DLoadWide);
    build_it_all!(@SIMP       b"\x26", DLoad0, parser_DLoad0);
    build_it_all!(@SIMP       b"\x27", DLoad1, parser_DLoad1);
    build_it_all!(@SIMP       b"\x28", DLoad2, parser_DLoad2);
    build_it_all!(@SIMP       b"\x29", DLoad3, parser_DLoad3);
    build_it_all!(@SIMP       b"\x6B", DMul, parser_DMul);
    build_it_all!(@SIMP       b"\x77", DNeg, parser_DNeg);
    build_it_all!(@SIMP       b"\x73", DRem, parser_DRem);
    build_it_all!(@U8         b"\xAF", DStore, parser_DStore);
    build_it_all!(@U8WIDE     b"\xAF", DStoreWide, parser_DStoreWide);
    build_it_all!(@SIMP       b"\x47", DStore0, parser_DStore0);
    build_it_all!(@SIMP       b"\x48", DStore1, parser_DStore1);
    build_it_all!(@SIMP       b"\x49", DStore2, parser_DStore2);
    build_it_all!(@SIMP       b"\x4A", DStore3, parser_DStore3);
    build_it_all!(@SIMP       b"\x67", DSub, parser_DSub);
    build_it_all!(@SIMP       b"\x59", Dup, parser_Dup);
    build_it_all!(@SIMP       b"\x5A", Dupx1, parser_Dupx1);
    build_it_all!(@SIMP       b"\x5B", Dupx2, parser_Dupx2);
    build_it_all!(@SIMP       b"\x5C", Dup2, parser_Dup2);
    build_it_all!(@SIMP       b"\x5D", Dup2x1, parser_Dup2x1);
    build_it_all!(@SIMP       b"\x5C", Dup2x2, parser_Dup2x2);
    build_it_all!(@SIMP       b"\x8D", F2D, parser_F2D);
    build_it_all!(@SIMP       b"\x8B", F2I, parser_F2I);
    build_it_all!(@SIMP       b"\x8C", F2L, parser_F2L);
    build_it_all!(@SIMP       b"\x62", FAdd, parser_FAdd);
    build_it_all!(@SIMP       b"\x30", FALoad, parser_FALoad);
    build_it_all!(@SIMP       b"\x51", FAStore, parser_FAStore);
    build_it_all!(@SIMP       b"\x96", FcmpG, parser_FcmpG);
    build_it_all!(@SIMP       b"\x95", FcmpL, parser_FcmpL);
    build_it_all!(@SIMP       b"\x0B", FConst0, parser_FConst0);
    build_it_all!(@SIMP       b"\x0C", FConst1, parser_FConst1);
    build_it_all!(@SIMP       b"\x0D", FConst2, parser_FConst2);
    build_it_all!(@SIMP       b"\x6E", FDiv, parser_FDiv);
    build_it_all!(@U8         b"\x17", FLoad, parser_FLoad);
    build_it_all!(@U8WIDE     b"\x17", FLoadWide, parser_FLoadWide);
    build_it_all!(@SIMP       b"\x22", FLoad0, parser_FLoad0);
    build_it_all!(@SIMP       b"\x23", FLoad1, parser_FLoad1);
    build_it_all!(@SIMP       b"\x24", FLoad2, parser_FLoad2);
    build_it_all!(@SIMP       b"\x25", FLoad3, parser_FLoad3);
    build_it_all!(@SIMP       b"\x6A", FMul, parser_FMul);
    build_it_all!(@SIMP       b"\x76", FNeg, parser_FNeg);
    build_it_all!(@SIMP       b"\x72", FRem, parser_FRem);
    build_it_all!(@SIMP       b"\xAE", FReturn, parser_FReturn);
    build_it_all!(@U8         b"\x38", FStore, parser_FStore);
    build_it_all!(@U8WIDE     b"\x38", FStoreWide, parser_FStoreWide);
    build_it_all!(@SIMP       b"\x43", FStore0, parser_FStore0);
    build_it_all!(@SIMP       b"\x44", FStore1, parser_FStore1);
    build_it_all!(@SIMP       b"\x45", FStore2, parser_FStore2);
    build_it_all!(@SIMP       b"\x46", FStore3, parser_FStore3);
    build_it_all!(@SIMP       b"\x66", FSub, parser_FSub);
    build_it_all!(@U16        b"\xB4", GetField, parser_GetField);
    build_it_all!(@U16        b"\xB2", GetStatic, parser_GetStatic);
    build_it_all!(@U16        b"\xA7", Goto, parser_Goto);
    build_it_all!(@U32        b"\xC8", GotoW, parser_GotoW);
    build_it_all!(@SIMP       b"\x91", I2B, parser_I2B);
    build_it_all!(@SIMP       b"\x92", I2C, parser_I2C);
    build_it_all!(@SIMP       b"\x87", I2D, parser_I2D);
    build_it_all!(@SIMP       b"\x86", I2F, parser_I2F);
    build_it_all!(@SIMP       b"\x85", I2L, parser_I2L);
    build_it_all!(@SIMP       b"\x93", I2S, parser_I2S);
    build_it_all!(@SIMP       b"\x60", IAdd, parser_IAdd);
    build_it_all!(@SIMP       b"\x2E", IALoad, parser_IALoad);
    build_it_all!(@SIMP       b"\x7E", IAnd, parser_IAnd);
    build_it_all!(@SIMP       b"\x4F", IAStore, parser_IAStore);
    build_it_all!(@SIMP       b"\x02", IConstM1, parser_IConstM1);
    build_it_all!(@SIMP       b"\x03", IConst0, parser_IConst0);
    build_it_all!(@SIMP       b"\x04", IConst1, parser_IConst1);
    build_it_all!(@SIMP       b"\x05", IConst2, parser_IConst2);
    build_it_all!(@SIMP       b"\x06", IConst3, parser_IConst3);
    build_it_all!(@SIMP       b"\x07", IConst4, parser_IConst4);
    build_it_all!(@SIMP       b"\x08", IConst5, parser_IConst5);
    build_it_all!(@SIMP       b"\x6C", IDiv, parser_IDiv);
    build_it_all!(@U16        b"\xA5", IFAcmpEQ, parser_IFAcmpEQ);
    build_it_all!(@U16        b"\xA6", IFAcmpNE, parser_IFAcmpNE);
    build_it_all!(@U16        b"\x9F", IFIcmpEQ, parser_IFIcmpEQ);
    build_it_all!(@U16        b"\xA2", IfIcmpGE, parser_IfIcmpGE);
    build_it_all!(@U16        b"\xA3", IFIcmpGT, parser_IFIcmpGT);
    build_it_all!(@U16        b"\xA4", IFIcmpLE, parser_IFIcmpLE);
    build_it_all!(@U16        b"\xA1", IFIcmpLT, parser_IFIcmpLT);
    build_it_all!(@U16        b"\xA0", IFIcmpNE, parser_IFIcmpNE);
    build_it_all!(@U16        b"\x99", IFEQ, parser_IFEQ);
    build_it_all!(@U16        b"\x9C", IFGE, parser_IFGE);
    build_it_all!(@U16        b"\x9D", IFGT, parser_IFGT);
    build_it_all!(@U16        b"\x9B", IFLE, parser_IFLE);
    build_it_all!(@U16        b"\x9A", IFNE, parser_IFNE);
    build_it_all!(@U16        b"\xC7", IFnonNull, parser_IFnonNull);
    build_it_all!(@U16        b"\xC6", IFNull, parser_IFNull);
    build_it_all!(@@U8        b"\x84", IInc, parser_IInc);
    build_it_all!(@IINC       b"\x84", IIncWide, parser_IIncWide);
    build_it_all!(@U8WIDE     b"\x15", ILoad, parser_ILoad);
    build_it_all!(@U8         b"\x15", ILoadWide, parser_ILoadWide);
    build_it_all!(@SIMP       b"\x1A", ILoad0, parser_ILoad0);
    build_it_all!(@SIMP       b"\x1B", ILoad1, parser_ILoad1);
    build_it_all!(@SIMP       b"\x1C", ILoad2, parser_ILoad2);
    build_it_all!(@SIMP       b"\x1D", ILoad3, parser_ILoad3);
    build_it_all!(@SIMP       b"\xF3", ImpDep1, parser_ImpDep1);
    build_it_all!(@SIMP       b"\xFF", ImpDep2, parser_ImpDep2);
    build_it_all!(@SIMP       b"\x68", IMul, parser_IMul);
    build_it_all!(@SIMP       b"\x74", INeg, parser_INeg);
    build_it_all!(@U16        b"\xC1", InstanceOf, parser_InstanceOf);
    build_it_all!(@U1600      b"\xBA", InvokedDynamic, parser_InvokedDynamic);
    build_it_all!(@@U16U80    b"\xB9", InvokedInterface, parser_InvokedInterface);
    build_it_all!(@U16        b"\xB7", InvokeSpecial, parser_InvokeSpecial);
    build_it_all!(@U16        b"\xB8", InvokeStatic, parser_InvokeStatic);
    build_it_all!(@U16        b"\xB6", InvokeVirtual, parser_InvokeVirtual);
    build_it_all!(@SIMP       b"\x80", IOr, parser_IOr);
    build_it_all!(@SIMP       b"\x70", IRem, parser_IRem);
    build_it_all!(@SIMP       b"\xAC", IReturn, parser_IReturn);
    build_it_all!(@SIMP       b"\x78", ISHL, parser_ISHL);
    build_it_all!(@SIMP       b"\x7A", ISHR, parser_ISHR);
    build_it_all!(@U8         b"\x36", IStore, parser_IStore);
    build_it_all!(@U8WIDE     b"\x36", IStoreWide, parser_IStoreWide);
    build_it_all!(@SIMP       b"\x3B", IStore0, parser_IStore0);
    build_it_all!(@SIMP       b"\x3C", IStore1, parser_IStore1);
    build_it_all!(@SIMP       b"\x3D", IStore2, parser_IStore2);
    build_it_all!(@SIMP       b"\x3E", IStore3, parser_IStore3);
    build_it_all!(@SIMP       b"\x64", ISub, parser_ISub);
    build_it_all!(@SIMP       b"\x7C", IUSHR, parser_IUSHR);
    build_it_all!(@SIMP       b"\x82", IXor, parser_IXor);
    build_it_all!(@U16        b"\xA8", JSR, parser_JSR);
    build_it_all!(@U16        b"\xC9", JSRW, parser_JSRW);
    build_it_all!(@SIMP       b"\x8A", L2D, parser_L2D);
    build_it_all!(@SIMP       b"\x89", L2F, parser_L2F);
    build_it_all!(@SIMP       b"\x88", L2I, parser_L2I);
    build_it_all!(@SIMP       b"\x61", LAdd, parser_LAdd);
    build_it_all!(@SIMP       b"\x2F", LALoad, parser_LALoad);
    build_it_all!(@SIMP       b"\x7F", LAnd, parser_LAnd);
    build_it_all!(@SIMP       b"\x50", LAStore, parser_LAStore);
    build_it_all!(@SIMP       b"\x94", Lcmp, parser_Lcmp);
    build_it_all!(@SIMP       b"\x09", LConst0, parser_LConst0);
    build_it_all!(@SIMP       b"\x0A", LConst1, parser_LConst1);
    build_it_all!(@U8         b"\x12", LDC, parser_LDC);
    build_it_all!(@U16        b"\x13", LDCW, parser_LDCW);
    build_it_all!(@U16        b"\x14", LDC2W, parser_LDC2W);
    build_it_all!(@SIMP       b"\x6D", LDiv, parser_LDiv);
    build_it_all!(@U8         b"\x16", LLoad, parser_LLoad);
    build_it_all!(@U8WIDE     b"\x16", LLoadWide, parser_LLoadWIDE);
    build_it_all!(@SIMP       b"\x1E", LLoad0, parser_LLoad0);
    build_it_all!(@SIMP       b"\x1F", LLoad1, parser_LLoad1);
    build_it_all!(@SIMP       b"\x20", LLoad2, parser_LLoad2);
    build_it_all!(@SIMP       b"\x21", LLoad3, parser_LLoad3);
    build_it_all!(@SIMP       b"\x69", LMul, parser_LMul);
    build_it_all!(@SIMP       b"\x75", LNeg, parser_LNeg);
    build_it_all!(@NPAIRS     b"\xAB", LookUpSwitch, parser_LookUpSwitch);
    build_it_all!(@SIMP       b"\x81", LOr, parser_LOr);
    build_it_all!(@SIMP       b"\x71", LRem, parser_LRem);
    build_it_all!(@SIMP       b"\xAD", LReturn, parser_LReturn);
    build_it_all!(@SIMP       b"\x79", LSHL, parser_LSHL);
    build_it_all!(@SIMP       b"\x7B", LSHR, parser_LSHR);
    build_it_all!(@U8         b"\x37", LStore, parser_LStore);
    build_it_all!(@U8WIDE     b"\x37", LStoreWide, parser_LStoreWide);
    build_it_all!(@SIMP       b"\x3F", LStore0, parser_LStore0);
    build_it_all!(@SIMP       b"\x40", LStore1, parser_LStore1);
    build_it_all!(@SIMP       b"\x41", LStore2, parser_LStore2);
    build_it_all!(@SIMP       b"\x42", LStore3, parser_LStore3);
    build_it_all!(@SIMP       b"\x65", LSub, parser_LSub);
    build_it_all!(@SIMP       b"\x7D", LUSHR, parser_LUSHR);
    build_it_all!(@SIMP       b"\x83", LXor, parser_LXor);
    build_it_all!(@SIMP       b"\xC2", MonitorEnter, parser_MonitorEnter);
    build_it_all!(@SIMP       b"\xC3", MonitorExit, parser_MonitorExit);
    build_it_all!(@@U16U8     b"\xC5", MultiAneWArray, parser_MultiAneWArray);
    build_it_all!(@U16        b"\xBB", New, parser_New);
    build_it_all!(@U8         b"\xBC", NewArray, parser_NewArray);
    build_it_all!(@SIMP       b"\x00", Nop, parser_Nop);
    build_it_all!(@SIMP       b"\x57", Pop, parser_Pop);
    build_it_all!(@SIMP       b"\x58", Pop2, parser_Pop2);
    build_it_all!(@U16        b"\xB5", PutField, parser_PutField);
    build_it_all!(@U16        b"\xB3", PutStatic, parser_PutStatic);
    build_it_all!(@U8         b"\xA9", Ret, parser_Ret);
    build_it_all!(@U8WIDE     b"\xA9", RetWide, parser_RetWide);
    build_it_all!(@SIMP       b"\xB1", Return, parser_Return);
    build_it_all!(@SIMP       b"\x35", SALoad, parser_SALoad);
    build_it_all!(@SIMP       b"\x56", SAStore, parser_SAStore);
    build_it_all!(@U16        b"\x11", SIPush, parser_SIPush);
    build_it_all!(@SIMP       b"\x5F", Swap, parser_Swap);
    build_it_all!(@TABLESWITCH b"\xAA", TableSwitch, parser_TableSwitch);

/// This is a full list of all Java Enums
#[derive(Clone,Debug)]
pub enum JOp {
    AALoad,
    AAStore,
    AConstNull,
    ALoad(u8),
    ALoadWide(u16),
    ALoad0,
    ALoad1,
    ALoad2,
    ALoad3,
    ANewArray(u16),
    AReturn,
    ArrayLength,
    AStore(u8),
    AStoreWide(u16),
    AStore0,
    AStore1,
    AStore2,
    AStore3,
    AThrow,
    BaLoad,
    BaStore,
    BiPush(u8),
    BreakPoint,
    CaLoad,
    CaStore,
    CheckCast(u16),
    D2F,
    D2I,
    D2L,
    DAdd,
    DALoad,
    DAStore,
    DcmpG,
    DcmpL,
    DConst0,
    DConst1,
    DDiv,
    DLoad(u8),
    DLoadWide(u16),
    DLoad0,
    DLoad1,
    DLoad2,
    DLoad3,
    DMul,
    DNeg,
    DRem,
    DStore(u8),
    DStoreWide(u16),
    DStore0,
    DStore1,
    DStore2,
    DStore3,
    DSub,
    Dup,
    Dupx1,
    Dupx2,
    Dup2,
    Dup2x1,
    Dup2x2,
    F2D,
    F2I,
    F2L,
    FAdd,
    FALoad,
    FAStore,
    FcmpG,
    FcmpL,
    FConst0,
    FConst1,
    FConst2,
    FDiv,
    FLoad(u8),
    FLoadWide(u16),
    FLoad0,
    FLoad1,
    FLoad2,
    FLoad3,
    FMul,
    FNeg,
    FRem,
    FReturn,
    FStore(u8),
    FStoreWide(u16),
    FStore0,
    FStore1,
    FStore2,
    FStore3,
    FSub,
    GetField(u16),
    GetStatic(u16),
    Goto(u16),
    GotoW(u32),
    I2B,
    I2C,
    I2D,
    I2F,
    I2L,
    I2S,
    IAdd,
    IALoad,
    IAnd,
    IAStore,
    IConstM1,
    IConst0,
    IConst1,
    IConst2,
    IConst3,
    IConst4,
    IConst5,
    IDiv,
    IFAcmpEQ(u16),
    IFAcmpNE(u16),
    IFIcmpEQ(u16),
    IfIcmpGE(u16),
    IFIcmpGT(u16),
    IFIcmpLE(u16),
    IFIcmpLT(u16),
    IFIcmpNE(u16),
    IFEQ(u16),
    IFGE(u16),
    IFGT(u16),
    IFLE(u16),
    IFNE(u16),
    IFnonNull(u16),
    IFNull(u16),
    IInc(u8, u8),
    IIncWide(u16,i16),
    ILoad(u16),
    ILoadWide(u8),
    ILoad0,
    ILoad1,
    ILoad2,
    ILoad3,
    ImpDep1,
    ImpDep2,
    IMul,
    INeg,
    InstanceOf(u16),
    InvokedDynamic(u16),
    InvokedInterface(u16, u8),
    InvokeSpecial(u16),
    InvokeStatic(u16),
    InvokeVirtual(u16),
    IOr,
    IRem,
    IReturn,
    ISHL,
    ISHR,
    IStore(u8),
    IStoreWide(u16),
    IStore0,
    IStore1,
    IStore2,
    IStore3,
    ISub,
    IUSHR,
    IXor,
    JSR(u16),
    JSRW(u16),
    L2D,
    L2F,
    L2I,
    LAdd,
    LALoad,
    LAnd,
    LAStore,
    Lcmp,
    LConst0,
    LConst1,
    LDC(u8),
    LDCW(u16),
    LDC2W(u16),
    LDiv,
    LLoad(u8),
    LLoadWide(u16),
    LLoad0,
    LLoad1,
    LLoad2,
    LLoad3,
    LMul,
    LNeg,
    LookUpSwitch(u32, Vec<(i32,u32)>),
    LOr,
    LRem,
    LReturn,
    LSHL,
    LSHR,
    LStore(u8),
    LStoreWide(u16),
    LStore0,
    LStore1,
    LStore2,
    LStore3,
    LSub,
    LUSHR,
    LXor,
    MonitorEnter,
    MonitorExit,
    MultiAneWArray(u16, u8),
    New(u16),
    NewArray(u8),
    Nop,
    Pop,
    Pop2,
    PutField(u16),
    PutStatic(u16),
    Ret(u8),
    RetWide(u16),
    Return,
    SALoad,
    SAStore,
    SIPush(u16),
    Swap,
    TableSwitch(i32, i32, i32, Vec<u32>)
}

/// Parse a single JOP
pub fn parse_java_op<'a>(buffer: &'a [u8]) -> IResult<&'a [u8], JOp> {
    if buffer.len() == 0 {
        return IResult::Incomplete(Needed::Unknown);
    }
    match parser_AALoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_AAStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_AConstNull(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ALoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ALoadWide(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ALoad0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ALoad1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ALoad2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ALoad3(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ANewArray(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_AReturn(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ArrayLength(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_AStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_AStoreWide(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_AStore0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_AStore1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_AStore2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_AStore3(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_AThrow(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_BaLoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_BaStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_BiPush(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_BreakPoint(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_CaLoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_CaStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_CheckCast(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_D2F(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_D2I(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_D2L(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DAdd(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DALoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DAStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DcmpG(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DcmpL(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DConst0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DConst1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DDiv(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DLoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DLoadWide(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DLoad0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DLoad1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DLoad2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DLoad3(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DMul(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DNeg(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DRem(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DStoreWide(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DStore0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DStore1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DStore2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DStore3(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_DSub(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Dup(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Dupx1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Dupx2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Dup2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Dup2x1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Dup2x2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_F2D(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_F2I(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_F2L(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FAdd(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FALoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FAStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FcmpG(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FcmpL(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FConst0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FConst1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FConst2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FDiv(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FLoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FLoadWide(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FLoad0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FLoad1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FLoad2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FLoad3(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FMul(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FNeg(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FRem(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FReturn(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FStoreWide(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FStore0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FStore1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FStore2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FStore3(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_FSub(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_GetField(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_GetStatic(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Goto(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_GotoW(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_I2B(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_I2C(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_I2D(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_I2F(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_I2L(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_I2S(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IAdd(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IALoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IAnd(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IAStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IConstM1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IConst0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IConst1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IConst2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IConst3(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IConst4(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IConst5(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IDiv(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFAcmpEQ(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFAcmpNE(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFIcmpEQ(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IfIcmpGE(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFIcmpGT(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFIcmpLE(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFIcmpLT(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFIcmpNE(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFEQ(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFGE(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFGT(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFLE(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFNE(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFnonNull(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IFNull(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IInc(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IIncWide(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ILoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ILoadWide(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ILoad0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ILoad1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ILoad2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ILoad3(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ImpDep1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ImpDep2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IMul(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_INeg(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_InstanceOf(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_InvokedDynamic(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_InvokedInterface(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_InvokeSpecial(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_InvokeStatic(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_InvokeVirtual(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IOr(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IRem(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IReturn(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ISHL(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ISHR(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IStoreWide(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IStore0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IStore1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IStore2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IStore3(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_ISub(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IUSHR(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_IXor(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_JSR(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_JSRW(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_L2D(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_L2F(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_L2I(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LAdd(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LALoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LAnd(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LAStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Lcmp(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LConst0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LConst1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LDC(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LDCW(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LDC2W(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LDiv(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LLoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LLoadWIDE(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LLoad0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LLoad1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LLoad2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LLoad3(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LMul(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LNeg(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LookUpSwitch(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LOr(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LRem(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LReturn(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LSHL(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LSHR(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LStoreWide(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LStore0(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LStore1(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LStore2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LStore3(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LSub(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LUSHR(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_LXor(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_MonitorEnter(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_MonitorExit(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_MultiAneWArray(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_New(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_NewArray(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Nop(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Pop(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Pop2(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_PutField(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_PutStatic(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Ret(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_RetWide(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Return(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_SALoad(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_SAStore(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_SIPush(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_Swap(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    match parser_TableSwitch(buffer) {
        IResult::Done(x,y) => return IResult::Done(x,y),
        _ => { }
    };
    IResult::Error(ErrorKind::Custom(1000u32))
}



