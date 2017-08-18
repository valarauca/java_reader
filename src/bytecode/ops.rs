
use super::super::nom::{le_u8, le_u32, le_u16, le_i32, IResult, ErrorKind, le_i16};

use super::tls_pos::{move_pos, reset_align_code, align_code};

use std::sync::atomic::{AtomicUsize, Ordering};

named!(parse_npairs<(i32,u32)>, do_parse!(
    match_var: le_i32 >>
    offset: le_u32 >>
    ( (match_var, offset) )
));

fn slen(high: i32, low: i32) -> usize {
    ((high - low) + 1) as usize
}

macro_rules! build_it_all {
    (@PARSE @SIMP $a: expr, $b: ident, $c: ident; $($tail:tt)*) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            ({
                move_pos(1);
                JOp::$b
            })
        ));
        build_it_all!{@PARSE $($tail)*}
    };
    (@PARSE @U8 $a: expr, $b: ident, $c: ident; $($tail:tt)*) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var: le_u8 >>
            ({
                move_pos(2);
                JOp::$b(var)
            })
        ));
        build_it_all!{@PARSE $($tail)*}
    };
    (@PARSE @U8WIDE $a: expr, $b: ident, $c: ident; $($tail:tt)*) => {
        named!($c<JOp>, do_parse!(
            tag!(b"\xC4") >>
            tag!($a) >>
            var: le_u16 >>
            ({
                move_pos(4);
                JOp::$b(var)
            })
        ));
        build_it_all!{@PARSE $($tail)*}
    };
    (@PARSE @U16 $a: expr, $b: ident, $c: ident; $($tail:tt)*) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var: le_u16 >>
            ({
                move_pos(3);
                JOp::$b(var)
            })
        ));
        build_it_all!{@PARSE $($tail)*}
    };
    (@PARSE @U1600 $a: expr, $b: ident, $c: ident; $($tail:tt)*) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var: le_u16 >>
            tag!(b"\x00\x00") >>
            ({
                move_pos(5);
                JOp::$b(var)
            })
        ));
        build_it_all!{@PARSE $($tail)*}
    };
    (@PARSE @U32 $a: expr, $b: ident, $c: ident; $($tail:tt)*) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var: le_u32 >>
            ({
                move_pos(5);
                JOp::$b(var)
            })
        ));
        build_it_all!{@PARSE $($tail)*}
    };
    (@PARSE @NPAIRS $a: expr, $b: ident, $c: ident; $($tail:tt)*) => {
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
        build_it_all!{@PARSE $($tail)*}
    };
    (@PARSE @IINC $a: expr, $b: ident, $c: ident; $($tail:tt)*) => {
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
        build_it_all!{@PARSE $($tail)*}
    };
    (@PARSE @@U8 $a: expr, $b: ident, $c: ident; $($tail:tt)*) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var0: le_u8 >>
            var1: le_u8 >>
            ({
                move_pos(3);
                JOp::$b(var0, var1)
            })
        ));
        build_it_all!{@PARSE $($tail)*}
    };
    (@PARSE @@U16U80 $a: expr, $b: ident, $c: ident; $($tail:tt)*) => {
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
        build_it_all!{@PARSE $($tail)*}
    };
    (@PARSE @@U16U8 $a: expr, $b: ident, $c: ident; $($tail:tt)*) => {
        named!($c<JOp>, do_parse!(
            tag!($a) >>
            var0: le_u16 >>
            var1: le_u8 >>
            ({
                move_pos(4);
                JOp::$b(var0, var1)
            })
        ));
        build_it_all!{@PARSE $($tail)*}
    };

    //exit condition
    (@PARSE @TABLESWITCH $a: expr, $b: ident, $c: ident;) => {
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

    //Actual entry point
    ($($tail:tt)*) => {
        build_it_all!{@PARSE $($tail)*}
    };
}


build_it_all! {
    @SIMP       b"\x32", AALoad, parser_AALoad;
    @SIMP       b"\x53", AAStore, parser_AAStore;
    @SIMP       b"\x01", AConstNull, parser_AConstNull;
    @U8         b"\x19", ALoad, parser_ALoad;
    @U8WIDE     b"\x19", ALoadWide, parser_ALoadWide;
    @SIMP       b"\x2A", ALoad0, parser_ALoad0;
    @SIMP       b"\x2B", ALoad1, parser_ALoad1;
    @SIMP       b"\x2C", ALoad2, parser_ALoad2;
    @SIMP       b"\x2D", ALoad3, parser_ALoad3;
    @U16        b"\xBD", ANewArray, parser_ANewArray;
    @SIMP       b"\xB0", AReturn, parser_AReturn;
    @SIMP       b"\xBE", ArrayLength, parser_ArrayLength;
    @U8         b"\x3A", AStore, parser_AStore;
    @U8WIDE     b"\x3A", AStoreWide, parser_AStoreWide;
    @SIMP       b"\x4B", AStore0, parser_AStore0;
    @SIMP       b"\x4C", AStore1, parser_AStore1;
    @SIMP       b"\x4D", AStore2, parser_AStore2;
    @SIMP       b"\x4E", AStore3, parser_AStore3;
    @SIMP       b"\xBF", AThrow, parser_AThrow;
    @SIMP       b"\x33", BaLoad, parser_BaLoad;
    @SIMP       b"\x54", BaStore, parser_BaStore;
    @U8         b"\x10", BiPush, parser_BiPush;
    @SIMP       b"\xCA", BreakPoint, parser_BreakPoint;
    @SIMP       b"\x34", CaLoad, parser_CaLoad;
    @SIMP       b"\x55", CaStore, parser_CaStore;
    @U16        b"\xC0", CheckCast, parser_CheckCast;
    @SIMP       b"\x90", D2F, parser_D2F;
    @SIMP       b"\x8E", D2I, parser_D2I;
    @SIMP       b"\x8F", D2L, parser_D2L;
    @SIMP       b"\x63", DAdd, parser_DAdd;
    @SIMP       b"\x31", DALoad, parser_DALoad;
    @SIMP       b"\x52", DAStore, parser_DAStore;
    @SIMP       b"\x98", DcmpG, parser_DcmpG;
    @SIMP       b"\x97", DcmpL, parser_DcmpL;
    @SIMP       b"\x0E", DConst0, parser_DConst0;
    @SIMP       b"\x0F", DConst1, parser_DConst1;
    @SIMP       b"\x6F", DDiv, parser_DDiv;
    @U8         b"\x18", DLoad, parser_DLoad;
    @U8WIDE     b"\x18", DLoadWide, parser_DLoadWide;
    @SIMP       b"\x26", DLoad0, parser_DLoad0;
    @SIMP       b"\x27", DLoad1, parser_DLoad1;
    @SIMP       b"\x28", DLoad2, parser_DLoad2;
    @SIMP       b"\x29", DLoad3, parser_DLoad3;
    @SIMP       b"\x6B", DMul, parser_DMul;
    @SIMP       b"\x77", DNeg, parser_DNeg;
    @SIMP       b"\x73", DRem, parser_DRem;
    @U8         b"\xAF", DStore, parser_DStore;
    @U8WIDE     b"\xAF", DStoreWide, parser_DStoreWide;
    @SIMP       b"\x47", DStore0, parser_DStore0;
    @SIMP       b"\x48", DStore1, parser_DStore1;
    @SIMP       b"\x49", DStore2, parser_DStore2;
    @SIMP       b"\x4A", DStore3, parser_DStore3;
    @SIMP       b"\x67", DSub, parser_DSub;
    @SIMP       b"\x59", Dup, parser_Dup;
    @SIMP       b"\x5A", Dupx1, parser_Dupx1;
    @SIMP       b"\x5B", Dupx2, parser_Dupx2;
    @SIMP       b"\x5C", Dup2, parser_Dup2;
    @SIMP       b"\x5D", Dup2x1, parser_Dup2x1;
    @SIMP       b"\x5C", Dup2x2, parser_Dup2x2;
    @SIMP       b"\x8D", F2D, parser_F2D;
    @SIMP       b"\x8B", F2I, parser_F2I;
    @SIMP       b"\x8C", F2L, parser_F2L;
    @SIMP       b"\x62", FAdd, parser_FAdd;
    @SIMP       b"\x30", FALoad, parser_FALoad;
    @SIMP       b"\x51", FAStore, parser_FAStore;
    @SIMP       b"\x96", FcmpG, parser_FcmpG;
    @SIMP       b"\x95", FcmpL, parser_FcmpL;
    @SIMP       b"\x0B", FConst0, parser_FConst0;
    @SIMP       b"\x0C", FConst1, parser_FConst1;
    @SIMP       b"\x0D", FConst2, parser_FConst2;
    @SIMP       b"\x6E", FDiv, parser_FDiv;
    @U8         b"\x17", FLoad, parser_FLoad;
    @U8WIDE     b"\x17", FLoadWide, parser_FLoadWide;
    @SIMP       b"\x22", FLoad0, parser_FLoad0;
    @SIMP       b"\x23", FLoad1, parser_FLoad1;
    @SIMP       b"\x24", FLoad2, parser_FLoad2;
    @SIMP       b"\x25", FLoad3, parser_FLoad3;
    @SIMP       b"\x6A", FMul, parser_FMul;
    @SIMP       b"\x76", FNeg, parser_FNeg;
    @SIMP       b"\x72", FRem, parser_FRem;
    @SIMP       b"\xAE", FReturn, parser_FReturn;
    @U8         b"\x38", FStore, parser_FStore;
    @U8WIDE     b"\x38", FStoreWide, parser_FStoreWide;
    @SIMP       b"\x43", FStore0, parser_FStore0;
    @SIMP       b"\x44", FStore1, parser_FStore1;
    @SIMP       b"\x45", FStore2, parser_FStore2;
    @SIMP       b"\x46", FStore3, parser_FStore3;
    @SIMP       b"\x66", FSub, parser_FSub;
    @U16        b"\xB4", GetField, parser_GetField;
    @U16        b"\xB2", GetStatic, parser_GetStatic;
    @U16        b"\xA7", Goto, parser_Goto;
    @U32        b"\xC8", GotoW, parser_GotoW;
    @SIMP       b"\x91", I2B, parser_I2B;
    @SIMP       b"\x92", I2C, parser_I2C;
    @SIMP       b"\x87", I2D, parser_I2D;
    @SIMP       b"\x86", I2F, parser_I2F;
    @SIMP       b"\x85", I2L, parser_I2L;
    @SIMP       b"\x93", I2S, parser_I2S;
    @SIMP       b"\x60", IAdd, parser_IAdd;
    @SIMP       b"\x2E", IALoad, parser_IALoad;
    @SIMP       b"\x7E", IAnd, parser_IAnd;
    @SIMP       b"\x4F", IAStore, parser_IAStore;
    @SIMP       b"\x02", IConstM1, parser_IConstM1;
    @SIMP       b"\x03", IConst0, parser_IConst0;
    @SIMP       b"\x04", IConst1, parser_IConst1;
    @SIMP       b"\x05", IConst2, parser_IConst2;
    @SIMP       b"\x06", IConst3, parser_IConst3;
    @SIMP       b"\x07", IConst4, parser_IConst4;
    @SIMP       b"\x08", IConst5, parser_IConst5;
    @SIMP       b"\x6C", IDiv, parser_IDiv;
    @U16        b"\xA5", IFAcmpEQ, parser_IFAcmpEQ;
    @U16        b"\xA6", IFAcmpNE, parser_IFAcmpNE;
    @U16        b"\x9F", IFIcmpEQ, parser_IFIcmpEQ;
    @U16        b"\xA2", IfIcmpGE, parser_IfIcmpGE;
    @U16        b"\xA3", IFIcmpGT, parser_IFIcmpGT;
    @U16        b"\xA4", IFIcmpLE, parser_IFIcmpLE;
    @U16        b"\xA1", IFIcmpLT, parser_IFIcmpLT;
    @U16        b"\xA0", IFIcmpNE, parser_IFIcmpNE;
    @U16        b"\x99", IFEQ, parser_IFEQ;
    @U16        b"\x9C", IFGE, parser_IFGE;
    @U16        b"\x9D", IFGT, parser_IFGT;
    @U16        b"\x9B", IFLE, parser_IFLE;
    @U16        b"\x9A", IFNE, parser_IFNE;
    @U16        b"\xC7", IFnonNull, parser_IFnonNull;
    @U16        b"\xC6", IFNull, parser_IFNull;
    @@U8        b"\x84", IInc, parser_IInc;
    @IINC       b"\x84", IIncWide, parser_IIncWide;
    @U8WIDE     b"\x15", ILoad, parser_ILoad;
    @U8         b"\x15", ILoadWide, parser_ILoadWide;
    @SIMP       b"\x1A", ILoad0, parser_ILoad0;
    @SIMP       b"\x1B", ILoad1, parser_ILoad1;
    @SIMP       b"\x1C", ILoad2, parser_ILoad2;
    @SIMP       b"\x1D", ILoad3, parser_ILoad3;
    @SIMP       b"\xF3", ImpDep1, parser_ImpDep1;
    @SIMP       b"\xFF", ImpDep2, parser_ImpDep2;
    @SIMP       b"\x68", IMul, parser_IMul;
    @SIMP       b"\x74", INeg, parser_INeg;
    @U16        b"\xC1", InstanceOf, parser_InstanceOf;
    @U1600      b"\xBA", InvokedDynamic, parser_InvokedDynamic;
    @@U16U80    b"\xB9", InvokedInterface, parser_InvokedInterface;
    @U16        b"\xB7", InvokeSpecial, parser_InvokeSpecial;
    @U16        b"\xB8", InvokeStatic, parser_InvokeStatic;
    @U16        b"\xB6", InvokeVirtual, parser_InvokeVirtual;
    @SIMP       b"\x80", IOr, parser_IOr;
    @SIMP       b"\x70", IRem, parser_IRem;
    @SIMP       b"\xAC", IReturn, parser_IReturn;
    @SIMP       b"\x78", ISHL, parser_ISHL;
    @SIMP       b"\x7A", ISHR, parser_ISHR;
    @U8         b"\x36", IStore, parser_IStore;
    @U8WIDE     b"\x36", IStoreWide, parser_IStoreWide;
    @SIMP       b"\x3B", IStore0, parser_IStore0;
    @SIMP       b"\x3C", IStore1, parser_IStore1;
    @SIMP       b"\x3D", IStore2, parser_IStore2;
    @SIMP       b"\x3E", IStore3, parser_IStore3;
    @SIMP       b"\x64", ISub, parser_ISub;
    @SIMP       b"\x7C", IUSHR, parser_IUSHR;
    @SIMP       b"\x82", IXor, parser_IXor;
    @U16        b"\xA8", JSR, parser_JSR;
    @U16        b"\xC9", JSRW, parser_JSRW;
    @SIMP       b"\x8A", L2D, parser_L2D;
    @SIMP       b"\x89", L2F, parser_L2F;
    @SIMP       b"\x88", L2I, parser_L2I;
    @SIMP       b"\x61", LAdd, parser_LAdd;
    @SIMP       b"\x2F", LALoad, parser_LALoad;
    @SIMP       b"\x7F", LAnd, parser_LAnd;
    @SIMP       b"\x50", LAStore, parser_LAStore;
    @SIMP       b"\x94", Lcmp, parser_Lcmp;
    @SIMP       b"\x09", LConst0, parser_LConst0;
    @SIMP       b"\x0A", LConst1, parser_LConst1;
    @U8         b"\x12", LDC, parser_LDC;
    @U16        b"\x13", LDCW, parser_LDCW;
    @U16        b"\x14", LDC2W, parser_LDC2W;
    @SIMP       b"\x6D", LDiv, parser_LDiv;
    @U8         b"\x16", LLoad, parser_LLoad;
    @U8WIDE     b"\x16", LLoadWide, parser_LLoadWIDE;
    @SIMP       b"\x1E", LLoad0, parser_LLoad0;
    @SIMP       b"\x1F", LLoad1, parser_LLoad1;
    @SIMP       b"\x20", LLoad2, parser_LLoad2;
    @SIMP       b"\x21", LLoad3, parser_LLoad3;
    @SIMP       b"\x69", LMul, parser_LMul;
    @SIMP       b"\x75", LNeg, parser_LNeg;
    @NPAIRS     b"\xAB", LookUpSwitch, parser_LookUpSwitch;
    @SIMP       b"\x81", LOr, parser_LOr;
    @SIMP       b"\x71", LRem, parser_LRem;
    @SIMP       b"\xAD", LReturn, parser_LReturn;
    @SIMP       b"\x79", LSHL, parser_LSHL;
    @SIMP       b"\x7B", LSHR, parser_LSHR;
    @U8         b"\x37", LStore, parser_LStore;
    @U8WIDE     b"\x37", LStoreWide, parser_LStoreWide;
    @SIMP       b"\x3F", LStore0, parser_LStore0;
    @SIMP       b"\x40", LStore1, parser_LStore1;
    @SIMP       b"\x41", LStore2, parser_LStore2;
    @SIMP       b"\x42", LStore3, parser_LStore3;
    @SIMP       b"\x65", LSub, parser_LSub;
    @SIMP       b"\x7D", LUSHR, parser_LUSHR;
    @SIMP       b"\x83", LXor, parser_LXor;
    @SIMP       b"\xC2", MonitorEnter, parser_MonitorEnter;
    @SIMP       b"\xC3", MonitorExit, parser_MonitorExit;
    @@U16U8     b"\xC5", MultiAneWArray, parser_MultiAneWArray;
    @U16        b"\xBB", New, parser_New;
    @U8         b"\xBC", NewArray, parser_NewArray;
    @SIMP       b"\x00", Nop, parser_Nop;
    @SIMP       b"\x57", Pop, parser_Pop;
    @SIMP       b"\x58", Pop2, parser_Pop2;
    @U16        b"\xB5", PutField, parser_PutField;
    @U16        b"\xB3", PutStatic, parser_PutStatic;
    @U8         b"\xA9", Ret, parser_Ret;
    @U8WIDE     b"\xA9", RetWide, parser_RetWide;
    @SIMP       b"\xB1", Return, parser_Return;
    @SIMP       b"\x35", SALoad, parser_SALoad;
    @SIMP       b"\x56", SAStore, parser_SAStore;
    @U16        b"\x11", SIPush, parser_SIPush;
    @SIMP       b"\x5F", Swap, parser_Swap;
    @TABLESWITCH b"\xAA", TableSwitch, parser_TableSwitch;
}

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

named!(pub parse_java_op<JOp>, alt_complete!(
  parser_AALoad|
  parser_AAStore|
  parser_AConstNull|
  parser_ALoad|
  parser_ALoadWide|
  parser_ALoad0|
  parser_ALoad1|
  parser_ALoad2|
  parser_ALoad3|
  parser_ANewArray|
  parser_AReturn|
  parser_ArrayLength|
  parser_AStore|
  parser_AStoreWide|
  parser_AStore0|
  parser_AStore1|
  parser_AStore2|
  parser_AStore3|
  parser_AThrow|
  parser_BaLoad|
  parser_BaStore|
  parser_BiPush|
  parser_BreakPoint|
  parser_CaLoad|
  parser_CaStore|
  parser_CheckCast|
  parser_D2F|
  parser_D2I|
  parser_D2L|
  parser_DAdd|
  parser_DALoad|
  parser_DAStore|
  parser_DcmpG|
  parser_DcmpL|
  parser_DConst0|
  parser_DConst1|
  parser_DDiv|
  parser_DLoad|
  parser_DLoadWide|
  parser_DLoad0|
  parser_DLoad1|
  parser_DLoad2|
  parser_DLoad3|
  parser_DMul|
  parser_DNeg|
  parser_DRem|
  parser_DStore|
  parser_DStoreWide|
  parser_DStore0|
  parser_DStore1|
  parser_DStore2|
  parser_DStore3|
  parser_DSub|
  parser_Dup|
  parser_Dupx1|
  parser_Dupx2|
  parser_Dup2|
  parser_Dup2x1|
  parser_Dup2x2|
  parser_F2D|
  parser_F2I|
  parser_F2L|
  parser_FAdd|
  parser_FALoad|
  parser_FAStore|
  parser_FcmpG|
  parser_FcmpL|
  parser_FConst0|
  parser_FConst1|
  parser_FConst2|
  parser_FDiv|
  parser_FLoad|
  parser_FLoadWide|
  parser_FLoad0|
  parser_FLoad1|
  parser_FLoad2|
  parser_FLoad3|
  parser_FMul|
  parser_FNeg|
  parser_FRem|
  parser_FReturn|
  parser_FStore|
  parser_FStoreWide|
  parser_FStore0|
  parser_FStore1|
  parser_FStore2|
  parser_FStore3|
  parser_FSub|
  parser_GetField|
  parser_GetStatic|
  parser_Goto|
  parser_GotoW|
  parser_I2B|
  parser_I2C|
  parser_I2D|
  parser_I2F|
  parser_I2L|
  parser_I2S|
  parser_IAdd|
  parser_IALoad|
  parser_IAnd|
  parser_IAStore|
  parser_IConstM1|
  parser_IConst0|
  parser_IConst1|
  parser_IConst2|
  parser_IConst3|
  parser_IConst4|
  parser_IConst5|
  parser_IDiv|
  parser_IFAcmpEQ|
  parser_IFAcmpNE|
  parser_IFIcmpEQ|
  parser_IfIcmpGE|
  parser_IFIcmpGT|
  parser_IFIcmpLE|
  parser_IFIcmpLT|
  parser_IFIcmpNE|
  parser_IFEQ|
  parser_IFGE|
  parser_IFGT|
  parser_IFLE|
  parser_IFNE|
  parser_IFnonNull|
  parser_IFNull|
  parser_IInc|
  parser_IIncWide|
  parser_ILoad|
  parser_ILoadWide|
  parser_ILoad0|
  parser_ILoad1|
  parser_ILoad2|
  parser_ILoad3|
  parser_ImpDep1|
  parser_ImpDep2|
  parser_IMul|
  parser_INeg|
  parser_InstanceOf|
  parser_InvokedDynamic|
  parser_InvokedInterface|
  parser_InvokeSpecial|
  parser_InvokeStatic|
  parser_InvokeVirtual|
  parser_IOr|
  parser_IRem|
  parser_IReturn|
  parser_ISHL|
  parser_ISHR|
  parser_IStore|
  parser_IStoreWide|
  parser_IStore0|
  parser_IStore1|
  parser_IStore2|
  parser_IStore3|
  parser_ISub|
  parser_IUSHR|
  parser_IXor|
  parser_JSR|
  parser_JSRW|
  parser_L2D|
  parser_L2F|
  parser_L2I|
  parser_LAdd|
  parser_LALoad|
  parser_LAnd|
  parser_LAStore|
  parser_Lcmp|
  parser_LConst0|
  parser_LConst1|
  parser_LDC|
  parser_LDCW|
  parser_LDC2W|
  parser_LDiv|
  parser_LLoad|
  parser_LLoadWIDE|
  parser_LLoad0|
  parser_LLoad1|
  parser_LLoad2|
  parser_LLoad3|
  parser_LMul|
  parser_LNeg|
  parser_LookUpSwitch|
  parser_LOr|
  parser_LRem|
  parser_LReturn|
  parser_LSHL|
  parser_LSHR|
  parser_LStore|
  parser_LStoreWide|
  parser_LStore0|
  parser_LStore1|
  parser_LStore2|
  parser_LStore3|
  parser_LSub|
  parser_LUSHR|
  parser_LXor|
  parser_MonitorEnter|
  parser_MonitorExit|
  parser_MultiAneWArray|
  parser_New|
  parser_NewArray|
  parser_Nop|
  parser_Pop|
  parser_Pop2|
  parser_PutField|
  parser_PutStatic|
  parser_Ret|
  parser_RetWide|
  parser_Return|
  parser_SALoad|
  parser_SAStore|
  parser_SIPush|
  parser_Swap|
  parser_TableSwitch
));




