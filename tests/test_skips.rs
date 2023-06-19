use anyhow::Result;
use dsi_bitstream::prelude::{
    BitRead, BufferedBitStreamRead, BufferedBitStreamWrite, DeltaRead, DeltaWrite, GammaRead,
    GammaWrite, MemWordReadInfinite, MemWordWriteVec, MinimalBinaryRead, MinimalBinaryWrite,
    ZetaRead, ZetaWrite,
};
use dsi_bitstream::traits::{BitSeek, BitWrite, BE};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[test]
fn test_skips() -> Result<()> {
    let mut r = SmallRng::seed_from_u64(0);
    let mut v = SmallRng::seed_from_u64(1);
    let mut buf = BufferedBitStreamWrite::<BE, _>::new(MemWordWriteVec::new(Vec::<u64>::new()));

    let mut pos = vec![];
    for i in 0..2 {
        eprintln!("first round {}", i);
        let mut written_bits = 0;
        match dbg!(r.gen_range(0..6)) {
            0 => {
                for _ in 0..r.gen_range(1..10) {
                    written_bits += buf.write_unary(v.gen_range(0..100))?;
                }
            }
            1 => {
                for _ in 0..r.gen_range(1..10) {
                    written_bits += buf.write_gamma(v.gen_range(0..100))?;
                }
            }
            2 => {
                for _ in 0..r.gen_range(1..10) {
                    written_bits += buf.write_delta(dbg!(v.gen_range(0..100)))?;
                }
            }
            3 => {
                let k = r.gen_range(2..4);
                for _ in 0..r.gen_range(1..10) {
                    written_bits += buf.write_zeta(v.gen_range(0..100), k)?;
                }
            }
            4 => {
                for _ in 0..r.gen_range(1..10) {
                    written_bits += buf.write_zeta3(v.gen_range(0..100))?;
                }
            }
            5 => {
                let max = r.gen_range(1..17);
                for _ in 0..r.gen_range(1..10) {
                    written_bits += buf.write_minimal_binary(v.gen_range(0..max), max)?;
                }
            }
            _ => unreachable!(),
        }
        pos.push(written_bits);
    }
    buf.flush()?;

    let buffer_32: &[u32] = unsafe { pos.align_to().1 };
    let mut read = BufferedBitStreamRead::<BE, u64, _>::new(MemWordReadInfinite::new(buffer_32));

    let mut r = SmallRng::seed_from_u64(0);
    let mut v = SmallRng::seed_from_u64(1);

    for i in 0..2 {
        eprintln!("second round {}", i);
        match dbg!(r.gen_range(0..6)) {
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
                    assert_eq!(v.gen_range(0..100), read.read_minimal_binary(max)?);
                }
            }
            _ => unreachable!(),
        }
    }

    read.set_pos(0)?;
    for i in 0..2 {
        eprintln!("second round {} ", i);
        match dbg!(r.gen_range(0..6)) {
            0 => assert_eq!(
                pos[i],
                (0..r.gen_range(1..10))
                    .map(|_| read.read_unary().unwrap() as usize + 1)
                    .sum::<usize>()
            ),
            1 => assert_eq!(pos[i], read.skip_gamma(r.gen_range(1..10))?),
            2 =>
            //assert_eq!(pos[i], read.skip_deltas(r.gen_range(1..10))?),
            {
                for _ in 0..r.gen_range(1..10) {
                    dbg!(read.read_delta()?);
                }
            }
            3 => {
                let k = r.gen_range(2..4);
                assert_eq!(pos[i], read.skip_zeta(k, r.gen_range(1..10))?)
            }
            4 => assert_eq!(pos[i], read.skip_zeta3(r.gen_range(1..10))?),
            5 => {
                let max = r.gen_range(1..17);
                assert_eq!(pos[i], read.skip_minimal_binary(max, r.gen_range(1..10))?);
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
