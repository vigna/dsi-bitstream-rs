use anyhow::Result;
use dsi_bitstream::prelude::{
    BitRead, BufBitReader, BufBitWriter, DeltaRead, DeltaWrite, GammaRead, GammaWrite,
    MemWordReaderInf, MemWordWriterVec, MinimalBinaryRead, MinimalBinaryWrite, ZetaRead, ZetaWrite,
};
use dsi_bitstream::traits::{BitSeek, BitWrite, BE, LE};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

macro_rules! test_stream {
    ($endianness: ident, $name: ident) => {
        #[test]
        fn $name() -> Result<()> {
            const N: usize = 100000;
            let mut r = SmallRng::seed_from_u64(0);
            let mut v = SmallRng::seed_from_u64(1);
            let mut buffer = Vec::<u64>::new();
            let mut write =
                BufBitWriter::<$endianness, u128, _>::new(MemWordWriterVec::new(&mut buffer));

            let mut pos = vec![];

            for _ in 0..N {
                let mut written_bits = 0;
                match r.gen_range(0..6) {
                    0 => {
                        for _ in 0..r.gen_range(1..10) {
                            written_bits += write.write_unary(v.gen_range(0..100))?;
                        }
                    }
                    1 => {
                        for _ in 0..r.gen_range(1..10) {
                            written_bits += write.write_gamma(v.gen_range(0..100))?;
                        }
                    }
                    2 => {
                        for _ in 0..r.gen_range(1..10) {
                            written_bits += write.write_delta(v.gen_range(0..100))?;
                        }
                    }
                    3 => {
                        let k = r.gen_range(2..4);
                        for _ in 0..r.gen_range(1..10) {
                            written_bits += write.write_zeta(v.gen_range(0..100), k)?;
                        }
                    }
                    4 => {
                        for _ in 0..r.gen_range(1..10) {
                            written_bits += write.write_zeta3(v.gen_range(0..100))?;
                        }
                    }
                    5 => {
                        let max = r.gen_range(1..17);
                        for _ in 0..r.gen_range(1..10) {
                            written_bits += write.write_minimal_binary(v.gen_range(0..max), max)?;
                        }
                    }
                    _ => unreachable!(),
                }
                pos.push(written_bits);
            }

            drop(write);

            let buffer_32: &[u32] = unsafe { &buffer.align_to::<u32>().1 };
            let mut read =
                BufBitReader::<$endianness, u64, _>::new(MemWordReaderInf::new(buffer_32));

            let mut r = SmallRng::seed_from_u64(0);
            let mut v = SmallRng::seed_from_u64(1);

            for _ in 0..N {
                match r.gen_range(0..6) {
                    0 => {
                        for _ in 0..r.gen_range(1..10) {
                            assert_eq!(v.gen_range(0..100), read.read_unary()?);
                        }
                    }
                    1 => {
                        for _ in 0..r.gen_range(1..10) {
                            assert_eq!(v.gen_range(0..100), read.read_gamma()?);
                        }
                    }
                    2 => {
                        for _ in 0..r.gen_range(1..10) {
                            assert_eq!(v.gen_range(0..100), read.read_delta()?);
                        }
                    }
                    3 => {
                        let k = r.gen_range(2..4);
                        for _ in 0..r.gen_range(1..10) {
                            assert_eq!(v.gen_range(0..100), read.read_zeta(k)?);
                        }
                    }
                    4 => {
                        for _ in 0..r.gen_range(1..10) {
                            assert_eq!(v.gen_range(0..100), read.read_zeta3()?);
                        }
                    }
                    5 => {
                        let max = r.gen_range(1..17);
                        for _ in 0..r.gen_range(1..10) {
                            assert_eq!(v.gen_range(0..max), read.read_minimal_binary(max)?);
                        }
                    }
                    _ => unreachable!(),
                }
            }

            let mut r = SmallRng::seed_from_u64(0);
            read.set_bit_pos(0)?;
            let mut count = dsi_bitstream::prelude::CountBitReader::<_, _, true>::new(read);
            let mut bits_read = 0;

            for i in 0..N {
                match r.gen_range(0..6) {
                    0 => {
                        for _ in 0..r.gen_range(1..10) {
                            count.skip_unary()?;
                        }
                        assert_eq!(pos[i], count.bits_read - bits_read);
                    }
                    1 => {
                        for _ in 0..r.gen_range(1..10) {
                            count.skip_gamma()?;
                        }
                        assert_eq!(pos[i], count.bits_read - bits_read);
                    }
                    2 => {
                        for _ in 0..r.gen_range(1..10) {
                            count.skip_delta()?;
                        }
                        assert_eq!(pos[i], count.bits_read - bits_read);
                    }
                    3 => {
                        let k = r.gen_range(2..4);
                        for _ in 0..r.gen_range(1..10) {
                            count.skip_zeta(k)?;
                        }
                        assert_eq!(pos[i], count.bits_read - bits_read);
                    }
                    4 => {
                        for _ in 0..r.gen_range(1..10) {
                            count.skip_zeta3()?;
                        }
                        assert_eq!(pos[i], count.bits_read - bits_read);
                    }
                    5 => {
                        let max = r.gen_range(1..17);
                        for _ in 0..r.gen_range(1..10) {
                            count.skip_minimal_binary(max)?;
                        }
                        assert_eq!(pos[i], count.bits_read - bits_read);
                    }
                    _ => unreachable!(),
                }

                bits_read = count.bits_read;
            }

            Ok(())
        }
    };
}

test_stream!(LE, test_le);
test_stream!(BE, test_be);
