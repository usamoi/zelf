use crate::context::*;
use crate::context::{PropU16, PropU32};
use crate::ident::*;
use crate::utils::*;
use crate::{Class, Data, Version};

#[derive(Debug, Clone)]
pub enum ParseElfsError {
    BadIdent(ParseIdentError),
    BadElf(ParseElfError),
}

#[derive(Debug, Clone)]
pub enum ParseElfError {
    BadHeader,
    BadPropertyType,
    BadPropertyEhsize,
}

#[derive(Debug, Clone, Copy)]
pub enum Elfs<'a> {
    Little32(Elf<'a, Little32>),
    Little64(Elf<'a, Little64>),
    Big32(Elf<'a, Big32>),
    Big64(Elf<'a, Big64>),
}

impl<'a> Elfs<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self, ParseElfsError> {
        use {Class::*, Data::*, ParseElfsError::*, Version::*};
        let ident = Ident::parse(data).map_err(BadIdent)?;
        let elf = match (ident.class(), ident.data(), ident.version()) {
            (Class32, Little, One) => Elfs::Little32(Elf::<Little32>::parse(data).map_err(BadElf)?),
            (Class32, Big, One) => Elfs::Big32(Elf::<Big32>::parse(data).map_err(BadElf)?),
            (Class64, Little, One) => Elfs::Little64(Elf::<Little64>::parse(data).map_err(BadElf)?),
            (Class64, Big, One) => Elfs::Big64(Elf::<Big64>::parse(data).map_err(BadElf)?),
        };
        Ok(elf)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Elf<'a, T: Context> {
    data: &'a [u8],
    header: &'a ElfHeader<T>,
}

impl<'a, T: Context> Elf<'a, T> {
    /// This function does not check if its identication matches the context.
    pub fn parse(data: &'a [u8]) -> Result<Self, ParseElfError> {
        use ParseElfError::*;
        let eheader: &ElfHeader<T> = read(data, 0).ok_or(BadHeader)?;
        let _type = eheader.checked_type().ok_or(BadPropertyType)?;
        if core::mem::size_of::<ElfHeader<T>>() != eheader.ehsize() as usize {
            return Err(BadPropertyEhsize);
        }
        Ok(Elf {
            data,
            header: eheader,
        })
    }
    pub fn data(&self) -> &'a [u8] {
        self.data
    }
    pub fn header(&self) -> &'a ElfHeader<T> {
        self.header
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ElfHeader<T: Context> {
    pub ident: Ident,
    pub typa: PropU16,
    pub machine: PropU16,
    pub version: PropU32,
    pub entry: T::PropUsize,
    pub phoff: T::PropUsize,
    pub shoff: T::PropUsize,
    pub flags: PropU32,
    pub ehsize: PropU16,
    pub phentsize: PropU16,
    pub phnum: PropU16,
    pub shentsize: PropU16,
    pub shnum: PropU16,
    pub shstrndx: PropU16,
}

impl<T: Context> ElfHeader<T> {
    /// Identification.
    pub fn ident(&self) -> &Ident {
        &self.ident
    }
    /// Object file type.
    pub fn checked_type(&self) -> Option<ElfType> {
        ElfType::try_from(T::interpret(self.typa)).ok()
    }
    /// Object file type.
    ///
    /// # Panics
    ///
    /// Panics if its value is invaild.
    pub fn typa(&self) -> ElfType {
        self.checked_type().unwrap()
    }
    /// Target instruction set architecture.
    pub fn machine(&self) -> u16 {
        T::interpret(self.machine)
    }
    /// Object file version.
    pub fn version(&self) -> u32 {
        T::interpret(self.version)
    }
    /// Entry point address.
    pub fn entry(&self) -> T::Integer {
        T::interpret(self.entry)
    }
    /// Program header table file offset.
    pub fn phoff(&self) -> T::Integer {
        T::interpret(self.phoff)
    }
    /// Section header table file offset.
    pub fn shoff(&self) -> T::Integer {
        T::interpret(self.shoff)
    }
    /// Flags.
    pub fn flags(&self) -> u32 {
        T::interpret(self.flags)
    }
    /// ELF header size in bytes.
    pub fn ehsize(&self) -> u16 {
        T::interpret(self.ehsize)
    }
    /// Program header table's entry size.
    pub fn phentsize(&self) -> u16 {
        T::interpret(self.phentsize)
    }
    /// Program header table's entry count.
    pub fn phnum(&self) -> u16 {
        T::interpret(self.phnum)
    }
    /// Section header table's entry size.
    pub fn shentsize(&self) -> u16 {
        T::interpret(self.shentsize)
    }
    /// Section header table's entry count.
    pub fn shnum(&self) -> u16 {
        T::interpret(self.shnum)
    }
    /// Section header string table index.
    pub fn shstrndx(&self) -> u16 {
        T::interpret(self.shstrndx)
    }
}

unsafe impl<T: Context> Pod for ElfHeader<T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfType {
    /// No file type.
    None,
    /// Relocatable file.
    Rel,
    /// Executable file.
    Exec,
    /// Shared object file.
    Dyn,
    /// Core file.
    Core,
    /// Operating system-specific.
    OsSpecific(u16),
    /// Processor-specific.
    ProcessorSpecific(u16),
}

impl TryFrom<u16> for ElfType {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        use ElfType::*;
        match value {
            0x00 => Ok(None),
            0x01 => Ok(Rel),
            0x02 => Ok(Exec),
            0x03 => Ok(Dyn),
            0x04 => Ok(Core),
            x @ 0xFE00..=0xFEFF => Ok(OsSpecific(x)),
            x @ 0xFF00..=0xFFFF => Ok(ProcessorSpecific(x)),
            _ => Err(()),
        }
    }
}

impl From<ElfType> for u16 {
    fn from(value: ElfType) -> Self {
        use ElfType::*;
        match value {
            None => 0x00,
            Rel => 0x01,
            Exec => 0x02,
            Dyn => 0x03,
            Core => 0x04,
            OsSpecific(x) => x,
            ProcessorSpecific(x) => x,
        }
    }
}

/// No machine.
pub const ELF_MACHINE_NONE: u16 = 0x00;
/// AT&T WE 32100.
pub const ELF_MACHINE_M32: u16 = 0x01;
/// SPARC.
pub const ELF_MACHINE_SPARC: u16 = 0x02;
/// Intel 80386.
pub const ELF_MACHINE_X86: u16 = 0x03;
/// Motorola 68000.
pub const ELF_MACHINE_68K: u16 = 0x04;
/// Motorola 88000.
pub const ELF_MACHINE_88K: u16 = 0x05;
/// Intel MCU.
pub const ELF_MACHINE_IAMCU: u16 = 0x06;
/// Intel 80860.
pub const ELF_MACHINE_860: u16 = 0x07;
/// MIPS I Architecture.
pub const ELF_MACHINE_MIPS: u16 = 0x08;
/// IBM System/370 Processor.
pub const ELF_MACHINE_S370: u16 = 0x09;
/// MIPS RS3000 Little-endian.
pub const ELF_MACHINE_MIPS_RS3_LE: u16 = 0x0A;
/// Hewlett-Packard PA-RISC.
pub const ELF_MACHINE_PARISC: u16 = 0x0F;
/// Fujitsu VPP500.
pub const ELF_MACHINE_VPP500: u16 = 0x11;
/// Enhanced instruction set SPARC.
pub const ELF_MACHINE_SPARC32PLUS: u16 = 0x12;
/// Intel 80960.
pub const ELF_MACHINE_960: u16 = 0x13;
/// PowerPC.
pub const ELF_MACHINE_PPC: u16 = 0x14;
/// 64-bit PowerPC.
pub const ELF_MACHINE_PPC64: u16 = 0x15;
/// IBM System/390 Processor.
pub const ELF_MACHINE_S390: u16 = 0x16;
/// IBM SPU/SPC.
pub const ELF_MACHINE_SPU: u16 = 0x17;
/// NEC V800.
pub const ELF_MACHINE_V800: u16 = 36;
/// Fujitsu FR20.
pub const ELF_MACHINE_FR20: u16 = 37;
/// TRW RH-32.
pub const ELF_MACHINE_RH32: u16 = 38;
/// Motorola RCE.
pub const ELF_MACHINE_RCE: u16 = 39;
/// ARM 32-bit architecture (AARCH32).
pub const ELF_MACHINE_ARM: u16 = 40;
/// Digital Alpha.
pub const ELF_MACHINE_ALPHA: u16 = 41;
/// Hitachi SH.
pub const ELF_MACHINE_SH: u16 = 42;
/// SPARC Version 9.
pub const ELF_MACHINE_SPARCV9: u16 = 43;
/// Siemens TriCore embedded processor.
pub const ELF_MACHINE_TRICORE: u16 = 44;
/// Argonaut RISC Core, Argonaut Technologies Inc..
pub const ELF_MACHINE_ARC: u16 = 45;
/// Hitachi H8/300.
pub const ELF_MACHINE_H8_300: u16 = 46;
/// Hitachi H8/300H.
pub const ELF_MACHINE_H8_300H: u16 = 47;
/// Hitachi H8S.
pub const ELF_MACHINE_H8S: u16 = 48;
/// Hitachi H8/500.
pub const ELF_MACHINE_H8_500: u16 = 49;
/// Intel IA-64 processor architecture.
pub const ELF_MACHINE_IA_64: u16 = 50;
/// Stanford MIPS-X.
pub const ELF_MACHINE_MIPS_X: u16 = 51;
/// Motorola ColdFire.
pub const ELF_MACHINE_COLDFIRE: u16 = 52;
/// Motorola M68HC12.
pub const ELF_MACHINE_68HC12: u16 = 53;
/// Fujitsu MMA Multimedia Accelerator.
pub const ELF_MACHINE_MMA: u16 = 54;
/// Siemens PCP.
pub const ELF_MACHINE_PCP: u16 = 55;
/// Sony nCPU embedded RISC processor.
pub const ELF_MACHINE_NCPU: u16 = 56;
/// Denso NDR1 microprocessor.
pub const ELF_MACHINE_NDR1: u16 = 57;
/// Motorola Star*Core processor.
pub const ELF_MACHINE_STARCORE: u16 = 58;
/// Toyota ME16 processor.
pub const ELF_MACHINE_ME16: u16 = 59;
/// STMicroelectronics ST100 processor.
pub const ELF_MACHINE_ST100: u16 = 60;
/// Advanced Logic Corp. TinyJ embedded processor family.
pub const ELF_MACHINE_TINYJ: u16 = 61;
/// AMD x86-64 architecture.
pub const ELF_MACHINE_X86_64: u16 = 62;
/// Sony DSP Processor.
pub const ELF_MACHINE_PDSP: u16 = 63;
/// Digital Equipment Corp. PDP-10.
pub const ELF_MACHINE_PDP10: u16 = 64;
/// Digital Equipment Corp. PDP-11.
pub const ELF_MACHINE_PDP11: u16 = 65;
/// Siemens FX66 microcontroller.
pub const ELF_MACHINE_FX66: u16 = 66;
/// STMicroelectronics ST9+ 8/16 bit microcontroller.
pub const ELF_MACHINE_ST9PLUS: u16 = 67;
/// STMicroelectronics ST7 8-bit microcontroller.
pub const ELF_MACHINE_ST7: u16 = 68;
/// Motorola MC68HC16 Microcontroller.
pub const ELF_MACHINE_68HC16: u16 = 69;
/// Motorola MC68HC11 Microcontroller.
pub const ELF_MACHINE_68HC11: u16 = 70;
/// Motorola MC68HC08 Microcontroller.
pub const ELF_MACHINE_68HC08: u16 = 71;
/// Motorola MC68HC05 Microcontroller.
pub const ELF_MACHINE_68HC05: u16 = 72;
/// Silicon Graphics SVx.
pub const ELF_MACHINE_SVX: u16 = 73;
/// STMicroelectronics ST19 8-bit microcontroller.
pub const ELF_MACHINE_ST19: u16 = 74;
/// Digital VAX.
pub const ELF_MACHINE_VAX: u16 = 75;
/// Axis Communications 32-bit embedded processor.
pub const ELF_MACHINE_CRIS: u16 = 76;
/// Infineon Technologies 32-bit embedded processor.
pub const ELF_MACHINE_JAVELIN: u16 = 77;
/// Element 14 64-bit DSP Processor.
pub const ELF_MACHINE_FIREPATH: u16 = 78;
/// LSI Logic 16-bit DSP Processor.
pub const ELF_MACHINE_ZSP: u16 = 79;
/// Donald Knuth's educational 64-bit processor.
pub const ELF_MACHINE_MMIX: u16 = 80;
/// Harvard University machine-independent object files.
pub const ELF_MACHINE_HUANY: u16 = 81;
/// SiTera Prism.
pub const ELF_MACHINE_PRISM: u16 = 82;
/// Atmel AVR 8-bit microcontroller.
pub const ELF_MACHINE_AVR: u16 = 83;
/// Fujitsu FR30.
pub const ELF_MACHINE_FR30: u16 = 84;
/// Mitsubishi D10V.
pub const ELF_MACHINE_D10V: u16 = 85;
/// Mitsubishi D30V.
pub const ELF_MACHINE_D30V: u16 = 86;
/// NEC v850.
pub const ELF_MACHINE_V850: u16 = 87;
/// Mitsubishi M32R.
pub const ELF_MACHINE_M32R: u16 = 88;
/// Matsushita MN10300.
pub const ELF_MACHINE_MN10300: u16 = 89;
/// Matsushita MN10200.
pub const ELF_MACHINE_MN10200: u16 = 90;
/// picoJava.
pub const ELF_MACHINE_PJ: u16 = 91;
/// OpenRISC 32-bit embedded processor.
pub const ELF_MACHINE_OPENRISC: u16 = 92;
/// ARC International ARCompact processor (old spelling/synonym: EM_ARC_A5).
pub const ELF_MACHINE_ARC_COMPACT: u16 = 93;
/// Tensilica Xtensa Architecture.
pub const ELF_MACHINE_XTENSA: u16 = 94;
/// Alphamosaic VideoCore processor.
pub const ELF_MACHINE_VIDEOCORE: u16 = 95;
/// Thompson Multimedia General Purpose Processor.
pub const ELF_MACHINE_TMM_GPP: u16 = 96;
/// National Semiconductor 32000 series.
pub const ELF_MACHINE_NS32K: u16 = 97;
/// Tenor Network TPC processor.
pub const ELF_MACHINE_TPC: u16 = 98;
/// Trebia SNP 1000 processor.
pub const ELF_MACHINE_SNP1K: u16 = 99;
/// STMicroelectronics (www.st.com) ST200 microcontroller.
pub const ELF_MACHINE_ST200: u16 = 100;
/// Ubicom IP2xxx microcontroller family.
pub const ELF_MACHINE_IP2K: u16 = 101;
/// MAX Processor.
pub const ELF_MACHINE_MAX: u16 = 102;
/// National Semiconductor CompactRISC microprocessor.
pub const ELF_MACHINE_CR: u16 = 103;
/// Fujitsu F2MC16.
pub const ELF_MACHINE_F2MC16: u16 = 104;
/// Texas Instruments embedded microcontroller msp430.
pub const ELF_MACHINE_MSP430: u16 = 105;
/// Analog Devices Blackfin (DSP) processor.
pub const ELF_MACHINE_BLACKFIN: u16 = 106;
/// S1C33 Family of Seiko Epson processors.
pub const ELF_MACHINE_SE_C33: u16 = 107;
/// Sharp embedded microprocessor.
pub const ELF_MACHINE_SEP: u16 = 108;
/// Arca RISC Microprocessor.
pub const ELF_MACHINE_ARCA: u16 = 109;
/// Microprocessor series from PKU-Unity Ltd. and MPRC of Peking University.
pub const ELF_MACHINE_UNICORE: u16 = 110;
/// eXcess: 16/32/64-bit configurable embedded CPU.
pub const ELF_MACHINE_EXCESS: u16 = 111;
/// Icera Semiconductor Inc. Deep Execution Processor.
pub const ELF_MACHINE_DXP: u16 = 112;
/// Altera Nios II soft-core processor.
pub const ELF_MACHINE_ALTERA_NIOS2: u16 = 113;
/// National Semiconductor CompactRISC CRX microprocessor.
pub const ELF_MACHINE_CRX: u16 = 114;
/// Motorola XGATE embedded processor.
pub const ELF_MACHINE_XGATE: u16 = 115;
/// Infineon C16x/XC16x processor.
pub const ELF_MACHINE_C166: u16 = 116;
/// Renesas M16C series microprocessors.
pub const ELF_MACHINE_M16C: u16 = 117;
/// Microchip Technology dsPIC30F Digital Signal Controller.
pub const ELF_MACHINE_DSPIC30F: u16 = 118;
/// Freescale Communication Engine RISC core.
pub const ELF_MACHINE_CE: u16 = 119;
/// Renesas M32C series microprocessors.
pub const ELF_MACHINE_M32C: u16 = 120;
/// Altium TSK3000 core.
pub const ELF_MACHINE_TSK3000: u16 = 131;
/// Freescale RS08 embedded processor.
pub const ELF_MACHINE_RS08: u16 = 132;
/// Analog Devices SHARC family of 32-bit DSP processors.
pub const ELF_MACHINE_SHARC: u16 = 133;
/// Cyan Technology eCOG2 microprocessor.
pub const ELF_MACHINE_ECOG2: u16 = 134;
/// Sunplus S+core7 RISC processor.
pub const ELF_MACHINE_SCORE7: u16 = 135;
/// New Japan Radio (NJR) 24-bit DSP Processor.
pub const ELF_MACHINE_DSP24: u16 = 136;
/// Broadcom VideoCore III processor.
pub const ELF_MACHINE_VIDEOCORE3: u16 = 137;
/// RISC processor for Lattice FPGA architecture.
pub const ELF_MACHINE_LATTICEMICO32: u16 = 138;
/// Seiko Epson C17 family.
pub const ELF_MACHINE_SE_C17: u16 = 139;
/// The Texas Instruments TMS320C6000 DSP family.
pub const ELF_MACHINE_TI_C6000: u16 = 140;
/// The Texas Instruments TMS320C2000 DSP family.
pub const ELF_MACHINE_TI_C2000: u16 = 141;
/// The Texas Instruments TMS320C55x DSP family.
pub const ELF_MACHINE_TI_C5500: u16 = 142;
/// Texas Instruments Application Specific RISC Processor, 32bit fetch.
pub const ELF_MACHINE_TI_ARP32: u16 = 143;
/// Texas Instruments Programmable Realtime Unit.
pub const ELF_MACHINE_TI_PRU: u16 = 144;
/// STMicroelectronics 64bit VLIW Data Signal Processor.
pub const ELF_MACHINE_MMDSP_PLUS: u16 = 160;
/// Cypress M8C microprocessor.
pub const ELF_MACHINE_CYPRESS_M8C: u16 = 161;
/// Renesas R32C series microprocessors.
pub const ELF_MACHINE_R32C: u16 = 162;
/// NXP Semiconductors TriMedia architecture family.
pub const ELF_MACHINE_TRIMEDIA: u16 = 163;
/// QUALCOMM DSP6 Processor.
pub const ELF_MACHINE_QDSP6: u16 = 164;
/// Intel 8051 and variants.
pub const ELF_MACHINE_8051: u16 = 165;
/// STMicroelectronics STxP7x family of configurable and extensible RISC processors.
pub const ELF_MACHINE_STXP7X: u16 = 166;
/// Andes Technology compact code size embedded RISC processor family.
pub const ELF_MACHINE_NDS32: u16 = 167;
/// Cyan Technology eCOG1X family.
pub const ELF_MACHINE_ECOG1X: u16 = 168;
/// Dallas Semiconductor MAXQ30 Core Micro-controllers.
pub const ELF_MACHINE_MAXQ30: u16 = 169;
/// New Japan Radio (NJR) 16-bit DSP Processor.
pub const ELF_MACHINE_XIMO16: u16 = 170;
/// M2000 Reconfigurable RISC Microprocessor.
pub const ELF_MACHINE_MANIK: u16 = 171;
/// Cray Inc. NV2 vector architecture.
pub const ELF_MACHINE_CRAYNV2: u16 = 172;
/// Renesas RX family.
pub const ELF_MACHINE_RX: u16 = 173;
/// Imagination Technologies META processor architecture.
pub const ELF_MACHINE_METAG: u16 = 174;
/// MCST Elbrus general purpose hardware architecture.
pub const ELF_MACHINE_MCST_ELBRUS: u16 = 175;
/// Cyan Technology eCOG16 family.
pub const ELF_MACHINE_ECOG16: u16 = 176;
/// National Semiconductor CompactRISC CR16 16-bit microprocessor.
pub const ELF_MACHINE_CR16: u16 = 177;
/// Freescale Extended Time Processing Unit.
pub const ELF_MACHINE_ETPU: u16 = 178;
/// Infineon Technologies SLE9X core.
pub const ELF_MACHINE_SLE9X: u16 = 179;
/// Intel L10M.
pub const ELF_MACHINE_L10M: u16 = 180;
/// Intel K10M.
pub const ELF_MACHINE_K10M: u16 = 181;
/// ARM 64-bit architecture (AARCH64).
pub const ELF_MACHINE_AARCH64: u16 = 183;
/// Atmel Corporation 32-bit microprocessor family.
pub const ELF_MACHINE_AVR32: u16 = 185;
/// STMicroeletronics STM8 8-bit microcontroller.
pub const ELF_MACHINE_STM8: u16 = 186;
/// Tilera TILE64 multicore architecture family.
pub const ELF_MACHINE_TILE64: u16 = 187;
/// Tilera TILEPro multicore architecture family.
pub const ELF_MACHINE_TILEPRO: u16 = 188;
/// Xilinx MicroBlaze 32-bit RISC soft processor core.
pub const ELF_MACHINE_MICROBLAZE: u16 = 189;
/// NVIDIA CUDA architecture.
pub const ELF_MACHINE_CUDA: u16 = 190;
/// Tilera TILE-Gx multicore architecture family.
pub const ELF_MACHINE_TILEGX: u16 = 191;
/// CloudShield architecture family.
pub const ELF_MACHINE_CLOUDSHIELD: u16 = 192;
/// KIPO-KAIST Core-A 1st generation processor family.
pub const ELF_MACHINE_COREA_1ST: u16 = 193;
/// KIPO-KAIST Core-A 2nd generation processor family.
pub const ELF_MACHINE_COREA_2ND: u16 = 194;
/// Synopsys ARCompact V2.
pub const ELF_MACHINE_ARC_COMPACT2: u16 = 195;
/// Open8 8-bit RISC soft processor core.
pub const ELF_MACHINE_OPEN8: u16 = 196;
/// Renesas RL78 family.
pub const ELF_MACHINE_RL78: u16 = 197;
/// Broadcom VideoCore V processor.
pub const ELF_MACHINE_VIDEOCORE5: u16 = 198;
/// Renesas 78KOR family.
pub const ELF_MACHINE_78KOR: u16 = 199;
/// Freescale 56800EX Digital Signal Controller (DSC).
pub const ELF_MACHINE_56800EX: u16 = 200;
/// Beyond BA1 CPU architecture.
pub const ELF_MACHINE_BA1: u16 = 201;
/// Beyond BA2 CPU architecture.
pub const ELF_MACHINE_BA2: u16 = 202;
/// XMOS xCORE processor family.
pub const ELF_MACHINE_XCORE: u16 = 203;
/// Microchip 8-bit PIC(r) family.
pub const ELF_MACHINE_MCHP_PIC: u16 = 204;
/// KM211 KM32 32-bit processor.
pub const ELF_MACHINE_KM32: u16 = 210;
/// KM211 KMX32 32-bit processor.
pub const ELF_MACHINE_KMX32: u16 = 211;
/// KM211 KMX16 16-bit processor.
pub const ELF_MACHINE_KMX16: u16 = 212;
/// KM211 KMX8 8-bit processor.
pub const ELF_MACHINE_KMX8: u16 = 213;
/// KM211 KVARC processor.
pub const ELF_MACHINE_KVARC: u16 = 214;
/// Paneve CDP architecture family.
pub const ELF_MACHINE_CDP: u16 = 215;
/// Cognitive Smart Memory Processor.
pub const ELF_MACHINE_COGE: u16 = 216;
/// Bluechip Systems CoolEngine.
pub const ELF_MACHINE_COOL: u16 = 217;
/// Nanoradio Optimized RISC.
pub const ELF_MACHINE_NORC: u16 = 218;
/// CSR Kalimba architecture family.
pub const ELF_MACHINE_CSR_KALIMBA: u16 = 219;
/// Zilog Z80.
pub const ELF_MACHINE_Z80: u16 = 220;
/// Controls and Data Services VISIUMcore processor.
pub const ELF_MACHINE_VISIUM: u16 = 221;
/// FTDI Chip FT32 high performance 32-bit RISC architecture.
pub const ELF_MACHINE_FT32: u16 = 222;
/// Moxie processor family.
pub const ELF_MACHINE_MOXIE: u16 = 223;
/// AMD GPU architecture.
pub const ELF_MACHINE_AMDGPU: u16 = 224;
/// RISC-V.
pub const ELF_MACHINE_RISCV: u16 = 243;
