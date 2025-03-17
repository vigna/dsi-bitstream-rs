use super::*;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
/// A zero-sized struct with a const generic parameter representing a code using
/// the values exported by the [`code_consts`] module.
///
/// Methods for all traits are implemented for this struct using a match on the
/// value of the const type parameter. Since the parameter is a constant, the
/// match is resolved at compile time, so there will be no runtime overhead.
///
/// If the value is not among those defined in the [`code_consts`] module, the
/// methods will panic.
///
/// See the [module documentation](crate::codes::dispatch) for more information.
pub struct ConstCode<const CODE: usize>;

impl<const CODE: usize> ConstCode<CODE> {
    /// Delegate the read method to the [`DynamicCodeRead`] implementation.
    ///
    /// This inherent method is provided to reduce ambiguity in method
    /// resolution.
    #[inline(always)]
    pub fn read<E: Endianness, CR: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, CR::Error> {
        DynamicCodeRead::read(self, reader)
    }

    /// Delegate to the [`DynamicCodeWrite`] implementation.
    ///
    /// This inherent method is provided to reduce ambiguity in method
    /// resolution.
    #[inline(always)]
    pub fn write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, CW::Error> {
        DynamicCodeWrite::write(self, writer, value)
    }
}

/// The constants to use as generic parameter for the [`ConstCode`] struct.
pub mod code_consts {
    pub const UNARY: usize = 0;
    pub const GAMMA: usize = 1;
    pub const DELTA: usize = 2;
    pub const OMEGA: usize = 3;
    pub const VBYTE_BE: usize = 4;
    pub const VBYTE_LE: usize = 5;
    pub const ZETA1: usize = GAMMA;
    pub const ZETA2: usize = 6;
    pub const ZETA3: usize = 7;
    pub const ZETA4: usize = 8;
    pub const ZETA5: usize = 9;
    pub const ZETA6: usize = 10;
    pub const ZETA7: usize = 11;
    pub const ZETA8: usize = 12;
    pub const ZETA9: usize = 13;
    pub const ZETA10: usize = 14;
    pub const RICE0: usize = UNARY;
    pub const RICE1: usize = 15;
    pub const RICE2: usize = 16;
    pub const RICE3: usize = 17;
    pub const RICE4: usize = 18;
    pub const RICE5: usize = 19;
    pub const RICE6: usize = 20;
    pub const RICE7: usize = 21;
    pub const RICE8: usize = 22;
    pub const RICE9: usize = 23;
    pub const RICE10: usize = 24;
    pub const PI0: usize = GAMMA;
    pub const PI1: usize = 25;
    pub const PI2: usize = 26;
    pub const PI3: usize = 27;
    pub const PI4: usize = 28;
    pub const PI5: usize = 29;
    pub const PI6: usize = 30;
    pub const PI7: usize = 31;
    pub const PI8: usize = 32;
    pub const PI9: usize = 33;
    pub const PI10: usize = 34;
    pub const GOLOMB1: usize = UNARY;
    pub const GOLOMB2: usize = RICE1;
    pub const GOLOMB3: usize = 35;
    pub const GOLOMB4: usize = RICE2;
    pub const GOLOMB5: usize = 36;
    pub const GOLOMB6: usize = 37;
    pub const GOLOMB7: usize = 38;
    pub const GOLOMB8: usize = RICE3;
    pub const GOLOMB9: usize = 39;
    pub const GOLOMB10: usize = 40;
    pub const EXP_GOLOMB0: usize = GAMMA;
    pub const EXP_GOLOMB1: usize = 41;
    pub const EXP_GOLOMB2: usize = 42;
    pub const EXP_GOLOMB3: usize = 43;
    pub const EXP_GOLOMB4: usize = 44;
    pub const EXP_GOLOMB5: usize = 45;
    pub const EXP_GOLOMB6: usize = 46;
    pub const EXP_GOLOMB7: usize = 47;
    pub const EXP_GOLOMB8: usize = 48;
    pub const EXP_GOLOMB9: usize = 49;
    pub const EXP_GOLOMB10: usize = 50;
}

impl<const CODE: usize> DynamicCodeRead for ConstCode<CODE> {
    fn read<E: Endianness, CR: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, CR::Error> {
        match CODE {
            code_consts::UNARY => reader.read_unary(),
            code_consts::GAMMA => reader.read_gamma(),
            code_consts::DELTA => reader.read_delta(),
            code_consts::OMEGA => reader.read_omega(),
            code_consts::VBYTE_BE => reader.read_vbyte_be(),
            code_consts::VBYTE_LE => reader.read_vbyte_le(),
            code_consts::ZETA2 => reader.read_zeta(2),
            code_consts::ZETA3 => reader.read_zeta3(),
            code_consts::ZETA4 => reader.read_zeta(4),
            code_consts::ZETA5 => reader.read_zeta(5),
            code_consts::ZETA6 => reader.read_zeta(6),
            code_consts::ZETA7 => reader.read_zeta(7),
            code_consts::ZETA8 => reader.read_zeta(8),
            code_consts::ZETA9 => reader.read_zeta(9),
            code_consts::ZETA10 => reader.read_zeta(10),
            code_consts::RICE1 => reader.read_rice(1),
            code_consts::RICE2 => reader.read_rice(2),
            code_consts::RICE3 => reader.read_rice(3),
            code_consts::RICE4 => reader.read_rice(4),
            code_consts::RICE5 => reader.read_rice(5),
            code_consts::RICE6 => reader.read_rice(6),
            code_consts::RICE7 => reader.read_rice(7),
            code_consts::RICE8 => reader.read_rice(8),
            code_consts::RICE9 => reader.read_rice(9),
            code_consts::RICE10 => reader.read_rice(10),
            code_consts::PI1 => reader.read_pi(1),
            code_consts::PI2 => reader.read_pi(2),
            code_consts::PI3 => reader.read_pi(3),
            code_consts::PI4 => reader.read_pi(4),
            code_consts::PI5 => reader.read_pi(5),
            code_consts::PI6 => reader.read_pi(6),
            code_consts::PI7 => reader.read_pi(7),
            code_consts::PI8 => reader.read_pi(8),
            code_consts::PI9 => reader.read_pi(9),
            code_consts::PI10 => reader.read_pi(10),
            code_consts::GOLOMB3 => reader.read_golomb(3),
            code_consts::GOLOMB5 => reader.read_golomb(5),
            code_consts::GOLOMB6 => reader.read_golomb(6),
            code_consts::GOLOMB7 => reader.read_golomb(7),
            code_consts::GOLOMB9 => reader.read_golomb(9),
            code_consts::GOLOMB10 => reader.read_golomb(10),
            code_consts::EXP_GOLOMB1 => reader.read_exp_golomb(1),
            code_consts::EXP_GOLOMB2 => reader.read_exp_golomb(2),
            code_consts::EXP_GOLOMB3 => reader.read_exp_golomb(3),
            code_consts::EXP_GOLOMB4 => reader.read_exp_golomb(4),
            code_consts::EXP_GOLOMB5 => reader.read_exp_golomb(5),
            code_consts::EXP_GOLOMB6 => reader.read_exp_golomb(6),
            code_consts::EXP_GOLOMB7 => reader.read_exp_golomb(7),
            code_consts::EXP_GOLOMB8 => reader.read_exp_golomb(8),
            code_consts::EXP_GOLOMB9 => reader.read_exp_golomb(9),
            code_consts::EXP_GOLOMB10 => reader.read_exp_golomb(10),
            _ => panic!("Unknown code index: {}", CODE),
        }
    }
}

impl<const CODE: usize> DynamicCodeWrite for ConstCode<CODE> {
    fn write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, CW::Error> {
        match CODE {
            code_consts::UNARY => writer.write_unary(value),
            code_consts::GAMMA => writer.write_gamma(value),
            code_consts::DELTA => writer.write_delta(value),
            code_consts::OMEGA => writer.write_omega(value),
            code_consts::VBYTE_BE => writer.write_vbyte_be(value),
            code_consts::VBYTE_LE => writer.write_vbyte_le(value),
            code_consts::ZETA2 => writer.write_zeta(value, 2),
            code_consts::ZETA3 => writer.write_zeta3(value),
            code_consts::ZETA4 => writer.write_zeta(value, 4),
            code_consts::ZETA5 => writer.write_zeta(value, 5),
            code_consts::ZETA6 => writer.write_zeta(value, 6),
            code_consts::ZETA7 => writer.write_zeta(value, 7),
            code_consts::ZETA8 => writer.write_zeta(value, 8),
            code_consts::ZETA9 => writer.write_zeta(value, 9),
            code_consts::ZETA10 => writer.write_zeta(value, 10),
            code_consts::RICE1 => writer.write_rice(value, 1),
            code_consts::RICE2 => writer.write_rice(value, 2),
            code_consts::RICE3 => writer.write_rice(value, 3),
            code_consts::RICE4 => writer.write_rice(value, 4),
            code_consts::RICE5 => writer.write_rice(value, 5),
            code_consts::RICE6 => writer.write_rice(value, 6),
            code_consts::RICE7 => writer.write_rice(value, 7),
            code_consts::RICE8 => writer.write_rice(value, 8),
            code_consts::RICE9 => writer.write_rice(value, 9),
            code_consts::RICE10 => writer.write_rice(value, 10),
            code_consts::PI1 => writer.write_pi(value, 2),
            code_consts::PI2 => writer.write_pi(value, 2),
            code_consts::PI3 => writer.write_pi(value, 3),
            code_consts::PI4 => writer.write_pi(value, 4),
            code_consts::PI5 => writer.write_pi(value, 5),
            code_consts::PI6 => writer.write_pi(value, 6),
            code_consts::PI7 => writer.write_pi(value, 7),
            code_consts::PI8 => writer.write_pi(value, 8),
            code_consts::PI9 => writer.write_pi(value, 9),
            code_consts::PI10 => writer.write_pi(value, 10),
            code_consts::GOLOMB3 => writer.write_golomb(value, 3),
            code_consts::GOLOMB5 => writer.write_golomb(value, 5),
            code_consts::GOLOMB6 => writer.write_golomb(value, 6),
            code_consts::GOLOMB7 => writer.write_golomb(value, 7),
            code_consts::GOLOMB9 => writer.write_golomb(value, 9),
            code_consts::GOLOMB10 => writer.write_golomb(value, 10),
            code_consts::EXP_GOLOMB1 => writer.write_exp_golomb(value, 1),
            code_consts::EXP_GOLOMB2 => writer.write_exp_golomb(value, 2),
            code_consts::EXP_GOLOMB3 => writer.write_exp_golomb(value, 3),
            code_consts::EXP_GOLOMB4 => writer.write_exp_golomb(value, 4),
            code_consts::EXP_GOLOMB5 => writer.write_exp_golomb(value, 5),
            code_consts::EXP_GOLOMB6 => writer.write_exp_golomb(value, 6),
            code_consts::EXP_GOLOMB7 => writer.write_exp_golomb(value, 7),
            code_consts::EXP_GOLOMB8 => writer.write_exp_golomb(value, 8),
            code_consts::EXP_GOLOMB9 => writer.write_exp_golomb(value, 9),
            code_consts::EXP_GOLOMB10 => writer.write_exp_golomb(value, 10),
            _ => panic!("Unknown code: {}", CODE),
        }
    }
}

impl<E: Endianness, CR: CodesRead<E> + ?Sized, const CODE: usize> StaticCodeRead<E, CR>
    for ConstCode<CODE>
{
    #[inline(always)]
    fn read(&self, reader: &mut CR) -> Result<u64, CR::Error> {
        <Self as DynamicCodeRead>::read(self, reader)
    }
}

impl<E: Endianness, CW: CodesWrite<E> + ?Sized, const CODE: usize> StaticCodeWrite<E, CW>
    for ConstCode<CODE>
{
    #[inline(always)]
    fn write(&self, writer: &mut CW, value: u64) -> Result<usize, CW::Error> {
        <Self as DynamicCodeWrite>::write(self, writer, value)
    }
}

impl<const CODE: usize> CodeLen for ConstCode<CODE> {
    #[inline]
    fn len(&self, value: u64) -> usize {
        match CODE {
            code_consts::UNARY => value as usize + 1,
            code_consts::GAMMA => len_gamma(value),
            code_consts::DELTA => len_delta(value),
            code_consts::OMEGA => len_omega(value),
            code_consts::VBYTE_BE | code_consts::VBYTE_LE => bit_len_vbyte(value),
            code_consts::ZETA2 => len_zeta(value, 2),
            code_consts::ZETA3 => len_zeta(value, 3),
            code_consts::ZETA4 => len_zeta(value, 4),
            code_consts::ZETA5 => len_zeta(value, 5),
            code_consts::ZETA6 => len_zeta(value, 6),
            code_consts::ZETA7 => len_zeta(value, 7),
            code_consts::ZETA8 => len_zeta(value, 8),
            code_consts::ZETA9 => len_zeta(value, 9),
            code_consts::ZETA10 => len_zeta(value, 10),
            code_consts::RICE1 => len_rice(value, 1),
            code_consts::RICE2 => len_rice(value, 2),
            code_consts::RICE3 => len_rice(value, 3),
            code_consts::RICE4 => len_rice(value, 4),
            code_consts::RICE5 => len_rice(value, 5),
            code_consts::RICE6 => len_rice(value, 6),
            code_consts::RICE7 => len_rice(value, 7),
            code_consts::RICE8 => len_rice(value, 8),
            code_consts::RICE9 => len_rice(value, 9),
            code_consts::RICE10 => len_rice(value, 10),
            code_consts::PI1 => len_pi(value, 1),
            code_consts::PI2 => len_pi(value, 2),
            code_consts::PI3 => len_pi(value, 3),
            code_consts::PI4 => len_pi(value, 4),
            code_consts::PI5 => len_pi(value, 5),
            code_consts::PI6 => len_pi(value, 6),
            code_consts::PI7 => len_pi(value, 7),
            code_consts::PI8 => len_pi(value, 8),
            code_consts::PI9 => len_pi(value, 9),
            code_consts::PI10 => len_pi(value, 10),
            code_consts::GOLOMB3 => len_golomb(value, 3),
            code_consts::GOLOMB5 => len_golomb(value, 5),
            code_consts::GOLOMB6 => len_golomb(value, 6),
            code_consts::GOLOMB7 => len_golomb(value, 7),
            code_consts::GOLOMB9 => len_golomb(value, 9),
            code_consts::GOLOMB10 => len_golomb(value, 10),
            code_consts::EXP_GOLOMB1 => len_exp_golomb(value, 1),
            code_consts::EXP_GOLOMB2 => len_exp_golomb(value, 2),
            code_consts::EXP_GOLOMB3 => len_exp_golomb(value, 3),
            code_consts::EXP_GOLOMB4 => len_exp_golomb(value, 4),
            code_consts::EXP_GOLOMB5 => len_exp_golomb(value, 5),
            code_consts::EXP_GOLOMB6 => len_exp_golomb(value, 6),
            code_consts::EXP_GOLOMB7 => len_exp_golomb(value, 7),
            code_consts::EXP_GOLOMB8 => len_exp_golomb(value, 8),
            code_consts::EXP_GOLOMB9 => len_exp_golomb(value, 9),
            code_consts::EXP_GOLOMB10 => len_exp_golomb(value, 10),
            _ => panic!("Unknown code: {}", CODE),
        }
    }
}
