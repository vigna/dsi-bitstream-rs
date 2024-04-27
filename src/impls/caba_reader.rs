use core::convert::Infallible;
use std::rc::Rc;

use crate::prelude::{BigEndian, BitRead, GammaRead};


#[derive(Clone, Eq, PartialEq, Debug)]
/// Reader from github.com/caba5/Webgraph
pub struct CabaBinaryReader {
    pub is: Rc<[u8]>,
    pub position: usize,
    pub read_bits: usize,
    pub current: u64,
    pub fill: usize,
}

impl Default for CabaBinaryReader {
    fn default() -> Self {
        Self { 
            is: Rc::new([]), 
            position: Default::default(), 
            read_bits: Default::default(), 
            current: Default::default(), 
            fill: Default::default()
        }
    }
}

impl CabaBinaryReader {
    pub fn new(input_stream: Rc<[u8]>) -> Self {
        CabaBinaryReader { 
            is: input_stream, 
            ..Default::default()
        }
    }

    #[inline(always)]
    pub fn position(&mut self, pos: u64) {
        let bit_delta = ((self.position as i64) << 3) - pos as i64;
        if bit_delta >= 0 && bit_delta as usize <= self.fill {
            self.fill = bit_delta as usize;
            return;
        }

        self.fill = 0;
        self.position = pos as usize >> 3;

        let residual = pos & 7;

        if residual != 0 {
            self.current = self.read().unwrap();
            self.fill = (8 - residual) as usize;
        }
    }

    #[inline(always)]
    pub fn get_position(&self) -> usize {
        (self.position << 3) - self.fill
    }

    #[inline(always)]
    pub(crate) fn read(&mut self) -> Result<u64, ()> {
        if self.position >= self.is.len() {
            return Err(());
        }

        self.position += 1;
        Ok(self.is[self.position - 1] as u64)
    }

    #[inline(always)]
    pub(crate) fn refill(&mut self) -> usize {
        debug_assert!(self.fill < 16);
        
        if let Ok(read) = self.read() {
            self.current = (self.current << 8) | read;
            self.fill += 8;
        }
        if let Ok(read) = self.read() {
            self.current = (self.current << 8) | read;
            self.fill += 8;
        }

        self.fill
    }

    #[inline(always)]
    pub(crate) fn read_from_current(&mut self, len: u64) -> u64 {
        if len == 0 {
            return 0;
        }

        if self.fill == 0 {
            self.current = self.read().unwrap();
            self.fill = 8;
        }

        debug_assert!(len as usize <= self.fill);

        self.read_bits += len as usize;

        self.fill -= len as usize;
        self.current >> self.fill & ((1 << len) - 1)
    }

    #[inline(always)]
    pub fn read_int(&mut self, len: u64) -> u64 {
        debug_assert!(len < 64);
        
        if self.fill < 16 {
            self.refill();
        }

        if len as usize <= self.fill {
            return self.read_from_current(len);
        }

        let mut len = len - self.fill as u64;
        
        let mut x = self.read_from_current(self.fill as u64);

        let mut i = len >> 3;

        while i != 0 {
            x = x << 8 | self.read().unwrap();
            i -= 1;
        }

        self.read_bits += len as usize & !7;

        len &= 7;

        (x << len) | self.read_from_current(len)
    }
}

impl BitRead<BigEndian> for CabaBinaryReader {
    type Error = Infallible;
    type PeekWord = u64;

    #[inline(always)]
    fn read_bits(&mut self, n: usize) -> Result<u64, Self::Error> {
        Ok(self.read_int(n as u64))
    }

    #[inline(always)]
    fn peek_bits(&mut self, n: usize) -> Result<Self::PeekWord, Self::Error> {
        Ok(self.read().unwrap())
    }

    #[inline(always)]
    fn skip_bits(&mut self, n: usize) -> Result<(), Self::Error> {
        self.read_int(n as u64);
        Ok(())
    }

    #[inline(always)]
    fn skip_bits_after_table_lookup(&mut self, n: usize) {
        self.read_from_current(n as u64);
    }

    #[inline(always)]
    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        debug_assert!(self.fill < 64);

        if self.fill < 16 {
            self.refill();
        }

        let mut x = u32::leading_zeros((self.current as u32) << (32 - self.fill));
        if x < self.fill as u32{
            self.read_bits += x as usize + 1;
            self.fill -= x as usize + 1;
            return Ok(x as u64);
        }

        x = self.fill as u32;
        let mut read = self.read();

        if read.is_ok() {
            self.current = read.unwrap();
            while self.current == 0 && read.is_ok() {
                x += 8;
                read = self.read();
                if let Ok(r) = read {
                    self.current = r;
                }
            }
        }

        self.fill = (63 - u64::leading_zeros(self.current)) as usize;
        x += 7 - self.fill as u32;
        self.read_bits += x as usize + 1;
        Ok(x as u64)
    }
}

impl GammaRead<BigEndian> for CabaBinaryReader {

    #[inline(always)]
    fn read_gamma(&mut self) -> Result<u64, Self::Error> {if self.fill >= 16 || self.refill() >= 16 {
        let precomp = super::caba_tables::GAMMAS[self.current as usize >> (self.fill - 16) & 0xFFFF];

        if precomp.1 != 0 {
            self.read_bits += precomp.1 as usize;
            self.fill -= precomp.1 as usize;

            return Ok(precomp.0 as u64);
        }
    }

    let msb = self.read_unary()?;
    Ok(((1 << msb) | self.read_int(msb)) - 1)
    }
}

