// An ELF executable file format parser
// References:
//     https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
//     https://wiki.osdev.org/ELF

use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use tabled::{Table, Tabled};

#[derive(Debug, Default, PartialEq)]
pub struct Cli {
    pub filepath: PathBuf,
    pub to_process: ElfParts,
}

#[derive(Debug, Default, PartialEq)]
pub enum ElfParts {
    #[default]
    Header,
    ProgramHeader,
}

pub trait Parse {
    type Item;
    type Error;

    fn parse(args: impl Iterator<Item = String>) -> Result<Self::Item, Self::Error>;
    fn helper();
}

impl Parse for Cli {
    type Item = Cli;
    type Error = String;

    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self::Item, Self::Error> {
        let mut cli = Cli::default();
        while let Some(next) = args.next() {
            let next = next.as_str();
            if next == "--filepath" || next == "-f" {
                let next: String = args
                    .next()
                    .ok_or_else(|| Err::<Self::Item, Self::Error>("Missing filepath".to_string()))
                    .unwrap();
                cli.filepath = Path::new(&next).to_path_buf();
            } else if next == "--help" || next == "-h" {
                Self::helper();
                std::process::exit(0);
            } else if next == "--header" || next == "-e" {
                cli.to_process = ElfParts::Header;
            } else if next == "--program" || next == "-p" {
                cli.to_process = ElfParts::ProgramHeader;
            }
        }

        if cli == Cli::default() {
            return Err("Missing args!".into());
        }

        Ok(cli)
    }

    fn helper() {
        // TODO: Fix this; Use raw strings
        const USAGE_INFO: &str = "
Usage:
    program <flags>
        --help    , -h    Show this information
        --filepath, -f    Path to the elf file
        --header  , -e    Display only the elf header
        --program , -p    Display only the elf program header
        ";

        println!("{USAGE_INFO}");
    }
}

#[derive(Debug)]
pub struct ElfHeader {
    pub magic_number: ElfMagicNumber,
    pub platform_type: ElfPlatformType,
    pub endianness: ElfEndianness,
    pub elf_header_version: ElfHeaderVersion,
    pub target_system_abi: ElfTargetSystemAbi,
    pub target_abi_version: ElfTargetAbiVersion,
    pub object_file_type: ElfObjectFileType,
    pub instruction_set: ElfInstructionSet,
    pub elf_version: ElfVersion,
    pub entry_point: ElfEntryPoint,
    pub program_header_offset: ElfProgramHeaderOffset,
    pub section_header_offset: ElfSectionHeaderOffset,
    pub flags: ElfFlags,
    pub header_size: ElfHeaderSize,
    pub program_header_entry_size: ElfProgramHeaderEntrySize,
    pub program_header_entry_count: ElfProgramHeaderEntryCount,
    pub section_header_entry_size: ElfSectionHeaderEntrySize,
    pub section_header_entry_count: ElfSectionHeaderEntryCount,
    pub section_header_sections_table_index: ElfSectionHeaderSectionsTableIndex,
}

#[derive(Debug)]
pub struct ElfHeaderRow {
    pub field: String,
    pub value: String,
}

impl std::fmt::Display for ElfHeaderRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = format!("{}\t{}", self.field, self.value);
        write!(f, "{}", txt)
    }
}

impl std::fmt::Display for ElfHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rows = self.to_table_rows();
        let txt = rows
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "{}", txt)
    }
}

#[derive(Debug)]
pub struct ElfSectionHeaderSectionsTableIndex(u16);

impl std::fmt::Display for ElfSectionHeaderSectionsTableIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSectionHeaderEntrySize(u16);

impl std::fmt::Display for ElfSectionHeaderEntrySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSectionHeaderEntryCount(u16);

impl std::fmt::Display for ElfSectionHeaderEntryCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfProgramHeaderEntryCount(u16);

impl ElfProgramHeaderEntryCount {
    pub fn inner(&self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for ElfProgramHeaderEntryCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfProgramHeaderEntrySize(u16);

impl std::fmt::Display for ElfProgramHeaderEntrySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfHeaderSize(u16);

impl std::fmt::Display for ElfHeaderSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfFlags(u32);

impl std::fmt::Display for ElfFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSectionHeaderOffset(usize);

impl std::fmt::Display for ElfSectionHeaderOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfProgramHeaderOffset(usize);

impl std::fmt::Display for ElfProgramHeaderOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfEntryPoint(usize);

impl std::fmt::Display for ElfEntryPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfVersion(u32);

impl std::fmt::Display for ElfVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug)]
pub enum ElfInstructionSet {
    AdvancedLogicCorpTinyJ,
    AmdX86_64,
    ArgonautRiscCore,
    Arm,
    Arm64bit,
    AtTwe32100,
    AxisCommunications32bit,
    BerkeleyPacketFilter,
    DensoNdr1,
    DigitalAlpha,
    DigitalEquipmentCorpPdp10,
    DigitalEquipmentCorpPdp11,
    DigitalVax,
    Element14_64bitDSP,
    FujitsuFr20,
    FujitsuMma,
    HewlettPackardPaRisc,
    HitachiH8500,
    HitachiH8S,
    HitachiH8_300,
    HitachiH8_300H,
    Ia64,
    IbmSpuSpc,
    Ibmsystem370,
    InfineonTechnologies32bit,
    Intel80860,
    Intel80960,
    IntelMcu,
    LoongArch,
    LsiLogic16bitDsp,
    Mips,
    Mipsrs3000LittleEndian,
    McstElbrusE2k,
    Motorola68000M68k,
    Motorola88000M88k,
    MotorolaColdFire,
    MotorolaM68hc12,
    MotorolaMc68hc05,
    MotorolaMc68hc08,
    MotorolaMc68hc11,
    MotorolaMc68hc16,
    MotorolaRce,
    MotorolaStarCore,
    NecV800,
    PowerPc,
    PowerPc64bit,
    Reserved,
    RiscV,
    S390,
    Sparc,
    SiemensFx66,
    SiemensPcp,
    SiemensTriCore,
    SiliconGraphicsSvx,
    SonyDsp,
    SonyNCpu,
    SparcV9,
    StanfordMipsX,
    StmicroElectronicsSt100,
    StmicroElectronicsSt19,
    StmicroElectronicsSt7,
    StmicroElectronicsSt9,
    SuperH,
    Tms320c6000Family,
    ToyotaMe16,
    TrwRh32,
    UnSpecified,
    Wdc65c816,
    X86,
    ZilogZ80,
}

impl std::fmt::Display for ElfInstructionSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            ElfInstructionSet::UnSpecified => "No specific instruction set",
            ElfInstructionSet::AtTwe32100 => "AT&T WE 32100",
            ElfInstructionSet::Sparc => "SPARC",
            ElfInstructionSet::X86 => "x86",
            ElfInstructionSet::Motorola68000M68k => "Motorola 68000 (M68k)",
            ElfInstructionSet::Motorola88000M88k => "Motorola 88000 (M88k)",
            ElfInstructionSet::IntelMcu => "Intel MCU",
            ElfInstructionSet::Intel80860 => "Intel 80860",
            ElfInstructionSet::Mips => "MIPS",
            ElfInstructionSet::Ibmsystem370 => "IBM System/370",
            ElfInstructionSet::Mipsrs3000LittleEndian => "MIPS RS3000 Little-endian",
            ElfInstructionSet::Reserved => "Reserved for future use",
            ElfInstructionSet::HewlettPackardPaRisc => "Hewlett-Packard PA-RISC",
            ElfInstructionSet::Intel80960 => "Intel 80960",
            ElfInstructionSet::PowerPc => "PowerPC",
            ElfInstructionSet::PowerPc64bit => "PowerPC (64-bit)",
            ElfInstructionSet::S390 => "S390, including S390x",
            ElfInstructionSet::IbmSpuSpc => "IBM SPU/SPC",
            ElfInstructionSet::NecV800 => "NEC V800",
            ElfInstructionSet::FujitsuFr20 => "Fujitsu FR20",
            ElfInstructionSet::TrwRh32 => "TRW RH-32",
            ElfInstructionSet::MotorolaRce => "Motorola RCE",
            ElfInstructionSet::Arm => "Arm (up to Armv7/AArch32)",
            ElfInstructionSet::DigitalAlpha => "Digital Alpha",
            ElfInstructionSet::SuperH => "SuperH",
            ElfInstructionSet::SparcV9 => "SPARC Version 9",
            ElfInstructionSet::SiemensTriCore => "Siemens TriCore embedded processor",
            ElfInstructionSet::ArgonautRiscCore => "Argonaut RISC Core",
            ElfInstructionSet::HitachiH8_300 => "Hitachi H8/300",
            ElfInstructionSet::HitachiH8_300H => "Hitachi H8/300H",
            ElfInstructionSet::HitachiH8S => "Hitachi H8S",
            ElfInstructionSet::HitachiH8500 => "Hitachi H8/500",
            ElfInstructionSet::Ia64 => "IA-64",
            ElfInstructionSet::StanfordMipsX => "Stanford MIPS-X",
            ElfInstructionSet::MotorolaColdFire => "Motorola ColdFire",
            ElfInstructionSet::MotorolaM68hc12 => "Motorola M68HC12",
            ElfInstructionSet::FujitsuMma => "Fujitsu MMA Multimedia Accelerator",
            ElfInstructionSet::SiemensPcp => "Siemens PCP",
            ElfInstructionSet::SonyNCpu => "Sony nCPU embedded RISC processor",
            ElfInstructionSet::DensoNdr1 => "Denso NDR1 microprocessor",
            ElfInstructionSet::MotorolaStarCore => "Motorola Star*Core processor",
            ElfInstructionSet::ToyotaMe16 => "Toyota ME16 processor",
            ElfInstructionSet::StmicroElectronicsSt100 => "STMicroelectronics ST100 processor",
            ElfInstructionSet::AdvancedLogicCorpTinyJ => {
                "Advanced Logic Corp. TinyJ embedded processor family"
            }
            ElfInstructionSet::AmdX86_64 => "AMD x86-64",
            ElfInstructionSet::SonyDsp => "Sony DSP Processor",
            ElfInstructionSet::DigitalEquipmentCorpPdp10 => "Digital Equipment Corp. PDP-10",
            ElfInstructionSet::DigitalEquipmentCorpPdp11 => "Digital Equipment Corp. PDP-11",
            ElfInstructionSet::SiemensFx66 => "Siemens FX66 microcontroller",
            ElfInstructionSet::StmicroElectronicsSt9 => {
                "STMicroelectronics ST9+ 8/16-bit microcontroller"
            }
            ElfInstructionSet::StmicroElectronicsSt7 => {
                "STMicroelectronics ST7 8-bit microcontroller"
            }
            ElfInstructionSet::MotorolaMc68hc16 => "Motorola MC68HC16 Microcontroller",
            ElfInstructionSet::MotorolaMc68hc11 => "Motorola MC68HC11 Microcontroller",
            ElfInstructionSet::MotorolaMc68hc08 => "Motorola MC68HC08 Microcontroller",
            ElfInstructionSet::MotorolaMc68hc05 => "Motorola MC68HC05 Microcontroller",
            ElfInstructionSet::SiliconGraphicsSvx => "Silicon Graphics SVx",
            ElfInstructionSet::StmicroElectronicsSt19 => {
                "STMicroelectronics ST19 8-bit microcontroller"
            }
            ElfInstructionSet::DigitalVax => "Digital VAX",
            ElfInstructionSet::AxisCommunications32bit => {
                "Axis Communications 32-bit embedded processor"
            }
            ElfInstructionSet::InfineonTechnologies32bit => {
                "Infineon Technologies 32-bit embedded processor"
            }
            ElfInstructionSet::Element14_64bitDSP => "Element 14 64-bit DSP Processor",
            ElfInstructionSet::LsiLogic16bitDsp => "LSI Logic 16-bit DSP Processor",
            ElfInstructionSet::Tms320c6000Family => "TMS320C6000 Family",
            ElfInstructionSet::McstElbrusE2k => "MCST Elbrus e2k",
            ElfInstructionSet::Arm64bit => "Arm 64-bits (Armv8/AArch64)",
            ElfInstructionSet::ZilogZ80 => "Zilog Z80",
            ElfInstructionSet::RiscV => "RISC-V",
            ElfInstructionSet::BerkeleyPacketFilter => "Berkeley Packet Filter",
            ElfInstructionSet::Wdc65c816 => "WDC 65C816",
            ElfInstructionSet::LoongArch => "LoongArch",
        };

        write!(f, "{}", txt)
    }
}

#[derive(Debug)]
pub enum ElfObjectFileType {
    EtNone,
    EtRel,
    EtExec,
    EtDyn,
    EtCore,
    EtLoos,
    EtHio,
    EtLoproc,
    EtHiproc,
}

impl std::fmt::Display for ElfObjectFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            ElfObjectFileType::EtNone => "ET_NONE",
            ElfObjectFileType::EtRel => "ET_REL",
            ElfObjectFileType::EtExec => "ET_EXEC",
            ElfObjectFileType::EtDyn => "ET_DYN",
            ElfObjectFileType::EtCore => "ET_CORE",
            ElfObjectFileType::EtLoos => "ET_LOOS",
            ElfObjectFileType::EtHio => "ET_HIO",
            ElfObjectFileType::EtLoproc => "ET_LOPROC",
            ElfObjectFileType::EtHiproc => "ET_HIPROC",
        };

        write!(f, "{}", txt)
    }
}

#[derive(Debug)]
pub struct ElfReservedPadding([u8; 7]);

impl std::fmt::Display for ElfReservedPadding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfTargetAbiVersion(u8);

impl std::fmt::Display for ElfTargetAbiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug)]
pub enum ElfTargetSystemAbi {
    SystemV,
    Hpux,
    NetBsd,
    Linux,
    GnuHurd,
    Solaris,
    AixMonterey,
    Irix,
    FreeBsd,
    Tru64,
    NovellModesto,
    OpenBsd,
    OpenVms,
    NonStopKernel,
    Aros,
    FenixOs,
    NuxiCloudAbi,
    StratusTechnologiesOpenVos,
}

impl std::fmt::Display for ElfTargetSystemAbi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            ElfTargetSystemAbi::SystemV => "System V",
            ElfTargetSystemAbi::Hpux => "HP-UX",
            ElfTargetSystemAbi::NetBsd => "NetBSD",
            ElfTargetSystemAbi::Linux => "Linux",
            ElfTargetSystemAbi::GnuHurd => "GNU Hurd",
            ElfTargetSystemAbi::Solaris => "Solaris",
            ElfTargetSystemAbi::AixMonterey => "AIX (Monterey)",
            ElfTargetSystemAbi::Irix => "IRIX",
            ElfTargetSystemAbi::FreeBsd => "FreeBSD",
            ElfTargetSystemAbi::Tru64 => "Tru64",
            ElfTargetSystemAbi::NovellModesto => "Novell Modesto",
            ElfTargetSystemAbi::OpenBsd => "OpenBSD",
            ElfTargetSystemAbi::OpenVms => "OpenVMS",
            ElfTargetSystemAbi::NonStopKernel => "NonStop Kernel",
            ElfTargetSystemAbi::Aros => "AROS",
            ElfTargetSystemAbi::FenixOs => "FenixOS",
            ElfTargetSystemAbi::NuxiCloudAbi => "Nuxi CloudABI",
            ElfTargetSystemAbi::StratusTechnologiesOpenVos => "Stratus Technologies OpenVOS",
        };

        write!(f, "{}", txt)
    }
}

#[derive(Debug)]
pub struct ElfHeaderVersion(u8);

impl std::fmt::Display for ElfHeaderVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug)]
pub enum ElfPlatformType {
    Bit32,
    Bit64,
}

impl std::fmt::Display for ElfPlatformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            ElfPlatformType::Bit32 => "32-bit",
            ElfPlatformType::Bit64 => "64-bit",
        };

        write!(f, "{}", txt)
    }
}

#[derive(Debug)]
pub enum ElfEndianness {
    Little,
    Big,
}

impl std::fmt::Display for ElfEndianness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            ElfEndianness::Little => "Little",
            ElfEndianness::Big => "Big",
        };

        write!(f, "{}", txt)
    }
}

impl ElfEndianness {
    pub fn u16_from(&self, bytes: &[u8]) -> u16 {
        match self {
            ElfEndianness::Little => u16::from_le_bytes([bytes[0], bytes[1]]),
            ElfEndianness::Big => u16::from_be_bytes([bytes[0], bytes[1]]),
        }
    }

    pub fn u32_from(&self, bytes: &[u8]) -> u32 {
        match self {
            ElfEndianness::Little => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            ElfEndianness::Big => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        }
    }

    pub fn u64_from(&self, bytes: &[u8]) -> u64 {
        match self {
            ElfEndianness::Little => u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
            ElfEndianness::Big => u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
        }
    }
}

#[derive(Debug)]
pub struct ElfMagicNumber([u8; 4]);

impl std::fmt::Display for ElfMagicNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfBinary {
    pub header: ElfHeader,
    pub program_header: ElfProgramHeader,
}

impl std::fmt::Display for ElfBinary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let header = self.header.to_string();

        let txt = format!("HEADER: {}", header);

        write!(f, "{}", txt)
    }
}

pub fn parse_section_header_sections_table_index(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfSectionHeaderSectionsTableIndex, String> {
    let bytes = [content[*pointer], content[*pointer + 1]];
    *pointer += 2;
    let index = endian.u16_from(&bytes);

    Ok(ElfSectionHeaderSectionsTableIndex(index))
}

pub fn parse_section_header_entry_count(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfSectionHeaderEntryCount, String> {
    let bytes = [content[*pointer], content[*pointer + 1]];
    *pointer += 2;
    let size = endian.u16_from(&bytes);

    Ok(ElfSectionHeaderEntryCount(size))
}

pub fn parse_section_header_entry_size(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfSectionHeaderEntrySize, String> {
    let bytes = [content[*pointer], content[*pointer + 1]];
    *pointer += 2;
    let size = endian.u16_from(&bytes);

    Ok(ElfSectionHeaderEntrySize(size))
}

pub fn parse_program_header_entry_count(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfProgramHeaderEntryCount, String> {
    let bytes = [content[*pointer], content[*pointer + 1]];
    *pointer += 2;
    let size = endian.u16_from(&bytes);

    Ok(ElfProgramHeaderEntryCount(size))
}

pub fn parse_program_header_entry_size(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfProgramHeaderEntrySize, String> {
    let bytes = [content[*pointer], content[*pointer + 1]];
    *pointer += 2;
    let size = endian.u16_from(&bytes);

    Ok(ElfProgramHeaderEntrySize(size))
}

pub fn parse_header_size(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfHeaderSize, String> {
    let bytes = [content[*pointer], content[*pointer + 1]];
    *pointer += 2;
    let size = endian.u16_from(&bytes);

    Ok(ElfHeaderSize(size))
}

pub fn parse_flags(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfFlags, String> {
    let bytes = [
        content[*pointer],
        content[*pointer + 1],
        content[*pointer + 2],
        content[*pointer + 3],
    ];
    *pointer += 4;
    let flags = endian.u32_from(&bytes);

    Ok(ElfFlags(flags))
}

pub fn parse_section_header_offset(
    pointer: &mut usize,
    content: &[u8],
    platform: &ElfPlatformType,
    endian: &ElfEndianness,
) -> Result<ElfSectionHeaderOffset, String> {
    let offset = {
        match platform {
            ElfPlatformType::Bit32 => {
                let bytes = [
                    content[*pointer],
                    content[*pointer + 1],
                    content[*pointer + 2],
                    content[*pointer + 3],
                ];
                *pointer += 4;
                let offset = endian.u32_from(&bytes);
                ElfSectionHeaderOffset(offset as usize)
            }
            ElfPlatformType::Bit64 => {
                let bytes = [
                    content[*pointer],
                    content[*pointer + 1],
                    content[*pointer + 2],
                    content[*pointer + 3],
                    content[*pointer + 4],
                    content[*pointer + 5],
                    content[*pointer + 6],
                    content[*pointer + 7],
                ];
                *pointer += 8;

                let offset = endian.u64_from(&bytes);
                ElfSectionHeaderOffset(offset as usize)
            }
        }
    };

    Ok(offset)
}

pub fn parse_program_header_offset(
    pointer: &mut usize,
    content: &[u8],
    platform: &ElfPlatformType,
    endian: &ElfEndianness,
) -> Result<ElfProgramHeaderOffset, String> {
    let offset = {
        match platform {
            ElfPlatformType::Bit32 => {
                let bytes = [
                    content[*pointer],
                    content[*pointer + 1],
                    content[*pointer + 2],
                    content[*pointer + 3],
                ];
                *pointer += 4;
                let offset = endian.u32_from(&bytes);
                ElfProgramHeaderOffset(offset as usize)
            }
            ElfPlatformType::Bit64 => {
                let bytes = [
                    content[*pointer],
                    content[*pointer + 1],
                    content[*pointer + 2],
                    content[*pointer + 3],
                    content[*pointer + 4],
                    content[*pointer + 5],
                    content[*pointer + 6],
                    content[*pointer + 7],
                ];
                *pointer += 8;

                let offset = endian.u64_from(&bytes);
                ElfProgramHeaderOffset(offset as usize)
            }
        }
    };

    Ok(offset)
}

pub fn parse_entry_point(
    pointer: &mut usize,
    content: &[u8],
    platform: &ElfPlatformType,
    endian: &ElfEndianness,
) -> Result<ElfEntryPoint, String> {
    let entry_point = {
        match platform {
            ElfPlatformType::Bit32 => {
                let bytes = [
                    content[*pointer],
                    content[*pointer + 1],
                    content[*pointer + 2],
                    content[*pointer + 3],
                ];
                *pointer += 4;
                let entry = endian.u32_from(&bytes);
                ElfEntryPoint(entry as usize)
            }
            ElfPlatformType::Bit64 => {
                let bytes = [
                    content[*pointer],
                    content[*pointer + 1],
                    content[*pointer + 2],
                    content[*pointer + 3],
                    content[*pointer + 4],
                    content[*pointer + 5],
                    content[*pointer + 6],
                    content[*pointer + 7],
                ];
                *pointer += 8;

                let entry = endian.u64_from(&bytes);
                ElfEntryPoint(entry as usize)
            }
        }
    };

    Ok(entry_point)
}

pub fn parse_elf_version(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfVersion, String> {
    let bytes = [
        content[*pointer],
        content[*pointer + 1],
        content[*pointer + 2],
        content[*pointer + 3],
    ];

    *pointer += 4;

    let e_version = endian.u32_from(&bytes);
    Ok(ElfVersion(e_version))
}

pub fn parse_instruction_set(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfInstructionSet, String> {
    let set = [content[*pointer], content[*pointer + 1]];
    *pointer += 2;
    let set = endian.u16_from(&set);

    let set = match set {
        0x00 => ElfInstructionSet::UnSpecified,
        0x01 => ElfInstructionSet::AtTwe32100,
        0x02 => ElfInstructionSet::Sparc,
        0x03 => ElfInstructionSet::X86,
        0x04 => ElfInstructionSet::Motorola68000M68k,
        0x05 => ElfInstructionSet::Motorola88000M88k,
        0x06 => ElfInstructionSet::IntelMcu,
        0x07 => ElfInstructionSet::Intel80860,
        0x08 => ElfInstructionSet::Mips,
        0x09 => ElfInstructionSet::Ibmsystem370,
        0x0A => ElfInstructionSet::Mipsrs3000LittleEndian,
        // 0x0B – 0x0E 	Reserved for future use
        0x0F => ElfInstructionSet::HewlettPackardPaRisc,
        0x13 => ElfInstructionSet::Intel80960,
        0x14 => ElfInstructionSet::PowerPc,
        0x15 => ElfInstructionSet::PowerPc64bit,
        0x16 => ElfInstructionSet::S390,
        0x17 => ElfInstructionSet::IbmSpuSpc,
        // 0x18 => ElfInstructionSet::– 0x23 	Reserved for future use,
        0x24 => ElfInstructionSet::NecV800,
        0x25 => ElfInstructionSet::FujitsuFr20,
        0x26 => ElfInstructionSet::TrwRh32,
        0x27 => ElfInstructionSet::MotorolaRce,
        0x28 => ElfInstructionSet::Arm,
        0x29 => ElfInstructionSet::DigitalAlpha,
        0x2A => ElfInstructionSet::SuperH,
        0x2B => ElfInstructionSet::SparcV9,
        0x2C => ElfInstructionSet::SiemensTriCore,
        0x2D => ElfInstructionSet::ArgonautRiscCore,
        0x2E => ElfInstructionSet::HitachiH8_300,
        0x2F => ElfInstructionSet::HitachiH8_300H,
        0x30 => ElfInstructionSet::HitachiH8S,
        0x31 => ElfInstructionSet::HitachiH8500,
        0x32 => ElfInstructionSet::Ia64,
        0x33 => ElfInstructionSet::StanfordMipsX,
        0x34 => ElfInstructionSet::MotorolaColdFire,
        0x35 => ElfInstructionSet::MotorolaM68hc12,
        0x36 => ElfInstructionSet::FujitsuMma,
        0x37 => ElfInstructionSet::SiemensPcp,
        0x38 => ElfInstructionSet::SonyNCpu,
        0x39 => ElfInstructionSet::DensoNdr1,
        0x3A => ElfInstructionSet::MotorolaStarCore,
        0x3B => ElfInstructionSet::ToyotaMe16,
        0x3C => ElfInstructionSet::StmicroElectronicsSt100,
        0x3D => ElfInstructionSet::AdvancedLogicCorpTinyJ,
        0x3E => ElfInstructionSet::AmdX86_64,
        0x3F => ElfInstructionSet::SonyDsp,
        0x40 => ElfInstructionSet::DigitalEquipmentCorpPdp10,
        0x41 => ElfInstructionSet::DigitalEquipmentCorpPdp11,
        0x42 => ElfInstructionSet::SiemensFx66,
        0x43 => ElfInstructionSet::StmicroElectronicsSt9,
        0x44 => ElfInstructionSet::StmicroElectronicsSt7,
        0x45 => ElfInstructionSet::MotorolaMc68hc16,
        0x46 => ElfInstructionSet::MotorolaMc68hc11,
        0x47 => ElfInstructionSet::MotorolaMc68hc08,
        0x48 => ElfInstructionSet::MotorolaMc68hc05,
        0x49 => ElfInstructionSet::SiliconGraphicsSvx,
        0x4A => ElfInstructionSet::StmicroElectronicsSt19,
        0x4B => ElfInstructionSet::DigitalVax,
        0x4C => ElfInstructionSet::AxisCommunications32bit,
        0x4D => ElfInstructionSet::InfineonTechnologies32bit,
        0x4E => ElfInstructionSet::Element14_64bitDSP,
        0x4F => ElfInstructionSet::LsiLogic16bitDsp,
        0x8C => ElfInstructionSet::Tms320c6000Family,
        0xAF => ElfInstructionSet::McstElbrusE2k,
        0xB7 => ElfInstructionSet::Arm64bit,
        0xDC => ElfInstructionSet::ZilogZ80,
        0xF3 => ElfInstructionSet::RiscV,
        0xF7 => ElfInstructionSet::BerkeleyPacketFilter,
        0x101 => ElfInstructionSet::Wdc65c816,
        0x102 => ElfInstructionSet::LoongArch,
        _ => return Err("Unsupported instructoin set".into()),
    };

    Ok(set)
}

pub fn parse_object_file_type(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfObjectFileType, String> {
    let f_type = [content[*pointer], content[*pointer + 1]];
    *pointer += 2;
    let f_type = endian.u16_from(&f_type);

    let f_type = match f_type {
        0x00 => ElfObjectFileType::EtNone,
        0x01 => ElfObjectFileType::EtRel,
        0x02 => ElfObjectFileType::EtExec,
        0x03 => ElfObjectFileType::EtDyn,
        0x04 => ElfObjectFileType::EtCore,
        0xFE00 => ElfObjectFileType::EtLoos,
        0xFEFF => ElfObjectFileType::EtHio,
        0xFF00 => ElfObjectFileType::EtLoproc,
        0xFFFF => ElfObjectFileType::EtHiproc,
        _ => return Err("Unsupported Object File Type".into()),
    };

    Ok(f_type)
}

pub fn parse_reserved_padding(
    pointer: &mut usize,
    content: &[u8],
) -> Result<ElfReservedPadding, String> {
    let padding = [
        content[*pointer],
        content[*pointer + 1],
        content[*pointer + 2],
        content[*pointer + 3],
        content[*pointer + 4],
        content[*pointer + 5],
        content[*pointer + 6],
    ];
    *pointer += 7;

    Ok(ElfReservedPadding(padding))
}

pub fn parse_target_abi_version(
    pointer: &mut usize,
    content: &[u8],
) -> Result<ElfTargetAbiVersion, String> {
    let ver = content[*pointer];
    *pointer += 1;
    Ok(ElfTargetAbiVersion(ver))
}

pub fn parse_target_system_abi(
    pointer: &mut usize,
    content: &[u8],
) -> Result<ElfTargetSystemAbi, String> {
    let t_abi = match content[*pointer] {
        0x00 => ElfTargetSystemAbi::SystemV,
        0x01 => ElfTargetSystemAbi::Hpux,
        0x02 => ElfTargetSystemAbi::NetBsd,
        0x03 => ElfTargetSystemAbi::Linux,
        0x04 => ElfTargetSystemAbi::GnuHurd,
        0x06 => ElfTargetSystemAbi::Solaris,
        0x07 => ElfTargetSystemAbi::AixMonterey,
        0x08 => ElfTargetSystemAbi::Irix,
        0x09 => ElfTargetSystemAbi::FreeBsd,
        0x0A => ElfTargetSystemAbi::Tru64,
        0x0B => ElfTargetSystemAbi::NovellModesto,
        0x0C => ElfTargetSystemAbi::OpenBsd,
        0x0D => ElfTargetSystemAbi::OpenVms,
        0x0E => ElfTargetSystemAbi::NonStopKernel,
        0x0F => ElfTargetSystemAbi::Aros,
        0x10 => ElfTargetSystemAbi::FenixOs,
        0x11 => ElfTargetSystemAbi::NuxiCloudAbi,
        0x12 => ElfTargetSystemAbi::StratusTechnologiesOpenVos,
        _ => return Err("Unsupported platform!".into()),
    };
    *pointer += 1;
    Ok(t_abi)
}

pub fn parse_elf_header_version(
    pointer: &mut usize,
    content: &[u8],
) -> Result<ElfHeaderVersion, String> {
    let v = content[*pointer];
    *pointer += 1;

    Ok(ElfHeaderVersion(v))
}

pub fn parse_endianness(pointer: &mut usize, content: &[u8]) -> Result<ElfEndianness, String> {
    let end = match content[*pointer] {
        1u8 => ElfEndianness::Little,
        2u8 => ElfEndianness::Big,
        _ => return Err("Invalid endianness!".into()),
    };

    *pointer += 1;

    Ok(end)
}

pub fn parse_platform_type(pointer: &mut usize, content: &[u8]) -> Result<ElfPlatformType, String> {
    let p_type = match content[*pointer] {
        1u8 => ElfPlatformType::Bit32,
        2u8 => ElfPlatformType::Bit64,
        _ => return Err("Invalid platform type".into()),
    };
    *pointer += 1;

    Ok(p_type)
}

pub fn parse_magic_number(pointer: &mut usize, content: &[u8]) -> Result<ElfMagicNumber, String> {
    let magic_number = [
        content[*pointer],
        content[*pointer + 1],
        content[*pointer + 2],
        content[*pointer + 3],
    ];
    *pointer += 4;
    let val_magic = [0x7f, 0x45, 0x4c, 0x46];

    if magic_number != val_magic {
        return Err("Unsupported file type".into());
    }

    Ok(ElfMagicNumber(magic_number))
}

pub fn parse_header(pointer: &mut usize, content: &[u8]) -> Result<ElfHeader, String> {
    let magic_number = parse_magic_number(pointer, content)?;
    let platform_type = parse_platform_type(pointer, content)?;
    let endianness = parse_endianness(pointer, content)?;
    let elf_header_version = parse_elf_header_version(pointer, content)?;
    let target_system_abi = parse_target_system_abi(pointer, content)?;
    let target_abi_version = parse_target_abi_version(pointer, content)?;
    let _reserved_padding = parse_reserved_padding(pointer, content)?;
    let object_file_type = parse_object_file_type(pointer, content, &endianness)?;
    let instruction_set = parse_instruction_set(pointer, content, &endianness)?;
    let elf_version = parse_elf_version(pointer, content, &endianness)?;
    let entry_point = parse_entry_point(pointer, content, &platform_type, &endianness)?;
    let program_header_offset =
        parse_program_header_offset(pointer, content, &platform_type, &endianness)?;
    let section_header_offset =
        parse_section_header_offset(pointer, content, &platform_type, &endianness)?;
    let flags = parse_flags(pointer, content, &endianness)?;
    let header_size = parse_header_size(pointer, content, &endianness)?;
    let program_header_entry_size = parse_program_header_entry_size(pointer, content, &endianness)?;
    let program_header_entry_count =
        parse_program_header_entry_count(pointer, content, &endianness)?;
    let section_header_entry_size = parse_section_header_entry_size(pointer, content, &endianness)?;
    let section_header_entry_count =
        parse_section_header_entry_count(pointer, content, &endianness)?;
    let section_header_sections_table_index =
        parse_section_header_sections_table_index(pointer, content, &endianness)?;

    Ok(ElfHeader {
        magic_number,
        platform_type,
        endianness,
        elf_header_version,
        target_system_abi,
        target_abi_version,
        object_file_type,
        instruction_set,
        elf_version,
        entry_point,
        program_header_offset,
        section_header_offset,
        flags,
        header_size,
        program_header_entry_size,
        program_header_entry_count,
        section_header_entry_size,
        section_header_entry_count,
        section_header_sections_table_index,
    })
}

pub fn read_file(filepath: &Path) -> Result<Vec<u8>, String> {
    let mut file = File::options()
        .read(true)
        .open(filepath)
        .map_err(|err| err.to_string())?;
    let mut buf = Vec::new();
    let read = file.read_to_end(&mut buf).map_err(|err| err.to_string())?;
    if read == 0 {
        return Err("File is empty!".into());
    }

    buf.shrink_to_fit();
    Ok(buf)
}

#[derive(Debug)]
pub struct ElfProgramHeader {
    pub inner: Vec<ElfProgramHeaderEntry>,
}

impl std::fmt::Display for ElfProgramHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl Displayable for ElfProgramHeader {
    fn to_table_rows(&self) -> Vec<Row> {
        self.inner
            .iter()
            .enumerate()
            .map(|(i, entry)| Row {
                field: (i + 1).to_string(),
                value: FieldValue::List(
                    entry
                        .to_string()
                        .lines()
                        .map(|s| FieldValue::Single(s.to_string()))
                        .collect::<Vec<_>>(),
                ),
            })
            .collect::<Vec<Row>>()
    }
}

#[derive(Debug)]
pub struct ElfProgramHeaderEntry {
    pub segment_type: ElfSegmentType,
    pub segment_flags: ElfSegmentFlags,
    pub segment_offset: ElfSegmentOffset,
    pub segment_vaddr: ElfSegmentVAddr,
    pub segment_paddr: ElfSegmentPAddr,
    pub segment_file_size: ElfSegmentFileSize,
    pub segment_memory_size: ElfSegmentMemorySize,
    pub segment_allignment: ElfSegmentAllignment,
}

impl std::fmt::Display for ElfProgramHeaderEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let segment_type = format!("SEG_TYPE: {}", self.segment_type);
        let segment_flags = format!("SEG_FLAGS: {}", self.segment_flags);
        let segment_offset = format!("SEG_OFFSET: {}", self.segment_offset);
        let segment_vaddr = format!("SEG_VADDR: {}", self.segment_vaddr);
        let segment_paddr = format!("SEG_PADDR: {}", self.segment_paddr);
        let segment_file_size = format!("SEG_FILE_SIZE: {}", self.segment_file_size);
        let segment_memory_size = format!("SEG_MEM_SIZE: {}", self.segment_memory_size);
        let segment_allignment = format!("SEG_ALLIGN: {}", self.segment_allignment);

        let txt = format!(
            "{segment_type}\n{segment_flags}\n{segment_offset}\n{segment_vaddr}\n{segment_paddr}\n{segment_file_size}\n{segment_memory_size}\n{segment_allignment}"
        );
        write!(f, "{}", txt)
    }
}

#[derive(Debug)]
pub struct ElfSegmentAllignment(usize);

impl std::fmt::Display for ElfSegmentAllignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSegmentMemorySize(usize);

impl std::fmt::Display for ElfSegmentMemorySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSegmentFileSize(usize);

impl std::fmt::Display for ElfSegmentFileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSegmentPAddr(usize);

impl std::fmt::Display for ElfSegmentPAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSegmentVAddr(usize);

impl std::fmt::Display for ElfSegmentVAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSegmentOffset(usize);

impl std::fmt::Display for ElfSegmentOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default)]
pub enum ElfSegmentFlags {
    PfX,
    PfW,
    #[default]
    PfR,
    PfUnknown,
}

impl std::fmt::Display for ElfSegmentFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            ElfSegmentFlags::PfX => "PF_X",
            ElfSegmentFlags::PfW => "PF_W",
            ElfSegmentFlags::PfR => "PF_R",
            ElfSegmentFlags::PfUnknown => "PF_UNKNOWN",
        };

        write!(f, "{}", txt)
    }
}

#[derive(Debug)]
pub enum ElfSegmentType {
    PtNull,
    PtLoad,
    PtDynamic,
    PtInterp,
    PtNote,
    PtShlib,
    PtPhdr,
    PtTls,
    PtLoos,
    PtHios,
    PtHiproc,
    PtLoproc,
    PtUnknown,
}

impl std::fmt::Display for ElfSegmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            ElfSegmentType::PtNull => "PT_NULL",
            ElfSegmentType::PtLoad => "PT_LOAD",
            ElfSegmentType::PtDynamic => "PT_DYNAMIC",
            ElfSegmentType::PtInterp => "PT_INTERP",
            ElfSegmentType::PtNote => "PT_NOTE",
            ElfSegmentType::PtShlib => "PT_SHLIB",
            ElfSegmentType::PtPhdr => "PT_PHDR",
            ElfSegmentType::PtTls => "PT_TLS",
            ElfSegmentType::PtLoos => "PT_LOOS",
            ElfSegmentType::PtHios => "PT_HIOS",
            ElfSegmentType::PtLoproc => "PT_LOPROC",
            ElfSegmentType::PtHiproc => "PT_HIPROC",
            ElfSegmentType::PtUnknown => "PT_UNKNOWN",
        };
        write!(f, "{}", txt)
    }
}

pub fn parse_segment_usize_t(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<usize, String> {
    let usize_t = match platform {
        ElfPlatformType::Bit32 => {
            let bytes = [
                content[*pointer],
                content[*pointer + 1],
                content[*pointer + 2],
                content[*pointer + 3],
            ];
            *pointer += 4;

            let usize_t = endian.u32_from(&bytes);
            usize_t as usize
        }
        ElfPlatformType::Bit64 => {
            let bytes = [
                content[*pointer],
                content[*pointer + 1],
                content[*pointer + 2],
                content[*pointer + 3],
                content[*pointer + 4],
                content[*pointer + 5],
                content[*pointer + 6],
                content[*pointer + 7],
            ];
            *pointer += 8;

            let usize_t = endian.u64_from(&bytes);
            usize_t as usize
        }
    };

    Ok(usize_t)
}

pub fn parse_segment_allignment(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSegmentAllignment, String> {
    let allign = parse_segment_usize_t(pointer, content, endian, platform)?;

    Ok(ElfSegmentAllignment(allign))
}

pub fn parse_segment_memory_size(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSegmentMemorySize, String> {
    let size = parse_segment_usize_t(pointer, content, endian, platform)?;

    Ok(ElfSegmentMemorySize(size))
}

pub fn parse_segment_file_size(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSegmentFileSize, String> {
    let size = parse_segment_usize_t(pointer, content, endian, platform)?;

    Ok(ElfSegmentFileSize(size))
}

pub fn parse_segment_paddr(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSegmentPAddr, String> {
    let vaddr = parse_segment_usize_t(pointer, content, endian, platform)?;

    Ok(ElfSegmentPAddr(vaddr))
}

pub fn parse_segment_vaddr(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSegmentVAddr, String> {
    let vaddr = parse_segment_usize_t(pointer, content, endian, platform)?;

    Ok(ElfSegmentVAddr(vaddr))
}

pub fn parse_segment_offset(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSegmentOffset, String> {
    let offset = parse_segment_usize_t(pointer, content, endian, platform)?;

    Ok(ElfSegmentOffset(offset))
}

pub fn parse_segment_flags(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfSegmentFlags, String> {
    let bytes = [
        content[*pointer],
        content[*pointer + 1],
        content[*pointer + 2],
        content[*pointer + 3],
    ];

    let flags = endian.u32_from(&bytes);
    let flags = match flags {
        0x1 => ElfSegmentFlags::PfX,
        0x2 => ElfSegmentFlags::PfW,
        0x4 => ElfSegmentFlags::PfR,
        _ => ElfSegmentFlags::PfUnknown,
        // other => return Err(format!("Unsupported Program Flags: {other}")),
    };
    *pointer += 4;

    Ok(flags)
}

pub fn parse_segment_type(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfSegmentType, String> {
    let bytes = [
        content[*pointer],
        content[*pointer + 1],
        content[*pointer + 2],
        content[*pointer + 3],
    ];
    let p_type = endian.u32_from(&bytes);
    let p_type = match p_type {
        0x00000000 => ElfSegmentType::PtNull,
        0x00000001 => ElfSegmentType::PtLoad,
        0x00000002 => ElfSegmentType::PtDynamic,
        0x00000003 => ElfSegmentType::PtInterp,
        0x00000004 => ElfSegmentType::PtNote,
        0x00000005 => ElfSegmentType::PtShlib,
        0x00000006 => ElfSegmentType::PtPhdr,
        0x00000007 => ElfSegmentType::PtTls,
        0x60000000 => ElfSegmentType::PtLoos,
        0x6FFFFFFF => ElfSegmentType::PtHios,
        0x70000000 => ElfSegmentType::PtLoproc,
        0x7FFFFFFF => ElfSegmentType::PtHiproc,
        _ => ElfSegmentType::PtUnknown,
        // other => return Err(format!("Unsupported Program type: {other:x}")),
    };

    *pointer += 4;
    Ok(p_type)
}

pub fn parse_program_header_entry(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfProgramHeaderEntry, String> {
    let segment_type = parse_segment_type(pointer, content, endian)?;
    let mut segment_flags = ElfSegmentFlags::default();
    if let ElfPlatformType::Bit64 = platform {
        segment_flags = parse_segment_flags(pointer, content, endian)?;
    }
    let segment_offset = parse_segment_offset(pointer, content, endian, platform)?;
    let segment_vaddr = parse_segment_vaddr(pointer, content, endian, platform)?;
    let segment_paddr = parse_segment_paddr(pointer, content, endian, platform)?;
    let segment_file_size = parse_segment_file_size(pointer, content, endian, platform)?;
    let segment_memory_size = parse_segment_memory_size(pointer, content, endian, platform)?;
    // These flags appear in a diffent offset depending on the target platform type
    // for allignment reasons
    if let ElfPlatformType::Bit32 = platform {
        segment_flags = parse_segment_flags(pointer, content, endian)?;
    }
    let segment_allignment = parse_segment_allignment(pointer, content, endian, platform)?;

    Ok(ElfProgramHeaderEntry {
        segment_type,
        segment_flags,
        segment_offset,
        segment_vaddr,
        segment_paddr,
        segment_file_size,
        segment_memory_size,
        segment_allignment,
    })
}

pub fn parse_program_header(
    pointer: &mut usize,
    content: &[u8],
    prog_header_entry_count: &ElfProgramHeaderEntryCount,
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfProgramHeader, String> {
    let entry_count = prog_header_entry_count.inner() as usize;
    let mut inner = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        match parse_program_header_entry(pointer, content, endian, platform) {
            Ok(entry) => inner.push(entry),
            Err(err) => eprintln!("{err}"),
        }
    }

    Ok(ElfProgramHeader { inner })
}

// TODO: Figure out a better way to display nested table rows
#[derive(Tabled)]
pub struct Row {
    field: String,
    value: FieldValue,
}

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = format!("{}\n{}", self.field, self.value);
        write!(f, "{}", txt)
    }
}

pub enum FieldValue {
    Single(String),
    List(Vec<FieldValue>),
}

impl std::fmt::Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            // TODO: Work around this clone
            FieldValue::Single(val) => val.clone(),
            FieldValue::List(field_values) => field_values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        };

        write!(f, "{}", txt)
    }
}

pub trait Displayable {
    fn to_table_rows(&self) -> Vec<Row>;
}

impl Displayable for ElfHeader {
    fn to_table_rows(&self) -> Vec<Row> {
        vec![
            Row {
                field: "MAGIC".to_string(),
                value: FieldValue::Single(format!("{:x?}", self.magic_number.0)),
            },
            Row {
                field: "PLATFORM".to_string(),
                value: FieldValue::Single(self.platform_type.to_string()),
            },
            Row {
                field: "ENDIANNESS".to_string(),
                value: FieldValue::Single(self.endianness.to_string()),
            },
            Row {
                field: "ELF_VERSION".to_string(),
                value: FieldValue::Single(self.elf_header_version.to_string()),
            },
            Row {
                field: "TARGET_SYS_ABI".to_string(),
                value: FieldValue::Single(self.target_system_abi.to_string()),
            },
            Row {
                field: "OBJECT_FILE_TYPE".to_string(),
                value: FieldValue::Single(self.object_file_type.to_string()),
            },
            Row {
                field: "INSTRUCTION_SET".to_string(),
                value: FieldValue::Single(self.instruction_set.to_string()),
            },
            Row {
                field: "E_VERSION".to_string(),
                value: FieldValue::Single(self.elf_version.to_string()),
            },
            Row {
                field: "ENTRY_POINT".to_string(),
                value: FieldValue::Single(self.entry_point.to_string()),
            },
            Row {
                field: "PROGRAM_HDR_OFFSET".to_string(),
                value: FieldValue::Single(self.program_header_offset.to_string()),
            },
            Row {
                field: "SECTION_HDR_OFFSET".to_string(),
                value: FieldValue::Single(self.section_header_offset.to_string()),
            },
            Row {
                field: "FLAGS".to_string(),
                value: FieldValue::Single(self.flags.to_string()),
            },
            Row {
                field: "HEADER_SIZE".to_string(),
                value: FieldValue::Single(self.header_size.to_string()),
            },
            Row {
                field: "PROG_HDR_ENTRY_SIZE".to_string(),
                value: FieldValue::Single(self.program_header_entry_size.to_string()),
            },
            Row {
                field: "PROG_HDR_ENTRY_COUNT".to_string(),
                value: FieldValue::Single(self.program_header_entry_count.to_string()),
            },
            Row {
                field: "SECTION_HDR_ENTRY_SIZE".to_string(),
                value: FieldValue::Single(self.section_header_entry_size.to_string()),
            },
            Row {
                field: "SECTION_HDR_ENTRY_COUNT".to_string(),
                value: FieldValue::Single(self.section_header_entry_count.to_string()),
            },
            Row {
                field: "SECTION_HDR_SECTIONS_TABLE_IDX".to_string(),
                value: FieldValue::Single(self.section_header_sections_table_index.to_string()),
            },
        ]
    }
}

pub fn pretty_display<T>(part: &T)
where
    T: Displayable,
{
    let table = Table::new(part.to_table_rows());
    println!("{}", table);
}

pub fn parse_file(args: &Cli) -> Result<(), String> {
    let content = read_file(&args.filepath)?;
    let mut pointer = 0x0usize;
    let header = parse_header(&mut pointer, &content)?;
    match args.to_process {
        ElfParts::Header => {
            pretty_display(&header);
        }
        ElfParts::ProgramHeader => {
            let program_header = parse_program_header(
                &mut pointer,
                &content,
                &header.program_header_entry_count,
                &header.endianness,
                &header.platform_type,
            )?;

            // println!("{:?}", program_header);
            pretty_display(&program_header);
        }
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let args = Cli::parse(std::env::args().skip(1))?;
    parse_file(&args)?;
    Ok(())
}
