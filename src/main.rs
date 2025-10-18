// An ELF executable file format parser
// References: https://en.wikipedia.org/wiki/Executable_and_Linkable_Format

use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use tabled::{Table, Tabled};

#[derive(Debug, Default, PartialEq)]
pub struct Cli {
    pub filepath: PathBuf,
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
            }
        }

        if cli == Cli::default() {
            return Err("Missing args!".into());
        }

        Ok(cli)
    }

    fn helper() {
        println!("Help Information:");
    }
}

#[derive(Debug, Tabled)]
pub struct ElfHeader {
    pub magic_number: ElfMagicNumber,
    pub platform_type: ElfPlatformType,
    pub endianness: ElfEndianness,
    pub elf_version: ElfVersion,
    pub target_system_abi: ElfTargetSystemAbi,
    pub target_abi_version: ElfTargetAbiVersion,
    pub object_file_type: ElfObjectFileType,
    pub instruction_set: ElfInstructionSet,
    pub e_version: EVersion,
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

#[derive(Debug, Tabled)]
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

impl ElfHeader {
    pub fn to_table_rows(&self) -> Vec<ElfHeaderRow> {
        vec![
            ElfHeaderRow {
                field: "MAGIC".to_string(),
                value: format!("{:x?}", self.magic_number.0),
            },
            ElfHeaderRow {
                field: "PLATFORM".to_string(),
                value: self.platform_type.to_string(),
            },
            ElfHeaderRow {
                field: "ENDIANNESS".to_string(),
                value: self.endianness.to_string(),
            },
            ElfHeaderRow {
                field: "ELF_VERSION".to_string(),
                value: self.elf_version.to_string(),
            },
            ElfHeaderRow {
                field: "TARGET_SYS_ABI".to_string(),
                value: self.target_system_abi.to_string(),
            },
            ElfHeaderRow {
                field: "OBJECT_FILE_TYPE".to_string(),
                value: self.object_file_type.to_string(),
            },
            ElfHeaderRow {
                field: "INSTRUCTION_SET".to_string(),
                value: self.instruction_set.to_string(),
            },
            ElfHeaderRow {
                field: "E_VERSION".to_string(),
                value: self.e_version.to_string(),
            },
            ElfHeaderRow {
                field: "ENTRY_POINT".to_string(),
                value: self.entry_point.to_string(),
            },
            ElfHeaderRow {
                field: "PROGRAM_HDR_OFFSET".to_string(),
                value: self.program_header_offset.to_string(),
            },
            ElfHeaderRow {
                field: "SECTION_HDR_OFFSET".to_string(),
                value: self.section_header_offset.to_string(),
            },
            ElfHeaderRow {
                field: "FLAGS".to_string(),
                value: self.flags.to_string(),
            },
            ElfHeaderRow {
                field: "HEADER_SIZE".to_string(),
                value: self.header_size.to_string(),
            },
            ElfHeaderRow {
                field: "PROG_HDR_ENTRY_SIZE".to_string(),
                value: self.program_header_entry_size.to_string(),
            },
            ElfHeaderRow {
                field: "PROG_HDR_ENTRY_COUNT".to_string(),
                value: self.program_header_entry_count.to_string(),
            },
            ElfHeaderRow {
                field: "SECTION_HDR_ENTRY_SIZE".to_string(),
                value: self.section_header_entry_size.to_string(),
            },
            ElfHeaderRow {
                field: "SECTION_HDR_ENTRY_COUNT".to_string(),
                value: self.section_header_entry_count.to_string(),
            },
            ElfHeaderRow {
                field: "SECTION_HDR_SECTIONS_TABLE_IDX".to_string(),
                value: self.section_header_sections_table_index.to_string(),
            },
        ]
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
pub enum ElfSectionHeaderOffset {
    Offset32(u32),
    Offsetu64(u64),
}

impl std::fmt::Display for ElfSectionHeaderOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElfSectionHeaderOffset::Offset32(val) => write!(f, "{:x}", val),
            ElfSectionHeaderOffset::Offsetu64(val) => write!(f, "{:x}", val),
        }
    }
}

#[derive(Debug)]
pub enum ElfProgramHeaderOffset {
    Offset32(u32),
    Offsetu64(u64),
}

impl std::fmt::Display for ElfProgramHeaderOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElfProgramHeaderOffset::Offset32(val) => write!(f, "{:x}", val),
            ElfProgramHeaderOffset::Offsetu64(val) => write!(f, "{:x}", val),
        }
    }
}

#[derive(Debug)]
pub enum ElfEntryPoint {
    EntryPoint32(u32),
    EntryPoint64(u64),
}

impl std::fmt::Display for ElfEntryPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElfEntryPoint::EntryPoint32(val) => write!(f, "{:x}", val),
            ElfEntryPoint::EntryPoint64(val) => write!(f, "{:x}", val),
        }
    }
}

#[derive(Debug)]
pub struct EVersion(u32);

impl std::fmt::Display for EVersion {
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
    HPUX,
    NetBSD,
    Linux,
    GNUHurd,
    Solaris,
    AIXMonterey,
    IRIX,
    FreeBSD,
    Tru64,
    NovellModesto,
    OpenBSD,
    OpenVMS,
    NonStopKernel,
    AROS,
    FenixOS,
    NuxiCloudABI,
    StratusTechnologiesOpenVOS,
}

impl std::fmt::Display for ElfTargetSystemAbi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            ElfTargetSystemAbi::SystemV => "System V",
            ElfTargetSystemAbi::HPUX => "HP-UX",
            ElfTargetSystemAbi::NetBSD => "NetBSD",
            ElfTargetSystemAbi::Linux => "Linux",
            ElfTargetSystemAbi::GNUHurd => "GNU Hurd",
            ElfTargetSystemAbi::Solaris => "Solaris",
            ElfTargetSystemAbi::AIXMonterey => "AIX (Monterey)",
            ElfTargetSystemAbi::IRIX => "IRIX",
            ElfTargetSystemAbi::FreeBSD => "FreeBSD",
            ElfTargetSystemAbi::Tru64 => "Tru64",
            ElfTargetSystemAbi::NovellModesto => "Novell Modesto",
            ElfTargetSystemAbi::OpenBSD => "OpenBSD",
            ElfTargetSystemAbi::OpenVMS => "OpenVMS",
            ElfTargetSystemAbi::NonStopKernel => "NonStop Kernel",
            ElfTargetSystemAbi::AROS => "AROS",
            ElfTargetSystemAbi::FenixOS => "FenixOS",
            ElfTargetSystemAbi::NuxiCloudABI => "Nuxi CloudABI",
            ElfTargetSystemAbi::StratusTechnologiesOpenVOS => "Stratus Technologies OpenVOS",
        };

        write!(f, "{}", txt)
    }
}

#[derive(Debug)]
pub struct ElfVersion(u8);

impl std::fmt::Display for ElfVersion {
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

#[derive(Debug, Tabled)]
pub struct ElfBinary {
    pub header: ElfHeader,
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
                ElfSectionHeaderOffset::Offset32(offset)
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
                ElfSectionHeaderOffset::Offsetu64(offset)
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
                ElfProgramHeaderOffset::Offset32(offset)
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
                ElfProgramHeaderOffset::Offsetu64(offset)
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
                ElfEntryPoint::EntryPoint32(entry)
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
                ElfEntryPoint::EntryPoint64(entry)
            }
        }
    };

    Ok(entry_point)
}

pub fn parse_e_version(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<EVersion, String> {
    let bytes = [
        content[*pointer],
        content[*pointer + 1],
        content[*pointer + 2],
        content[*pointer + 3],
    ];

    *pointer += 4;

    let e_version = endian.u32_from(&bytes);
    Ok(EVersion(e_version))
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
        0x01 => ElfTargetSystemAbi::HPUX,
        0x02 => ElfTargetSystemAbi::NetBSD,
        0x03 => ElfTargetSystemAbi::Linux,
        0x04 => ElfTargetSystemAbi::GNUHurd,
        0x06 => ElfTargetSystemAbi::Solaris,
        0x07 => ElfTargetSystemAbi::AIXMonterey,
        0x08 => ElfTargetSystemAbi::IRIX,
        0x09 => ElfTargetSystemAbi::FreeBSD,
        0x0A => ElfTargetSystemAbi::Tru64,
        0x0B => ElfTargetSystemAbi::NovellModesto,
        0x0C => ElfTargetSystemAbi::OpenBSD,
        0x0D => ElfTargetSystemAbi::OpenVMS,
        0x0E => ElfTargetSystemAbi::NonStopKernel,
        0x0F => ElfTargetSystemAbi::AROS,
        0x10 => ElfTargetSystemAbi::FenixOS,
        0x11 => ElfTargetSystemAbi::NuxiCloudABI,
        0x12 => ElfTargetSystemAbi::StratusTechnologiesOpenVOS,
        _ => return Err("Unsupported platform!".into()),
    };
    *pointer += 1;
    Ok(t_abi)
}

pub fn parse_elf_version(pointer: &mut usize, content: &[u8]) -> Result<ElfVersion, String> {
    let v = content[*pointer];
    *pointer += 1;

    Ok(ElfVersion(v))
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

pub fn parse_header(content: &[u8]) -> Result<ElfHeader, String> {
    let mut pointer = 0usize;
    let magic_number = parse_magic_number(&mut pointer, content)?;
    let platform_type = parse_platform_type(&mut pointer, content)?;
    let endianness = parse_endianness(&mut pointer, content)?;
    let elf_version = parse_elf_version(&mut pointer, content)?;
    let target_system_abi = parse_target_system_abi(&mut pointer, content)?;
    let target_abi_version = parse_target_abi_version(&mut pointer, content)?;
    let _reserved_padding = parse_reserved_padding(&mut pointer, content)?;
    let object_file_type = parse_object_file_type(&mut pointer, content, &endianness)?;
    let instruction_set = parse_instruction_set(&mut pointer, content, &endianness)?;
    let e_version = parse_e_version(&mut pointer, content, &endianness)?;
    let entry_point = parse_entry_point(&mut pointer, content, &platform_type, &endianness)?;
    let program_header_offset =
        parse_program_header_offset(&mut pointer, content, &platform_type, &endianness)?;
    let section_header_offset =
        parse_section_header_offset(&mut pointer, content, &platform_type, &endianness)?;
    let flags = parse_flags(&mut pointer, content, &endianness)?;
    let header_size = parse_header_size(&mut pointer, content, &endianness)?;
    let program_header_entry_size =
        parse_program_header_entry_size(&mut pointer, content, &endianness)?;
    let program_header_entry_count =
        parse_program_header_entry_count(&mut pointer, content, &endianness)?;
    let section_header_entry_size =
        parse_section_header_entry_size(&mut pointer, content, &endianness)?;
    let section_header_entry_count =
        parse_section_header_entry_count(&mut pointer, content, &endianness)?;
    let section_header_sections_table_index =
        parse_section_header_sections_table_index(&mut pointer, content, &endianness)?;

    Ok(ElfHeader {
        magic_number,
        platform_type,
        endianness,
        elf_version,
        target_system_abi,
        target_abi_version,
        object_file_type,
        instruction_set,
        e_version,
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

pub fn parse_file(args: &Cli) -> Result<ElfBinary, String> {
    let content = read_file(&args.filepath)?;
    let header = parse_header(&content)?;

    Ok(ElfBinary { header })
}

fn main() -> Result<(), String> {
    let args = Cli::parse(std::env::args().skip(1))?;
    let elf: ElfBinary = parse_file(&args)?;
    let tabled_elf = Table::new(elf.header.to_table_rows());
    println!("{}", tabled_elf);
    // println!("{}", elf.header);
    Ok(())
}
