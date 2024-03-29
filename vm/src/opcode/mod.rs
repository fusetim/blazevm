use crate::class_manager::ClassManager;
use crate::thread::Thread;
use crate::{opcode_with_operand1, opcode_with_operand2};
use binrw::{BinRead, BinReaderExt};
use reader::base::ParsingError;
use snafu::Snafu;
use std::io::{Read, Seek};

mod comparison;
mod constant;
mod control;
mod conversion;
mod extended;
mod load;
mod math;
mod reference;
mod stack;
mod store;

#[derive(Debug, Clone)]
pub enum Opcode {
    Nop,
    AConstNull,
    IConstM1,
    IConst0,
    IConst1,
    IConst2,
    IConst3,
    IConst4,
    IConst5,
    LConst0,
    LConst1,
    FConst0,
    FConst1,
    FConst2,
    DConst0,
    DConst1,
    Bipush(i8),
    Sipush(i16),
    Ldc(u8),
    LdcW(u16),
    Ldc2W(u16),
    ILoad(u8),
    LLoad(u8),
    FLoad(u8),
    DLoad(u8),
    ALoad(u8),
    ILoad0,
    ILoad1,
    ILoad2,
    ILoad3,
    LLoad0,
    LLoad1,
    LLoad2,
    LLoad3,
    FLoad0,
    FLoad1,
    FLoad2,
    FLoad3,
    DLoad0,
    DLoad1,
    DLoad2,
    DLoad3,
    ALoad0,
    ALoad1,
    ALoad2,
    ALoad3,
    IALoad,
    LALoad,
    FALoad,
    DALoad,
    AALoad,
    BALoad,
    CALoad,
    SALoad,
    IStore(u8),
    LStore(u8),
    FStore(u8),
    DStore(u8),
    AStore(u8),
    IStore0,
    IStore1,
    IStore2,
    IStore3,
    LStore0,
    LStore1,
    LStore2,
    LStore3,
    FStore0,
    FStore1,
    FStore2,
    FStore3,
    DStore0,
    DStore1,
    DStore2,
    DStore3,
    AStore0,
    AStore1,
    AStore2,
    AStore3,
    IAStore,
    LAStore,
    FAStore,
    DAStore,
    AAStore,
    BAStore,
    CAStore,
    SAStore,
    Pop,
    Pop2,
    Dup,
    DupX1,
    DupX2,
    Dup2,
    Dup2X1,
    Dup2X2,
    Swap,
    IAdd,
    LAdd,
    FAdd,
    DAdd,
    ISub,
    LSub,
    FSub,
    DSub,
    IMul,
    LMul,
    FMul,
    DMul,
    IDiv,
    LDiv,
    FDiv,
    DDiv,
    IRem,
    LRem,
    FRem,
    DRem,
    INeg,
    LNeg,
    FNeg,
    DNeg,
    IShl,
    LShl,
    IShr,
    LShr,
    IUshr,
    LUshr,
    IAnd,
    LAnd,
    IOr,
    LOr,
    IXor,
    LXor,
    IInc(u8, i8),
    I2L,
    I2F,
    I2D,
    L2I,
    L2F,
    L2D,
    F2I,
    F2L,
    F2D,
    D2I,
    D2L,
    D2F,
    I2B,
    I2C,
    I2S,
    LCmp,
    FCmpL,
    FCmpG,
    DCmpL,
    DCmpG,
    IfEq(i16),
    IfNe(i16),
    IfLt(i16),
    IfGe(i16),
    IfGt(i16),
    IfLe(i16),
    IfICmpEq(i16),
    IfICmpNe(i16),
    IfICmpLt(i16),
    IfICmpGe(i16),
    IfICmpGt(i16),
    IfICmpLe(i16),
    IfACmpEq(i16),
    IfACmpNe(i16),
    Goto(i16),
    Jsr(i16),
    Ret(u8),
    TableSwitch(TableSwitch),
    LookupSwitch(LookupSwitch),
    IReturn,
    LReturn,
    FReturn,
    DReturn,
    AReturn,
    Return,
    GetStatic(u16),
    PutStatic(u16),
    GetField(u16),
    PutField(u16),
    InvokeVirtual(u16),
    InvokeSpecial(u16),
    InvokeStatic(u16),
    InvokeInterface(u16),
    InvokeDynamic(u16),
    New(u16),
    NewArray(u8),
    ANewArray(u16),
    ArrayLength,
    AThrow,
    CheckCast(u16),
    InstanceOf(u16),
    MonitorEnter,
    MonitorExit,
    Wide,
    MultiANewArray(u16, u8),
    IfNull(i16),
    IfNonNull(i16),
    GotoW(i32),
    JsrW(i32),
    Breakpoint,
    ImpDep1,
    ImpDep2,
}

#[derive(Debug, Clone, BinRead)]
#[br(big)]
pub struct TableSwitch {
    default: i32,
    low: i32,
    high: i32,
    #[br(count = high - low + 1)]
    jump_offsets: Vec<i32>,
}

#[derive(Debug, Clone, BinRead)]
#[br(big)]
pub struct LookupSwitch {
    default: i32,
    npairs: i32,
    #[br(count = npairs)]
    match_offsets: Vec<(i32, i32)>,
}

pub fn read_instruction(mut reader: impl Read + Seek) -> Result<(usize, Opcode), InstructionError> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    match buf[0] {
        0x00 => Ok((1, Opcode::Nop)),
        0x01 => Ok((1, Opcode::AConstNull)),
        0x02 => Ok((1, Opcode::IConstM1)),
        0x03 => Ok((1, Opcode::IConst0)),
        0x04 => Ok((1, Opcode::IConst1)),
        0x05 => Ok((1, Opcode::IConst2)),
        0x06 => Ok((1, Opcode::IConst3)),
        0x07 => Ok((1, Opcode::IConst4)),
        0x08 => Ok((1, Opcode::IConst5)),
        0x09 => Ok((1, Opcode::LConst0)),
        0x0a => Ok((1, Opcode::LConst1)),
        0x0b => Ok((1, Opcode::FConst0)),
        0x0c => Ok((1, Opcode::FConst1)),
        0x0d => Ok((1, Opcode::FConst2)),
        0x0e => Ok((1, Opcode::DConst0)),
        0x0f => Ok((1, Opcode::DConst1)),
        0x10 => opcode_with_operand1!(reader, Bipush, i8),
        0x11 => opcode_with_operand2!(reader, Sipush, i16),
        0x12 => opcode_with_operand1!(reader, Ldc),
        0x13 => opcode_with_operand2!(reader, LdcW),
        0x14 => opcode_with_operand2!(reader, Ldc2W),
        0x15 => opcode_with_operand1!(reader, ILoad),
        0x16 => opcode_with_operand1!(reader, LLoad),
        0x17 => opcode_with_operand1!(reader, FLoad),
        0x18 => opcode_with_operand1!(reader, DLoad),
        0x19 => opcode_with_operand1!(reader, ALoad),
        0x1a => Ok((1, Opcode::ILoad0)),
        0x1b => Ok((1, Opcode::ILoad1)),
        0x1c => Ok((1, Opcode::ILoad2)),
        0x1d => Ok((1, Opcode::ILoad3)),
        0x1e => Ok((1, Opcode::LLoad0)),
        0x1f => Ok((1, Opcode::LLoad1)),
        0x20 => Ok((1, Opcode::LLoad2)),
        0x21 => Ok((1, Opcode::LLoad3)),
        0x22 => Ok((1, Opcode::FLoad0)),
        0x23 => Ok((1, Opcode::FLoad1)),
        0x24 => Ok((1, Opcode::FLoad2)),
        0x25 => Ok((1, Opcode::FLoad3)),
        0x26 => Ok((1, Opcode::DLoad0)),
        0x27 => Ok((1, Opcode::DLoad1)),
        0x28 => Ok((1, Opcode::DLoad2)),
        0x29 => Ok((1, Opcode::DLoad3)),
        0x2a => Ok((1, Opcode::ALoad0)),
        0x2b => Ok((1, Opcode::ALoad1)),
        0x2c => Ok((1, Opcode::ALoad2)),
        0x2d => Ok((1, Opcode::ALoad3)),
        0x2e => Ok((1, Opcode::IALoad)),
        0x2f => Ok((1, Opcode::LALoad)),
        0x30 => Ok((1, Opcode::FALoad)),
        0x31 => Ok((1, Opcode::DALoad)),
        0x32 => Ok((1, Opcode::AALoad)),
        0x33 => Ok((1, Opcode::BALoad)),
        0x34 => Ok((1, Opcode::CALoad)),
        0x35 => Ok((1, Opcode::SALoad)),
        0x36 => opcode_with_operand1!(reader, IStore),
        0x37 => opcode_with_operand1!(reader, LStore),
        0x38 => opcode_with_operand1!(reader, FStore),
        0x39 => opcode_with_operand1!(reader, DStore),
        0x3a => opcode_with_operand1!(reader, AStore),
        0x3b => Ok((1, Opcode::IStore0)),
        0x3c => Ok((1, Opcode::IStore1)),
        0x3d => Ok((1, Opcode::IStore2)),
        0x3e => Ok((1, Opcode::IStore3)),
        0x3f => Ok((1, Opcode::LStore0)),
        0x40 => Ok((1, Opcode::LStore1)),
        0x41 => Ok((1, Opcode::LStore2)),
        0x42 => Ok((1, Opcode::LStore3)),
        0x43 => Ok((1, Opcode::FStore0)),
        0x44 => Ok((1, Opcode::FStore1)),
        0x45 => Ok((1, Opcode::FStore2)),
        0x46 => Ok((1, Opcode::FStore3)),
        0x47 => Ok((1, Opcode::DStore0)),
        0x48 => Ok((1, Opcode::DStore1)),
        0x49 => Ok((1, Opcode::DStore2)),
        0x4a => Ok((1, Opcode::DStore3)),
        0x4b => Ok((1, Opcode::AStore0)),
        0x4c => Ok((1, Opcode::AStore1)),
        0x4d => Ok((1, Opcode::AStore2)),
        0x4e => Ok((1, Opcode::AStore3)),
        0x4f => Ok((1, Opcode::IAStore)),
        0x50 => Ok((1, Opcode::LAStore)),
        0x51 => Ok((1, Opcode::FAStore)),
        0x52 => Ok((1, Opcode::DAStore)),
        0x53 => Ok((1, Opcode::AAStore)),
        0x54 => Ok((1, Opcode::BAStore)),
        0x55 => Ok((1, Opcode::CAStore)),
        0x56 => Ok((1, Opcode::SAStore)),
        0x57 => Ok((1, Opcode::Pop)),
        0x58 => Ok((1, Opcode::Pop2)),
        0x59 => Ok((1, Opcode::Dup)),
        0x5a => Ok((1, Opcode::DupX1)),
        0x5b => Ok((1, Opcode::DupX2)),
        0x5c => Ok((1, Opcode::Dup2)),
        0x5d => Ok((1, Opcode::Dup2X1)),
        0x5e => Ok((1, Opcode::Dup2X2)),
        0x5f => Ok((1, Opcode::Swap)),
        0x60 => Ok((1, Opcode::IAdd)),
        0x61 => Ok((1, Opcode::LAdd)),
        0x62 => Ok((1, Opcode::FAdd)),
        0x63 => Ok((1, Opcode::DAdd)),
        0x64 => Ok((1, Opcode::ISub)),
        0x65 => Ok((1, Opcode::LSub)),
        0x66 => Ok((1, Opcode::FSub)),
        0x67 => Ok((1, Opcode::DSub)),
        0x68 => Ok((1, Opcode::IMul)),
        0x69 => Ok((1, Opcode::LMul)),
        0x6a => Ok((1, Opcode::FMul)),
        0x6b => Ok((1, Opcode::DMul)),
        0x6c => Ok((1, Opcode::IDiv)),
        0x6d => Ok((1, Opcode::LDiv)),
        0x6e => Ok((1, Opcode::FDiv)),
        0x6f => Ok((1, Opcode::DDiv)),
        0x70 => Ok((1, Opcode::IRem)),
        0x71 => Ok((1, Opcode::LRem)),
        0x72 => Ok((1, Opcode::FRem)),
        0x73 => Ok((1, Opcode::DRem)),
        0x74 => Ok((1, Opcode::INeg)),
        0x75 => Ok((1, Opcode::LNeg)),
        0x76 => Ok((1, Opcode::FNeg)),
        0x77 => Ok((1, Opcode::DNeg)),
        0x78 => Ok((1, Opcode::IShl)),
        0x79 => Ok((1, Opcode::LShl)),
        0x7a => Ok((1, Opcode::IShr)),
        0x7b => Ok((1, Opcode::LShr)),
        0x7c => Ok((1, Opcode::IUshr)),
        0x7d => Ok((1, Opcode::LUshr)),
        0x7e => Ok((1, Opcode::IAnd)),
        0x7f => Ok((1, Opcode::LAnd)),
        0x80 => Ok((1, Opcode::IOr)),
        0x81 => Ok((1, Opcode::LOr)),
        0x82 => Ok((1, Opcode::IXor)),
        0x83 => Ok((1, Opcode::LXor)),
        0x84 => opcode_with_operand2!(reader, IInc, u8, i8),
        0x85 => Ok((1, Opcode::I2L)),
        0x86 => Ok((1, Opcode::I2F)),
        0x87 => Ok((1, Opcode::I2D)),
        0x88 => Ok((1, Opcode::L2I)),
        0x89 => Ok((1, Opcode::L2F)),
        0x8a => Ok((1, Opcode::L2D)),
        0x8b => Ok((1, Opcode::F2I)),
        0x8c => Ok((1, Opcode::F2L)),
        0x8d => Ok((1, Opcode::F2D)),
        0x8e => Ok((1, Opcode::D2I)),
        0x8f => Ok((1, Opcode::D2L)),
        0x90 => Ok((1, Opcode::D2F)),
        0x91 => Ok((1, Opcode::I2B)),
        0x92 => Ok((1, Opcode::I2C)),
        0x93 => Ok((1, Opcode::I2S)),
        0x94 => Ok((1, Opcode::LCmp)),
        0x95 => Ok((1, Opcode::FCmpL)),
        0x96 => Ok((1, Opcode::FCmpG)),
        0x97 => Ok((1, Opcode::DCmpL)),
        0x98 => Ok((1, Opcode::DCmpG)),
        0x99 => opcode_with_operand2!(reader, IfEq, i16),
        0x9a => opcode_with_operand2!(reader, IfNe, i16),
        0x9b => opcode_with_operand2!(reader, IfLt, i16),
        0x9c => opcode_with_operand2!(reader, IfGe, i16),
        0x9d => opcode_with_operand2!(reader, IfGt, i16),
        0x9e => opcode_with_operand2!(reader, IfLe, i16),
        0x9f => opcode_with_operand2!(reader, IfICmpEq, i16),
        0xa0 => opcode_with_operand2!(reader, IfICmpNe, i16),
        0xa1 => opcode_with_operand2!(reader, IfICmpLt, i16),
        0xa2 => opcode_with_operand2!(reader, IfICmpGe, i16),
        0xa3 => opcode_with_operand2!(reader, IfICmpGt, i16),
        0xa4 => opcode_with_operand2!(reader, IfICmpLe, i16),
        0xa5 => opcode_with_operand2!(reader, IfACmpEq, i16),
        0xa6 => opcode_with_operand2!(reader, IfACmpNe, i16),
        0xa7 => opcode_with_operand2!(reader, Goto, i16),
        0xa8 => opcode_with_operand2!(reader, Jsr, i16),
        0xa9 => opcode_with_operand1!(reader, Ret),
        0xaa => {
            // tableswitch
            let pos = reader.stream_position()?;
            let padding = (4 - (pos % 4)) % 4;
            reader.seek(std::io::SeekFrom::Current(padding as i64))?;
            let ts: TableSwitch =
                reader
                    .read_be()
                    .map_err(|e| InstructionError::CorruptedOpcode {
                        opcode: 0xaa,
                        source: e,
                    })?;
            Ok((
                1 + (padding as usize) + (4 * 3) + 4 * ts.jump_offsets.len(),
                Opcode::TableSwitch(ts),
            ))
        }
        0xab => {
            // lookupswitch
            let pos = reader.stream_position()?;
            let padding = (4 - (pos % 4)) % 4;
            reader.seek(std::io::SeekFrom::Current(padding as i64))?;
            let ls: LookupSwitch =
                reader
                    .read_be()
                    .map_err(|e| InstructionError::CorruptedOpcode {
                        opcode: 0xab,
                        source: e,
                    })?;
            Ok((
                1 + (padding as usize) + (4 * 2) + 8 * ls.match_offsets.len(),
                Opcode::LookupSwitch(ls),
            ))
        }
        0xac => Ok((1, Opcode::IReturn)),
        0xad => Ok((1, Opcode::LReturn)),
        0xae => Ok((1, Opcode::FReturn)),
        0xaf => Ok((1, Opcode::DReturn)),
        0xb0 => Ok((1, Opcode::AReturn)),
        0xb1 => Ok((1, Opcode::Return)),
        0xb2 => opcode_with_operand2!(reader, GetStatic),
        0xb3 => opcode_with_operand2!(reader, PutStatic),
        0xb4 => opcode_with_operand2!(reader, GetField),
        0xb5 => opcode_with_operand2!(reader, PutField),
        0xb6 => opcode_with_operand2!(reader, InvokeVirtual),
        0xb7 => opcode_with_operand2!(reader, InvokeSpecial),
        0xb8 => opcode_with_operand2!(reader, InvokeStatic),
        0xb9 => {
            // For historical reasons, the operand of the invokeinterface instruction is 4 bytes long.
            // The first two bytes are the indexbyte1 and indexbyte2 bytes of the instruction, and the 3rd and 4th ones
            // can be ignored.
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            Ok((
                5,
                Opcode::InvokeInterface(u16::from_be_bytes([buf[0], buf[1]])),
            ))
        }
        0xba => opcode_with_operand2!(reader, InvokeDynamic),
        0xbb => opcode_with_operand2!(reader, New),
        0xbc => opcode_with_operand1!(reader, NewArray),
        0xbd => opcode_with_operand2!(reader, ANewArray),
        0xbe => Ok((1, Opcode::ArrayLength)),
        0xbf => Ok((1, Opcode::AThrow)),
        0xc0 => opcode_with_operand2!(reader, CheckCast),
        0xc1 => opcode_with_operand2!(reader, InstanceOf),
        0xc2 => Ok((1, Opcode::MonitorEnter)),
        0xc3 => Ok((1, Opcode::MonitorExit)),
        // TODO: 0xc4 - wide (special instruction that modifies the next instruction behavior)
        0xc5 => {
            let mut buf = [0u8; 3];
            reader.read_exact(&mut buf)?;
            Ok((
                4,
                Opcode::MultiANewArray(u16::from_be_bytes([buf[0], buf[1]]), buf[2]),
            ))
        }
        0xc6 => opcode_with_operand2!(reader, IfNull, i16),
        0xc7 => opcode_with_operand2!(reader, IfNonNull, i16),
        0xc8 => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            Ok((5, Opcode::GotoW(i32::from_be_bytes(buf))))
        }
        0xc9 => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            Ok((5, Opcode::JsrW(i32::from_be_bytes(buf))))
        }
        0xca => Ok((1, Opcode::Breakpoint)),
        0xfe => Ok((1, Opcode::ImpDep1)),
        0xff => Ok((1, Opcode::ImpDep2)),
        invalid => Err(InstructionError::InvalidOpcode { opcode: invalid }),
    }
}

impl Opcode {
    pub fn execute(
        &self,
        thread: &mut Thread,
        cm: &mut ClassManager,
    ) -> Result<InstructionSuccess, InstructionError> {
        match self {
            Opcode::Nop => constant::nop(thread),
            Opcode::AConstNull => constant::aconst_null(thread),
            Opcode::IConstM1 => constant::iconst_m1(thread),
            Opcode::IConst0 => constant::iconst_0(thread),
            Opcode::IConst1 => constant::iconst_1(thread),
            Opcode::IConst2 => constant::iconst_2(thread),
            Opcode::IConst3 => constant::iconst_3(thread),
            Opcode::IConst4 => constant::iconst_4(thread),
            Opcode::IConst5 => constant::iconst_5(thread),
            Opcode::LConst0 => constant::lconst_0(thread),
            Opcode::LConst1 => constant::lconst_1(thread),
            Opcode::FConst0 => constant::fconst_0(thread),
            Opcode::FConst1 => constant::fconst_1(thread),
            Opcode::FConst2 => constant::fconst_2(thread),
            Opcode::DConst0 => constant::dconst_0(thread),
            Opcode::DConst1 => constant::dconst_1(thread),
            Opcode::Bipush(value) => constant::bipush(thread, *value),
            Opcode::Sipush(value) => constant::sipush(thread, *value),
            Opcode::Ldc(value) => constant::ldc(thread, cm, *value),
            Opcode::LdcW(value) => constant::ldc_w(thread, cm, *value),
            Opcode::Ldc2W(value) => constant::ldc2_w(thread, cm, *value),
            Opcode::ILoad(index) => load::iload(thread, *index),
            Opcode::LLoad(index) => load::lload(thread, *index),
            Opcode::FLoad(index) => load::fload(thread, *index),
            Opcode::DLoad(index) => load::dload(thread, *index),
            Opcode::ALoad(index) => load::aload(thread, *index),
            Opcode::ILoad0 => load::iload_0(thread),
            Opcode::ILoad1 => load::iload_1(thread),
            Opcode::ILoad2 => load::iload_2(thread),
            Opcode::ILoad3 => load::iload_3(thread),
            Opcode::LLoad0 => load::lload_0(thread),
            Opcode::LLoad1 => load::lload_1(thread),
            Opcode::LLoad2 => load::lload_2(thread),
            Opcode::LLoad3 => load::lload_3(thread),
            Opcode::FLoad0 => load::fload_0(thread),
            Opcode::FLoad1 => load::fload_1(thread),
            Opcode::FLoad2 => load::fload_2(thread),
            Opcode::FLoad3 => load::fload_3(thread),
            Opcode::DLoad0 => load::dload_0(thread),
            Opcode::DLoad1 => load::dload_1(thread),
            Opcode::DLoad2 => load::dload_2(thread),
            Opcode::DLoad3 => load::dload_3(thread),
            Opcode::ALoad0 => load::aload_0(thread),
            Opcode::ALoad1 => load::aload_1(thread),
            Opcode::ALoad2 => load::aload_2(thread),
            Opcode::ALoad3 => load::aload_3(thread),
            Opcode::IALoad => load::iaload(thread),
            Opcode::LALoad => load::laload(thread),
            Opcode::FALoad => load::faload(thread),
            Opcode::DALoad => load::daload(thread),
            Opcode::AALoad => load::aaload(thread),
            Opcode::BALoad => load::baload(thread),
            Opcode::CALoad => load::caload(thread),
            Opcode::SALoad => load::saload(thread),
            Opcode::IStore(index) => store::istore(thread, *index),
            Opcode::LStore(index) => store::lstore(thread, *index),
            Opcode::FStore(index) => store::fstore(thread, *index),
            Opcode::DStore(index) => store::dstore(thread, *index),
            Opcode::AStore(index) => store::astore(thread, *index),
            Opcode::IStore0 => store::istore_0(thread),
            Opcode::IStore1 => store::istore_1(thread),
            Opcode::IStore2 => store::istore_2(thread),
            Opcode::IStore3 => store::istore_3(thread),
            Opcode::LStore0 => store::lstore_0(thread),
            Opcode::LStore1 => store::lstore_1(thread),
            Opcode::LStore2 => store::lstore_2(thread),
            Opcode::LStore3 => store::lstore_3(thread),
            Opcode::FStore0 => store::fstore_0(thread),
            Opcode::FStore1 => store::fstore_1(thread),
            Opcode::FStore2 => store::fstore_2(thread),
            Opcode::FStore3 => store::fstore_3(thread),
            Opcode::DStore0 => store::dstore_0(thread),
            Opcode::DStore1 => store::dstore_1(thread),
            Opcode::DStore2 => store::dstore_2(thread),
            Opcode::DStore3 => store::dstore_3(thread),
            Opcode::AStore0 => store::astore_0(thread),
            Opcode::AStore1 => store::astore_1(thread),
            Opcode::AStore2 => store::astore_2(thread),
            Opcode::AStore3 => store::astore_3(thread),
            Opcode::IAStore => store::iastore(thread),
            Opcode::LAStore => store::lastore(thread),
            Opcode::FAStore => store::fastore(thread),
            Opcode::DAStore => store::dastore(thread),
            Opcode::AAStore => store::aastore(thread),
            Opcode::BAStore => store::bastore(thread),
            Opcode::CAStore => store::castore(thread),
            Opcode::SAStore => store::sastore(thread),
            Opcode::Pop => stack::pop(thread),
            Opcode::Pop2 => stack::pop2(thread),
            Opcode::Dup => stack::dup(thread),
            Opcode::DupX1 => stack::dup_x1(thread),
            Opcode::DupX2 => stack::dup_x2(thread),
            Opcode::Dup2 => stack::dup2(thread),
            Opcode::Dup2X1 => stack::dup2_x1(thread),
            Opcode::Dup2X2 => stack::dup2_x2(thread),
            Opcode::Swap => stack::swap(thread),
            Opcode::IAdd => math::iadd(thread),
            Opcode::LAdd => math::ladd(thread),
            Opcode::FAdd => math::fadd(thread),
            Opcode::DAdd => math::dadd(thread),
            Opcode::ISub => math::isub(thread),
            Opcode::LSub => math::lsub(thread),
            Opcode::FSub => math::fsub(thread),
            Opcode::DSub => math::dsub(thread),
            Opcode::IMul => math::imul(thread),
            Opcode::LMul => math::lmul(thread),
            Opcode::FMul => math::fmul(thread),
            Opcode::DMul => math::dmul(thread),
            Opcode::IDiv => math::idiv(thread),
            Opcode::LDiv => math::ldiv(thread),
            Opcode::FDiv => math::fdiv(thread),
            Opcode::DDiv => math::ddiv(thread),
            Opcode::IRem => math::irem(thread),
            Opcode::LRem => math::lrem(thread),
            Opcode::FRem => math::frem(thread),
            Opcode::DRem => math::drem(thread),
            Opcode::INeg => math::ineg(thread),
            Opcode::LNeg => math::lneg(thread),
            Opcode::FNeg => math::fneg(thread),
            Opcode::DNeg => math::dneg(thread),
            Opcode::IShl => math::ishl(thread),
            Opcode::LShl => math::lshl(thread),
            Opcode::IShr => math::ishr(thread),
            Opcode::LShr => math::lshr(thread),
            // TODO: Implement IUshr and LUshr
            Opcode::IAnd => math::iand(thread),
            Opcode::LAnd => math::land(thread),
            Opcode::IOr => math::ior(thread),
            Opcode::LOr => math::lor(thread),
            Opcode::IXor => math::ixor(thread),
            Opcode::LXor => math::lxor(thread),
            Opcode::IInc(index, value) => math::iinc(thread, *index, *value),
            Opcode::I2L => conversion::i2l(thread),
            Opcode::I2F => conversion::i2f(thread),
            Opcode::I2D => conversion::i2d(thread),
            Opcode::L2I => conversion::l2i(thread),
            Opcode::L2F => conversion::l2f(thread),
            Opcode::L2D => conversion::l2d(thread),
            Opcode::F2I => conversion::f2i(thread),
            Opcode::F2L => conversion::f2l(thread),
            Opcode::F2D => conversion::f2d(thread),
            Opcode::D2I => conversion::d2i(thread),
            Opcode::D2L => conversion::d2l(thread),
            Opcode::D2F => conversion::d2f(thread),
            Opcode::I2B => conversion::i2b(thread),
            Opcode::I2C => conversion::i2c(thread),
            Opcode::I2S => conversion::i2s(thread),
            Opcode::LCmp => comparison::lcmp(thread),
            Opcode::FCmpL => comparison::fcmpl(thread),
            Opcode::FCmpG => comparison::fcmpg(thread),
            Opcode::DCmpL => comparison::dcmpl(thread),
            Opcode::DCmpG => comparison::dcmpg(thread),
            Opcode::IfEq(value) => comparison::ifeq(thread, *value),
            Opcode::IfNe(value) => comparison::ifne(thread, *value),
            Opcode::IfLt(value) => comparison::iflt(thread, *value),
            Opcode::IfGe(value) => comparison::ifge(thread, *value),
            Opcode::IfGt(value) => comparison::ifgt(thread, *value),
            Opcode::IfLe(value) => comparison::ifle(thread, *value),
            Opcode::IfICmpEq(value) => comparison::if_icmpeq(thread, *value),
            Opcode::IfICmpNe(value) => comparison::if_icmpne(thread, *value),
            Opcode::IfICmpLt(value) => comparison::if_icmplt(thread, *value),
            Opcode::IfICmpGe(value) => comparison::if_icmpge(thread, *value),
            Opcode::IfICmpGt(value) => comparison::if_icmpgt(thread, *value),
            Opcode::IfICmpLe(value) => comparison::if_icmple(thread, *value),
            Opcode::IfACmpEq(value) => comparison::if_acmpeq(thread, *value),
            Opcode::IfACmpNe(value) => comparison::if_acmpne(thread, *value),
            Opcode::Goto(value) => control::goto(thread, *value),
            Opcode::Jsr(value) => control::jsr(thread, *value),
            Opcode::Ret(value) => control::ret(thread, *value),
            Opcode::TableSwitch(ts) => control::tableswitch(thread, ts),
            Opcode::LookupSwitch(ls) => control::lookupswitch(thread, ls),
            Opcode::IReturn => control::ireturn(thread),
            Opcode::LReturn => control::lreturn(thread),
            Opcode::FReturn => control::freturn(thread),
            Opcode::DReturn => control::dreturn(thread),
            Opcode::AReturn => control::areturn(thread),
            Opcode::Return => control::vreturn(thread),
            Opcode::GetStatic(index) => reference::getstatic(thread, cm, *index),
            Opcode::PutStatic(index) => reference::putstatic(thread, cm, *index),
            Opcode::GetField(index) => reference::getfield(thread, cm, *index),
            Opcode::PutField(index) => reference::putfield(thread, cm, *index),
            Opcode::InvokeVirtual(index) => reference::invokevirtual(thread, cm, *index),
            Opcode::InvokeSpecial(index) => reference::invokespecial(thread, cm, *index),
            Opcode::InvokeInterface(index) => reference::invokeinterface(thread, cm, *index),
            // TODO: Implement InvokeDynamic
            Opcode::InvokeStatic(index) => reference::invokestatic(thread, cm, *index),
            Opcode::New(index) => reference::new(thread, cm, *index),
            Opcode::NewArray(atype) => reference::newarray(thread, *atype),
            Opcode::ANewArray(index) => reference::anewarray(thread, cm, *index),
            Opcode::ArrayLength => reference::arraylength(thread),
            // TODO: Implement AThrow, CheckCast, InstanceOf, MonitorEnter, MonitorExit
            // TODO: Implement Wide
            // TODO: Implement MultiANewArray
            Opcode::IfNull(value) => extended::ifnull(thread, *value),
            Opcode::IfNonNull(value) => extended::ifnonnull(thread, *value),
            Opcode::GotoW(value) => control::goto_w(thread, *value),
            Opcode::JsrW(value) => control::jsr_w(thread, *value),
            x => Err(InstructionError::UnimplementedInstruction { opcode: x.clone() }),
        }
    }
}

#[derive(Debug, Snafu)]
pub enum InstructionError {
    #[snafu(display("Class loading error for class {}: {}", class_name, source))]
    ClassLoadingError {
        class_name: String,
        source: Box<crate::class_loader::ClassLoadingError>,
    },

    #[snafu(display("Invalid state: {}", context))]
    InvalidState { context: String },

    #[snafu(display("Unimplemented instruction, opcode: {:?}", opcode))]
    UnimplementedInstruction { opcode: Opcode },

    #[snafu(context(false))]
    #[snafu(display("IO error: {}", source))]
    IOError { source: std::io::Error },

    #[snafu(display("Invalid opcode: {}", opcode))]
    InvalidOpcode { opcode: u8 },

    #[snafu(display("Corrupted opcode: {}, context: {:?}", opcode, source))]
    CorruptedOpcode { opcode: u8, source: ParsingError },
}

/// The result of executing an instruction.
///
/// Indicate where the next instruction should be read from.
#[derive(Debug, Clone)]
pub enum InstructionSuccess {
    /// The next instruction is the next instruction in the code.
    ///
    /// The offset is the number of bytes to skip to get to the next instruction.
    Next(usize),

    /// Jump relatively to the address.
    ///
    /// The offset is the number of bytes to add/remove to get to the next instruction.
    JumpRelative(isize),

    /// Jump absolutely to the address.
    JumpAbsolute(usize),

    /// Frame has been added/removed. The PC is set to the value given.
    ///
    /// A frame has been pushed or popped from the stack. The PC is set to the value given.
    /// In case of the return of a method invokation, the PC is generally set to the InvokationReturnAddress of the last frame.
    /// In case of the invokation of a method, the PC is generally set to the first instruction of the method, ie 0.
    FrameChange(usize),

    /// The execution of the thread has completed.
    ///
    /// The stack is empty, the thread has completed its execution.
    Completed,
}

#[macro_use]
mod macros {

    #[macro_export]
    macro_rules! opcode_with_operand1 {
        ($reader:expr, $name:ident) => {{
            let mut buf = [0u8; 1];
            $reader.read_exact(&mut buf)?;
            Ok((2, Opcode::$name(buf[0])))
        }};
        ($reader:expr, $name:ident, $ty:ty) => {{
            let mut buf = [0u8; 1];
            $reader.read_exact(&mut buf)?;
            Ok((2, Opcode::$name(<$ty>::from_be_bytes(buf))))
        }};
    }

    #[macro_export]
    macro_rules! opcode_with_operand2 {
        ($reader:expr, $name:ident) => {{
            let mut buf = [0u8; 2];
            $reader.read_exact(&mut buf)?;
            Ok((3, Opcode::$name(u16::from_be_bytes(buf))))
        }};
        ($reader:expr, $name:ident, $ty:ty) => {{
            let mut buf = [0u8; 2];
            $reader.read_exact(&mut buf)?;
            Ok((3, Opcode::$name(<$ty>::from_be_bytes(buf))))
        }};
        ($reader:expr, $name:ident, $ty1:ty, $ty2:ty) => {{
            let mut buf = [0u8; 2];
            $reader.read_exact(&mut buf)?;
            Ok((3, Opcode::$name(buf[0] as $ty1, buf[1] as $ty2)))
        }};
    }
}
