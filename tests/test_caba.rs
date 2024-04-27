use dsi_bitstream::prelude::*;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::hint::black_box;
use std::rc::Rc;
use std::time::Duration;

pub fn gen_gamma_data(n: usize) -> Vec<u64> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distr = rand_distr::Zeta::new(2.0).unwrap();

    (0..n)
        .map(|_| rng.sample(distr) as u64 - 1)
        .collect::<Vec<_>>()
}


#[test]
fn test_caba() {
    let data = <Vec<u64>>::new();
    let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(data));
    let s = gen_gamma_data(1 << 10);

    for value in s {
        writer.write_gamma(value).unwrap();
    }

    let mut data = writer.into_inner().unwrap().into_inner();

    let data = unsafe{data.align_to().1}.to_vec();
    let data: Rc<[u8]> = data.into_boxed_slice().into();
    let mut reader = CabaBinaryReader::new(data);

    for i in 0..1000 {
        black_box(reader.read_gamma().unwrap());
    }
}