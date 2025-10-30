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
    Data,
    SectionHeader,
    All,
}

impl ElfParts {
    pub fn as_str(&self) -> &'static str {
        match self {
            ElfParts::Header => "Header",
            ElfParts::ProgramHeader => "ProgramHeader",
            ElfParts::Data => "Data",
            ElfParts::SectionHeader => "SectionHeader",
            ElfParts::All => "All",
        }
    }
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
                let next = match args.next() {
                    Some(val) => val,
                    None => return Err("Missing Filepath".to_string()),
                };
                cli.filepath = Path::new(&next).to_path_buf();
            }

            if next == "--help" || next == "-h" {
                Self::helper();
                std::process::exit(0);
            }

            if next == "--header" || next == "-e" {
                cli.to_process = ElfParts::Header;
                return Ok(cli);
            }

            if next == "--program" || next == "-p" {
                cli.to_process = ElfParts::ProgramHeader;
                return Ok(cli);
            }

            if next == "--data" || next == "-d" {
                cli.to_process = ElfParts::Data;
                return Ok(cli);
            }

            if next == "--section" || next == "-s" {
                cli.to_process = ElfParts::SectionHeader;
                return Ok(cli);
            }

            if next == "--all" || next == "-a" {
                cli.to_process = ElfParts::All;
                return Ok(cli);
            }
        }

        if cli == Cli::default() {
            return Err("Missing args!".into());
        }

        Ok(cli)
    }

    fn helper() {
        const USAGE_INFO: &str = r#"
Usage:
    program <flags>
        --help    , -h    Show this information
        --filepath, -f    Path to the elf file
        --header  , -e    Display only the elf header (default)
        --program , -p    Display only the elf program header
        --section , -s    Display only the section header
        --all     , -a    Display all headers
        "#;

        println!("{USAGE_INFO}");
    }
}

#[derive(Debug, Default, Tabled)]
pub struct ElfHeader {
    // 0x7F followed by ELF(45 4c 46) in ASCII;
    pub magic_number: ElfMagicNumber,
    // This byte is set to either 1 or 2 to signify 32- or 64-bit format, respectively.
    pub platform_type: ElfPlatformType,
    // This byte is set to either 1 or 2 to signify little or big endianness, respectively.
    // This affects interpretation of multi-byte fields starting with offset 0x10.
    pub endianness: ElfEndianness,
    // Set to 1 for the original and current version of ELF.
    pub elf_header_version: ElfHeaderVersion,
    // Identifies the target operating system ABI.
    pub target_system_abi: ElfTargetSystemAbi,
    // Further specifies the ABI version. Its interpretation depends on the target ABI.
    // Linux kernel (after at least 2.6) has no definition of it,[6] so it is ignored for
    // statically linked executables. In that case, offset and size of EI_PAD are 8.
    pub target_abi_version: ElfTargetAbiVersion,
    pub object_file_type: ElfObjectFileType,
    // Specifies target instruction set architecture.
    pub instruction_set: ElfInstructionSet,
    // Set to 1 for the original version of ELF.
    pub elf_version: ElfVersion,
    // This is the memory address of the entry point from where the process starts executing.
    pub entry_point: ElfEntryPoint,
    // Points to the start of the program header table.
    pub program_header_offset: ElfProgramHeaderOffset,
    // Points to the start of the section header table
    pub section_header_offset: ElfSectionHeaderOffset,
    pub flags: ElfFlags,
    // Contains the size of this header, normally 64 Bytes for 64-bit
    // and 52 Bytes for 32-bit format.
    pub header_size: ElfHeaderSize,
    // Contains the size of a program header table entry.
    pub program_header_entry_size: ElfProgramHeaderEntrySize,
    // Contains the number of entries in the program header table.
    pub program_header_entry_count: ElfProgramHeaderEntryCount,
    // Contains the size of a section header table entry
    pub section_header_entry_size: ElfSectionHeaderEntrySize,
    // Contains the number of entries in the section header table.
    pub section_header_entry_count: ElfSectionHeaderEntryCount,
    // Contains index of the section header table entry that contains the section names.
    pub section_header_sections_table_index: ElfSectionHeaderSectionsTableIndex,
}

impl std::fmt::Display for ElfHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.magic_number,
            self.platform_type,
            self.endianness,
            self.elf_header_version,
            self.target_system_abi,
            self.target_abi_version,
            self.object_file_type,
            self.instruction_set,
            self.elf_version,
            self.entry_point,
            self.program_header_offset,
            self.section_header_offset,
            self.flags,
            self.header_size,
            self.program_header_entry_size,
            self.program_header_entry_count,
            self.section_header_entry_size,
            self.section_header_entry_count,
            self.section_header_sections_table_index
        );
        write!(f, "{}", txt)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionHeaderSectionsTableIndex(u16);

impl std::fmt::Display for ElfSectionHeaderSectionsTableIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionHeaderEntrySize(u16);

impl std::fmt::Display for ElfSectionHeaderEntrySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionHeaderEntryCount(u16);

impl std::fmt::Display for ElfSectionHeaderEntryCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
pub struct ElfProgramHeaderEntrySize(u16);

impl std::fmt::Display for ElfProgramHeaderEntrySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfHeaderSize(u16);

impl std::fmt::Display for ElfHeaderSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfFlags(u32);

impl std::fmt::Display for ElfFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionHeaderOffset(usize);

impl std::fmt::Display for ElfSectionHeaderOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfProgramHeaderOffset(usize);

impl std::fmt::Display for ElfProgramHeaderOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfEntryPoint(usize);

impl std::fmt::Display for ElfEntryPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Default, Debug)]
pub struct ElfVersion(u32);

impl std::fmt::Display for ElfVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Default, Debug)]
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
    #[default]
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

#[derive(Default, Debug)]
pub enum ElfObjectFileType {
    #[default]
    EtNone, //Unknown.
    EtRel,    //Relocatable file.
    EtExec,   //Executable file.
    EtDyn,    //Shared object.
    EtCore,   //Core file.
    EtLoos,   //Reserved inclusive range. Operating system specific.
    EtHios,   //
    EtLoproc, //Reserved inclusive range. Processor specific.
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
            ElfObjectFileType::EtHios => "ET_HIOS",
            ElfObjectFileType::EtLoproc => "ET_LOPROC",
            ElfObjectFileType::EtHiproc => "ET_HIPROC",
        };

        write!(f, "{}", txt)
    }
}

#[derive(Debug, Default)]
pub struct ElfReservedPadding([u8; 7]);

impl std::fmt::Display for ElfReservedPadding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfTargetAbiVersion(u8);

impl std::fmt::Display for ElfTargetAbiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Default, Debug)]
pub enum ElfTargetSystemAbi {
    #[default]
    Unknown,
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
            ElfTargetSystemAbi::Unknown => "UNKNOWN",
        };

        write!(f, "{}", txt)
    }
}

#[derive(Debug, Default)]
pub struct ElfHeaderVersion(u8);

impl std::fmt::Display for ElfHeaderVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug, Default)]
pub enum ElfPlatformType {
    #[default]
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

#[derive(Default, Debug)]
pub enum ElfEndianness {
    #[default]
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

#[derive(Debug, Default)]
pub struct ElfMagicNumber(String);

impl std::fmt::Display for ElfMagicNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
        0x0B..=0x0E => ElfInstructionSet::Reserved,
        0x0F => ElfInstructionSet::HewlettPackardPaRisc,
        0x13 => ElfInstructionSet::Intel80960,
        0x14 => ElfInstructionSet::PowerPc,
        0x15 => ElfInstructionSet::PowerPc64bit,
        0x16 => ElfInstructionSet::S390,
        0x17 => ElfInstructionSet::IbmSpuSpc,
        0x18..=0x23 => ElfInstructionSet::Reserved,
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
        0xFEFF => ElfObjectFileType::EtHios,
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

    let magic_number = String::from_utf8_lossy(&magic_number).to_string();

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

// This is an array of N (given in the `ElfHeader`) entries
#[derive(Debug, Default)]
pub struct ElfProgramHeader {
    pub inner: Vec<ElfProgramHeaderEntry>,
}

impl ElfProgramHeader {
    pub fn inner(self) -> Vec<ElfProgramHeaderEntry> {
        self.inner
    }
}

impl std::fmt::Display for ElfProgramHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

#[derive(Debug, Tabled)]
pub struct ElfProgramHeaderEntry {
    pub segment_type: ElfSegmentType,
    pub segment_flags: ElfSegmentFlags,
    // Offset of the segment in the file image.
    pub segment_offset: ElfSegmentOffset,
    // Virtual address of the segment in memory.
    pub segment_vaddr: ElfSegmentVAddr,
    // On systems where physical address is relevant, reserved for segment's physical address.
    pub segment_paddr: ElfSegmentPAddr,
    pub segment_file_size: ElfSegmentFileSize,
    // Size in bytes of the segment in memory. May be 0.
    pub segment_memory_size: ElfSegmentMemorySize,
    // 0 and 1 specify no alignment. Otherwise should be a positive, integral power of 2,
    // with p_vaddr equating p_offset modulus p_align.
    pub segment_allignment: ElfSegmentAllignment,
}

impl std::fmt::Display for ElfProgramHeaderEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.segment_type,
            self.segment_flags,
            self.segment_offset,
            self.segment_vaddr,
            self.segment_paddr,
            self.segment_file_size,
            self.segment_memory_size,
            self.segment_allignment
        );
        write!(f, "{}", txt)
    }
}

#[derive(Debug)]
pub struct ElfSegmentAllignment(usize);

impl std::fmt::Display for ElfSegmentAllignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSegmentMemorySize(usize);

impl std::fmt::Display for ElfSegmentMemorySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSegmentFileSize(usize);

impl std::fmt::Display for ElfSegmentFileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSegmentPAddr(usize);

impl std::fmt::Display for ElfSegmentPAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSegmentVAddr(usize);

impl std::fmt::Display for ElfSegmentVAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug)]
pub struct ElfSegmentOffset(usize);

impl std::fmt::Display for ElfSegmentOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug, Default)]
pub enum ElfSegmentFlags {
    PfX, // Executable segment
    PfW, // Writeable segment
    PfR, // readable segment
    #[default]
    PfUnknown, // Unknown flag to me
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
    PtNull,    //Program header table entry unused.
    PtLoad,    //Loadable segment.
    PtDynamic, //Dynamic linking information.
    PtInterp,  //Interpreter information.
    PtNote,    //Auxiliary information.
    PtShlib,   //Reserved.
    PtPhdr,    //Segment containing program header table itself.
    PtTls,     //Thread-Local Storage template.
    PtLoos,    //Reserved inclusive range. Operating system specific.
    PtHios,    //
    PtLoproc,  //Reserved inclusive range. Processor specific.
    PtHiproc,  //
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

#[derive(Debug, Default)]
pub struct ElfSectionHeader {
    pub inner: Vec<ElfSectionHeaderEntry>,
}

impl ElfSectionHeader {
    pub fn inner(self) -> Vec<ElfSectionHeaderEntry> {
        self.inner
    }
}

impl std::fmt::Display for ElfSectionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

#[derive(Debug, Default, Tabled)]
pub struct ElfSectionHeaderEntry {
    // An offset to a string in the .shstrtab section that
    // represents the name of this section.
    pub section_name_offset: ElfSectionNameOffset,
    // The actual name
    pub section_name: ElfSectionName,
    // Identifies the type of this header.
    pub section_header_type: ElfSectionHeaderType,
    // Identifies the attributes of the section.
    pub section_flags: ElfSectionFlags,
    // Virtual address of the section in memory, for sections
    // that are loaded.
    pub section_addr: ElfSectionAddr,
    // Offset of the section in the file image
    pub section_offset: ElfSectionOffset,
    // Size in bytes of the section. May be 0.
    pub section_size: ElfSectionSize,
    // Contains the section index of an associated section.
    // This field is used for several purposes, depending on
    // the type of section.
    pub section_link: ElfSectionLink,
    // Contains extra information about the section.
    // This field is used for several purposes, depending on
    // the type of section.
    pub section_info: ElfSectionInfo,
    // Contains the required alignment of the section.
    // This field must be a power of two.
    pub section_addr_allign: ElfSectionAddrAllign,
    // Contains the size, in bytes, of each entry, for sections
    // that contain fixed-size entries. Otherwise, this field contains zero.
    pub section_entry_size: ElfSectionEntrySize,
}

impl std::fmt::Display for ElfSectionHeaderEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.section_name_offset,
            self.section_name,
            self.section_header_type,
            self.section_flags,
            self.section_addr,
            self.section_offset,
            self.section_size,
            self.section_link,
            self.section_info,
            self.section_addr_allign,
            self.section_entry_size
        );
        write!(f, "{}", txt)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionName(String);

impl std::fmt::Display for ElfSectionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionAddrAllign(usize);

impl std::fmt::Display for ElfSectionAddrAllign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionEntrySize(usize);

impl std::fmt::Display for ElfSectionEntrySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionLink(u32);

impl std::fmt::Display for ElfSectionLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionInfo(u32);

impl std::fmt::Display for ElfSectionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Default, Debug)]
pub struct ElfSectionAddr(usize);

impl std::fmt::Display for ElfSectionAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionOffset(usize);

impl std::fmt::Display for ElfSectionOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionSize(usize);

impl std::fmt::Display for ElfSectionSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Default, Debug)]
pub enum ElfSectionFlags {
    ShfWrite,           // Writable
    ShfAlloc,           // Occupies memory during execution
    ShfExecinstr,       // Executable
    ShfMerge,           // Might be merged
    ShfStrings,         // Contains null-terminated strings
    ShfInfoLink,        // sh_info' contains SHT index
    ShfLinkOrder,       // Preserve order after combining
    ShfOsNonconforming, // Non-standard OS specific handling required
    ShfGroup,           // Section is member of a group
    ShfTls,             // Section hold thread-local data
    ShfMaskos,          // OS-specific
    ShfMaskproc,        // Processor-specific
    ShfOrdered,         // Special ordering requirement (Solaris)
    ShfExclude,         // Section is excluded unless referenced or allocated (Solaris)
    #[default]
    ShfNull, //
}

impl std::fmt::Display for ElfSectionFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            ElfSectionFlags::ShfWrite => "SHF_WRITE",
            ElfSectionFlags::ShfAlloc => "SHF_ALLOC",
            ElfSectionFlags::ShfExecinstr => "SHF_EXECINSTR",
            ElfSectionFlags::ShfMerge => "SHF_MERGE",
            ElfSectionFlags::ShfStrings => "SHF_STRINGS",
            ElfSectionFlags::ShfInfoLink => "SHF_INFO_LINK",
            ElfSectionFlags::ShfLinkOrder => "SHF_LINK_ORDER",
            ElfSectionFlags::ShfOsNonconforming => "SHF_OS_NONCONFORMING",
            ElfSectionFlags::ShfGroup => "SHF_GROUP",
            ElfSectionFlags::ShfTls => "SHF_TLS",
            ElfSectionFlags::ShfMaskos => "SHF_MASKOS",
            ElfSectionFlags::ShfMaskproc => "SHF_MASKPROC",
            ElfSectionFlags::ShfOrdered => "SHF_ORDERED",
            ElfSectionFlags::ShfExclude => "SHF_EXCLUDE",
            ElfSectionFlags::ShfNull => "SHF_NULL",
        };

        write!(f, "{}", txt)
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum ElfSectionHeaderType {
    #[default]
    ShtNull, //Section header table entry unused
    ShtProgbits,     //Program data
    ShtSymtab,       //Symbol table
    ShtStrtab,       //String table
    ShtRela,         //Relocation entries with addends
    ShtHash,         //Symbol hash table
    ShtDynamic,      //Dynamic linking information
    ShtNote,         //Notes
    ShtNobits,       //Program space with no data (bss)
    ShtRel,          //Relocation entries, no addends
    ShtShlib,        //Reserved
    ShtDynsym,       //Dynamic linker symbol table
    ShtInitArray,    //Array of constructors
    ShtFiniArray,    //Array of destructors
    ShtPreinitArray, //Array of pre-constructors
    ShtGroup,        //Section group
    ShtSymtabShndx,  //Extended section indices
    ShtNum,          //Number of defined types.
    ShtLoos,         //Start OS-specific.
}

impl std::fmt::Display for ElfSectionHeaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            ElfSectionHeaderType::ShtNull => "SHT_NULL",
            ElfSectionHeaderType::ShtProgbits => "SHT_PROGBITS",
            ElfSectionHeaderType::ShtSymtab => "SHT_SYMTAB",
            ElfSectionHeaderType::ShtStrtab => "SHT_STRTAB ",
            ElfSectionHeaderType::ShtRela => "SHT_RELA",
            ElfSectionHeaderType::ShtHash => "SHT_HASH",
            ElfSectionHeaderType::ShtDynamic => "SHT_DYNAMIC",
            ElfSectionHeaderType::ShtNote => "SHT_NOTE",
            ElfSectionHeaderType::ShtNobits => "SHT_NOBITS",
            ElfSectionHeaderType::ShtRel => "SHT_REL",
            ElfSectionHeaderType::ShtShlib => "SHT_SHLIB",
            ElfSectionHeaderType::ShtDynsym => "SHT_DYNSYM",
            ElfSectionHeaderType::ShtInitArray => "SHT_INIT_ARRAY",
            ElfSectionHeaderType::ShtFiniArray => "SHT_FINI_ARRAY",
            ElfSectionHeaderType::ShtPreinitArray => "SHT_PREINIT_ARRAY",
            ElfSectionHeaderType::ShtGroup => "SHT_GROUP",
            ElfSectionHeaderType::ShtSymtabShndx => "SHT_SYMTAB_SHNDX",
            ElfSectionHeaderType::ShtNum => "SHT_NUM",
            ElfSectionHeaderType::ShtLoos => "SHT_LOOS",
        };

        write!(f, "{}", txt)
    }
}

#[derive(Debug, Default)]
pub struct ElfSectionNameOffset(u32);

impl std::fmt::Display for ElfSectionNameOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug, Default, Tabled)]
pub struct ElfBinary {
    pub header: ElfHeader,
    pub program_header: ElfProgramHeader,
    pub section_header: ElfSectionHeader,

    #[tabled(skip)]
    pub content: Vec<u8>,
}

impl std::fmt::Display for ElfBinary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n{}\n{}",
            self.header, self.program_header, self.section_header
        )
    }
}

pub fn parse_section_entry_size(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSectionEntrySize, String> {
    let size = parse_segment_usize_t(pointer, content, endian, platform)?;

    Ok(ElfSectionEntrySize(size))
}

pub fn parse_section_addr_allignment(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSectionAddrAllign, String> {
    let allign = parse_segment_usize_t(pointer, content, endian, platform)?;

    Ok(ElfSectionAddrAllign(allign))
}

pub fn parse_section_info(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfSectionInfo, String> {
    let bytes = [
        content[*pointer],
        content[*pointer + 1],
        content[*pointer + 2],
        content[*pointer + 3],
    ];
    *pointer += 4;
    let info = endian.u32_from(&bytes);

    Ok(ElfSectionInfo(info))
}

pub fn parse_section_link(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfSectionLink, String> {
    let bytes = [
        content[*pointer],
        content[*pointer + 1],
        content[*pointer + 2],
        content[*pointer + 3],
    ];
    *pointer += 4;
    let link = endian.u32_from(&bytes);

    Ok(ElfSectionLink(link))
}

pub fn parse_section_size(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSectionSize, String> {
    let size = parse_segment_usize_t(pointer, content, endian, platform)?;

    Ok(ElfSectionSize(size))
}

pub fn parse_section_offset(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSectionOffset, String> {
    let offset = parse_segment_usize_t(pointer, content, endian, platform)?;

    Ok(ElfSectionOffset(offset))
}

pub fn parse_section_addr(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSectionAddr, String> {
    let addr = parse_segment_usize_t(pointer, content, endian, platform)?;

    Ok(ElfSectionAddr(addr))
}

pub fn parse_section_flags(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSectionFlags, String> {
    let flags = parse_segment_usize_t(pointer, content, endian, platform)?;

    let flags = match flags {
        0x1 => ElfSectionFlags::ShfWrite,
        0x2 => ElfSectionFlags::ShfAlloc,
        0x4 => ElfSectionFlags::ShfExecinstr,
        0x10 => ElfSectionFlags::ShfMerge,
        0x20 => ElfSectionFlags::ShfStrings,
        0x40 => ElfSectionFlags::ShfInfoLink,
        0x80 => ElfSectionFlags::ShfLinkOrder,
        0x100 => ElfSectionFlags::ShfOsNonconforming,
        0x200 => ElfSectionFlags::ShfGroup,
        0x400 => ElfSectionFlags::ShfTls,
        0x0FF00000 => ElfSectionFlags::ShfMaskos,
        0xF0000000 => ElfSectionFlags::ShfMaskproc,
        0x4000000 => ElfSectionFlags::ShfOrdered,
        0x8000000 => ElfSectionFlags::ShfExclude,
        _ => ElfSectionFlags::ShfNull,
        // other => return Err(format!("Unsupported section flags: {other}")),
    };

    Ok(flags)
}

pub fn parse_section_header_type(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfSectionHeaderType, String> {
    let bytes = [
        content[*pointer],
        content[*pointer + 1],
        content[*pointer + 2],
        content[*pointer + 3],
    ];
    *pointer += 4;

    let h_type = endian.u32_from(&bytes);
    let h_type = match h_type {
        0x0 => ElfSectionHeaderType::ShtNull,
        0x1 => ElfSectionHeaderType::ShtProgbits,
        0x2 => ElfSectionHeaderType::ShtSymtab,
        0x3 => ElfSectionHeaderType::ShtStrtab,
        0x4 => ElfSectionHeaderType::ShtRela,
        0x5 => ElfSectionHeaderType::ShtHash,
        0x6 => ElfSectionHeaderType::ShtDynamic,
        0x7 => ElfSectionHeaderType::ShtNote,
        0x8 => ElfSectionHeaderType::ShtNobits,
        0x9 => ElfSectionHeaderType::ShtRel,
        0x0A => ElfSectionHeaderType::ShtShlib,
        0x0B => ElfSectionHeaderType::ShtDynsym,
        0x0E => ElfSectionHeaderType::ShtInitArray,
        0x0F => ElfSectionHeaderType::ShtFiniArray,
        0x10 => ElfSectionHeaderType::ShtPreinitArray,
        0x11 => ElfSectionHeaderType::ShtGroup,
        0x12 => ElfSectionHeaderType::ShtSymtabShndx,
        0x13 => ElfSectionHeaderType::ShtNum,
        0x60000000 => ElfSectionHeaderType::ShtLoos,
        _ => ElfSectionHeaderType::ShtNull,
        // other => return Err(format!("Unsupported section header type: {other}")),
    };

    Ok(h_type)
}

pub fn parse_section_name_offset(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
) -> Result<ElfSectionNameOffset, String> {
    let bytes = [
        content[*pointer],
        content[*pointer + 1],
        content[*pointer + 2],
        content[*pointer + 3],
    ];
    *pointer += 4;

    let offset = endian.u32_from(&bytes);
    Ok(ElfSectionNameOffset(offset))
}

pub fn parse_section_header_entry(
    pointer: &mut usize,
    content: &[u8],
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSectionHeaderEntry, String> {
    let section_name_offset = parse_section_name_offset(pointer, content, endian)?;
    let section_name = ElfSectionName::default();
    let section_header_type = parse_section_header_type(pointer, content, endian)?;
    let section_flags = parse_section_flags(pointer, content, endian, platform)?;
    let section_addr = parse_section_addr(pointer, content, endian, platform)?;
    let section_offset = parse_section_offset(pointer, content, endian, platform)?;
    let section_size = parse_section_size(pointer, content, endian, platform)?;
    let section_link = parse_section_link(pointer, content, endian)?;
    let section_info = parse_section_info(pointer, content, endian)?;
    let section_addr_allign = parse_section_addr_allignment(pointer, content, endian, platform)?;
    let section_entry_size = parse_section_entry_size(pointer, content, endian, platform)?;

    Ok({
        ElfSectionHeaderEntry {
            section_name_offset,
            section_name,
            section_header_type,
            section_flags,
            section_addr,
            section_offset,
            section_size,
            section_link,
            section_info,
            section_addr_allign,
            section_entry_size,
        }
    })
}

pub fn parse_section_header(
    pointer: &mut usize,
    content: &[u8],
    entry_count: &ElfSectionHeaderEntryCount,
    sections_names_index: &ElfSectionHeaderSectionsTableIndex,
    endian: &ElfEndianness,
    platform: &ElfPlatformType,
) -> Result<ElfSectionHeader, String> {
    let entry_count = entry_count.0 as usize;
    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let entry = parse_section_header_entry(pointer, content, endian, platform)?;
        entries.push(entry);
    }

    // update the sections names
    let index = sections_names_index.0 as usize;
    let section = &entries[index];
    let section_offset = &section.section_offset.0 as &usize;
    let section_size = section.section_size.0 as usize;
    let bytes: &[u8] = &content[*section_offset..*section_offset + section_size];
    let names = bytes
        .split(|b| *b == 0u8)
        .map(|b| String::from_utf8_lossy(b).to_string())
        .collect::<Vec<String>>();
    entries
        .iter_mut()
        .zip(names.into_iter())
        .for_each(|(entry, name)| entry.section_name = ElfSectionName(name));

    Ok(ElfSectionHeader { inner: entries })
}

pub fn pretty_display<T>(items: &[T])
where
    T: Tabled,
{
    let table = Table::new(items);
    println!("{}", table);
}

pub fn parse_file(args: &Cli) -> Result<ElfBinary, String> {
    let content = read_file(&args.filepath)?;
    let mut elf_binary = ElfBinary::default();
    let mut pointer = 0x0usize;
    elf_binary.header = parse_header(&mut pointer, &content)?;

    elf_binary.content = content;

    match args.to_process {
        ElfParts::Header => {}
        ElfParts::ProgramHeader => {
            pointer = elf_binary.header.program_header_offset.0;
            elf_binary.program_header = parse_program_header(
                &mut pointer,
                &elf_binary.content,
                &elf_binary.header.program_header_entry_count,
                &elf_binary.header.endianness,
                &elf_binary.header.platform_type,
            )?;
        }
        ElfParts::SectionHeader => {
            pointer = elf_binary.header.section_header_offset.0;
            elf_binary.section_header = parse_section_header(
                &mut pointer,
                &elf_binary.content,
                &elf_binary.header.section_header_entry_count,
                &elf_binary.header.section_header_sections_table_index,
                &elf_binary.header.endianness,
                &elf_binary.header.platform_type,
            )?;
        }
        ElfParts::Data => {
            pointer = elf_binary.header.section_header_offset.0;
            elf_binary.section_header = parse_section_header(
                &mut pointer,
                &elf_binary.content,
                &elf_binary.header.section_header_entry_count,
                &elf_binary.header.section_header_sections_table_index,
                &elf_binary.header.endianness,
                &elf_binary.header.platform_type,
            )?;
        }
        ElfParts::All => {
            pointer = elf_binary.header.program_header_offset.0;
            elf_binary.program_header = parse_program_header(
                &mut pointer,
                &elf_binary.content,
                &elf_binary.header.program_header_entry_count,
                &elf_binary.header.endianness,
                &elf_binary.header.platform_type,
            )?;

            pointer = elf_binary.header.section_header_offset.0;
            elf_binary.section_header = parse_section_header(
                &mut pointer,
                &elf_binary.content,
                &elf_binary.header.section_header_entry_count,
                &elf_binary.header.section_header_sections_table_index,
                &elf_binary.header.endianness,
                &elf_binary.header.platform_type,
            )?;
        }
    }

    Ok(elf_binary)
}

fn print_data(elf_binary: &ElfBinary) {
    elf_binary.section_header.inner.iter().for_each(|section| {
        if section.section_header_type == ElfSectionHeaderType::ShtProgbits {
            let section_offset = section.section_offset.0 as usize;
            let section_size = section.section_size.0 as usize;
            let section_name = &section.section_name.0;

            let data = &elf_binary.content[section_offset..section_offset + section_size];

            println!();
            // println!();
            println!(
                "Section: {} | Offset: {:X} | Size: {:X}",
                section_name, section_offset, section_size
            );

            println!("{:02X?}", &data[..16.min(data.len())]);
        }
    });
}

fn main() -> Result<(), String> {
    let args_1: Vec<String> = std::env::args().collect();
    println!("Raw args: {:?}", args_1);

    for i in 3..args_1.len() {
        let args = Cli::parse(std::env::args().skip(1))?;
        let elf_binary: ElfBinary = parse_file(&args)?;

        let arg = Cli::parse(std::env::args().skip(i))?;

        let x = arg.to_process.as_str();
        println!();
        println!();
        println!("\n--- Processing   {}\n ", x);
        println!();

        match arg.to_process {
            ElfParts::Header => pretty_display(&[elf_binary.header]),
            ElfParts::ProgramHeader => pretty_display(&elf_binary.program_header.inner()),
            ElfParts::Data => print_data(&elf_binary),
            ElfParts::SectionHeader => pretty_display(&elf_binary.section_header.inner()),
            ElfParts::All => {
                pretty_display(&[elf_binary.header]);
                pretty_display(&elf_binary.program_header.inner());
                pretty_display(&elf_binary.section_header.inner());
            }
        }
    }

    Ok(())
}
