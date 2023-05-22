use crate::backends::*;
use crate::codes::*;
use crate::traits::*;
use anyhow::Result;

pub trait ReadCodesParams {}

pub struct DefaultReadParams;
impl ReadCodesParams for DefaultReadParams {}

macro_rules! impl_default_read_codes {
    ($($endianess:ident),*) => {$(
        impl<BW: Word, WR: WordRead, DC: ReadCodesParams> GammaRead<$endianess>
            for BufferedBitStreamRead<$endianess, BW, WR, DC>
        where
            BW: DowncastableInto<WR::Word> + CastableInto<u64>,
            WR::Word: UpcastableInto<BW> + UpcastableInto<u64>,
        {
            #[inline(always)]
            fn read_gamma(&mut self) -> Result<u64> {
                // From our tests, the ARM architecture is faster
                // without tables ɣ codes.
                #[cfg(target_arch = "arm" )]
                return self.read_gamma_param::<false>();
                #[cfg(not(target_arch = "arm" ))]
                return self.read_gamma_param::<true>();
            }
        }

        impl<BW: Word, WR: WordRead, DC: ReadCodesParams> DeltaRead<$endianess>
            for BufferedBitStreamRead<$endianess, BW, WR, DC>
        where
            BW: DowncastableInto<WR::Word> + CastableInto<u64>,
            WR::Word: UpcastableInto<BW> + UpcastableInto<u64>,
        {
            #[inline(always)]
            fn read_delta(&mut self) -> Result<u64> {
                // From our tests, the ARM architecture is faster
                // without tables for ɣ codes.
                #[cfg(target_arch = "arm" )]
                return self.read_delta_param::<false, false>();
                #[cfg(not(target_arch = "arm" ))]
                return self.read_delta_param::<false, true>();
            }
        }

        impl<BW: Word, WR: WordRead, DC: ReadCodesParams> ZetaRead<$endianess>
            for BufferedBitStreamRead<$endianess, BW, WR, DC>
        where
            BW: DowncastableInto<WR::Word> + CastableInto<u64>,
            WR::Word: UpcastableInto<BW> + UpcastableInto<u64>,
        {
            #[inline(always)]
            fn read_zeta(&mut self, k: u64) -> Result<u64> {
                self.read_zeta_param::<true>(k)
            }

            #[inline(always)]
            fn read_zeta3(&mut self) -> Result<u64> {
                self.read_zeta3_param::<true>()
            }
        }

    )*};
}

impl_default_read_codes! {LittleEndian, BigEndian}

pub trait WriteCodesParams {}

pub struct DefaultWriteParams;
impl WriteCodesParams for DefaultWriteParams {}

macro_rules! impl_default_write_codes {
    ($($endianess:ident),*) => {$(
        impl<DC: WriteCodesParams> GammaWrite<$endianess>
            for BufferedBitStreamWrite<$endianess, DC>
        {
            #[inline(always)]
            fn write_gamma(&mut self) -> Result<u64> {
                self.write_gamma_param::<true>()
            }
        }

        impl<DC: WriteCodesParams> DeltaWrite<$endianess>
            for BufferedBitStreamWrite<$endianess, DC>
        {
            #[inline(always)]
            fn write_delta(&mut self) -> Result<u64> {
                self.write_delta_param::<true, true>()
            }
        }

        impl<DC: WriteCodesParams> ZetaWrite<$endianess>
            for BufferedBitStreamWrite<$endianess, DC>
        {
            #[inline(always)]
            fn write_zeta(&mut self, k: u64) -> Result<u64> {
                self.write_zeta_param::<true>(k)
            }

            #[inline(always)]
            fn write_zeta3(&mut self) -> Result<u64> {
                self.write_zeta3_param::<true>()
            }
        }

    )*};
}
impl_default_write_codes! {LittleEndian, BigEndian}
