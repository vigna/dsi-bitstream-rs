/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Implementation of Huffman optimal codes for encoding and decoding.

use common_traits::CastableInto;
use mem_dbg::{MemDbg, MemSize};
use epserde::Epserde;
use anyhow::{Result, ensure};

use crate::prelude::{BitRead, BitSeek, BitWrite, Endianness};

#[derive(Debug, Clone, Copy, Epserde, MemDbg, MemSize)]
/// A representation of a binary code. This is just used to make the code
/// more readable.
pub struct Code {
    pub code: usize,
    pub len: u8,
}

impl core::default::Default for Code {
    #[inline(always)]
    fn default() -> Self {
        Self { code: 0, len: 0 }
    }
}

impl core::fmt::Display for Code {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:0width$b}", self.code, width = self.len as usize)
    }
}

impl core::ops::Shl<bool> for Code {
    type Output = Self;

    #[inline(always)]
    fn shl(mut self, bit: bool) -> Self {
        debug_assert!(self.len < usize::BITS as u8, "Code too long");
        self.code <<= 1;
        self.code |= bit as usize;
        self.len += 1;
        self
    }
}

#[derive(Debug)]
/// Node of an huffman-tree in construction.
enum Node {
    Leaf {
        count: usize,
        symbol: usize,
    },
    Internal {
        count: usize,
        left: Box<Node>,
        right: Box<Node>,
    },
}

impl Node {
    #[inline(always)]
    fn new(count: usize, symbol: usize) -> Self {
        Self::Leaf {
            count,
            symbol: symbol,
        }
    }
    #[inline(always)]
    fn get_count(&self) -> usize {
        match self {
            Self::Leaf { count, .. } => *count,
            Self::Internal { count, .. } => *count,
        }
    }
}

impl PartialEq for Node {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.get_count() == other.get_count()
    }
}
impl Eq for Node {}
impl PartialOrd for Node {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.get_count().partial_cmp(&self.get_count())
    }
}
impl Ord for Node {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.get_count().cmp(&self.get_count())
    }
}

#[derive(Debug, Clone, Copy, Default, Epserde, MemDbg, MemSize)]
/// Compact representation of a node in the huffman tree.
/// The node is either a leaf or an index to another node.
/// For debug purposes, it also encode "empty" to represent an invalid node,
/// this is used for assretions, but is not needed for the algorithm. 
pub struct CompactNode(u16);

impl CompactNode {
    #[inline(always)]
    fn new_leaf(symbol: usize) -> Self {
        debug_assert!(symbol < 0x8000);
        Self((symbol as u16) | 0x8000)
    }
    #[inline(always)]
    fn new_index(idx: usize) -> Self {
        debug_assert!(idx < 0x7FFF);
        Self(idx as u16)
    }
    #[inline(always)]
    fn empty() -> Self {
        Self(!0)
    }
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.0 == !0
    }
    #[inline(always)]
    fn is_leaf(&self) -> bool {
        self.0 & 0x8000 != 0
    }
    #[inline(always)]
    fn is_index(&self) -> bool {
        !self.is_leaf()
    }
    #[inline(always)]
    fn symbol(&self) -> usize {
        debug_assert!(self.is_leaf());
        self.0 as usize & 0x7FFF
    }
    #[inline(always)]
    fn index(&self) -> usize {
        debug_assert!(self.is_index());
        self.0 as usize
    }
}


/// Do a depth first visit of the huffman tree and extract the codes.
fn extract_codes(node: &Box<Node>, code: Code, result: &mut Vec<(usize, Code)>) {
    match &**node {
        Node::Leaf { symbol, .. } => {
            result.push((*symbol, code));
        }
        Node::Internal { left, right, .. } => {
            extract_codes(left, code << false, result);
            extract_codes(right, code << true, result);
        }
    }
}

#[derive(Debug, Epserde, MemDbg, MemSize)]
/// A huffman tree that can be used to encode and decode values with optimal
/// prefix codes.
pub struct HuffmanTree<W = Vec<Code>, D = Vec<[CompactNode; 256]>> {
    pub write_table: W,
    pub decode_tree: D,
}

impl HuffmanTree {
    /// inner recursive function to build the decode tree
    fn build(&mut self, node: &Node, idx: usize, code: Code) {
        debug_assert_eq!(Self::ARRAY_LEN, self.decode_tree[idx].len());
        match node {
            Node::Leaf { symbol, .. } => {
                let len: usize = code.len as usize;
                let code = code.code as usize;
                let bits_left = Self::BITS.saturating_sub(len as usize);
                // fill all the codes that match the prefix
                for value in 0..(1 << bits_left).max(1) {
                    let offset = (code << bits_left) | value;
                    debug_assert!(self.decode_tree[idx][offset].is_empty());
                    self.decode_tree[idx][offset] = CompactNode::new_leaf(*symbol);
                }
            }
            Node::Internal { left, right, .. } => {
                // every 8-bit codes, create a sub node
                if code.len as usize == Self::BITS {
                    let new_idx = self.decode_tree.len();
                    self.decode_tree.push([CompactNode::empty(); Self::ARRAY_LEN]);
                    self.decode_tree[idx][code.code as usize] = CompactNode::new_index(new_idx);
                    self.build(left, new_idx, Code::default() << false);
                    self.build(right, new_idx, Code::default() << true);
                } else {
                    self.build(left, idx, code << false);
                    self.build(right, idx, code << true);
                }
            }
        }
    }

    /// Given a vector count of occourences, computes the huffman codes.
    pub fn new(counts: &[usize]) -> Result<Self> {
        ensure!(counts.len() > 0, "Empty counts");
        let mut nodes = counts
            .iter()
            .enumerate()
            .map(|(i, count)| Box::new(Node::new(count + 1, i)))
            .collect::<Vec<_>>();
        nodes.sort();

        // iteratively merge the two nodes with the lowest count
        while nodes.len() >= 2 {
            let left = nodes.pop().unwrap();
            let right = nodes.pop().unwrap();

            nodes.push(Box::new(Node::Internal {
                count: left.get_count() + right.get_count(),
                left,
                right,
            }));
            nodes.sort();
        }

        let root = nodes.pop().unwrap();
        let mut codes = Vec::new();
        extract_codes(&root, Code::default(), &mut codes);

        // build the write lookup tables
        let mut write_table = vec![Code::default(); counts.len()];
        for (symbol, code) in &codes {
            write_table[*symbol] = *code;
        }

        // build the decode tree
        let mut res = Self {
            write_table,
            decode_tree: vec![[CompactNode::empty(); Self::ARRAY_LEN]],
        };
        res.build(&*root, 0, Code::default());

        #[cfg(debug_assertions)]
        {
            // check that we filled all 
            for node in res.decode_tree.iter() {
                for compact_node in node.iter() {
                    assert!(!compact_node.is_empty());
                }
            }
        }

        Ok(res)
    }
}

impl<W, D> HuffmanTree<W, D> 
where
    W: AsRef<[Code]>,
    D: AsRef<[[CompactNode; 256]]>,
{
    const BITS: usize = 8;
    const ARRAY_LEN: usize = 1 << Self::BITS;

    #[inline(always)]
    /// Encodes a value using the huffman tree on the given writer and returns
    /// the number of bits written.
    pub fn encode<E: Endianness, BW: BitWrite<E>>(
        &self,
        value: u64,
        writer: &mut BW,
    ) -> Result<usize, BW::Error> {
        let write_table = self.write_table.as_ref();
        debug_assert!(value < write_table.len() as u64, "Symbol out of range");
        let code = write_table[value as usize];
        writer.write_bits(code.code as u64, code.len as usize)
    }

    #[inline(always)]
    /// Decodes a value using the huffman tree on the given reader and returns
    /// the decoded value.
    pub fn decode<E: Endianness, BR: BitRead<E> + BitSeek>(&self, reader: &mut BR) -> Result<u64, <BR as BitRead<E>>::Error> {
        let mut idx = 0;
        let decode_tree = self.decode_tree.as_ref();
        let mut bits_skipped = 0;
        loop {
            let node = &decode_tree[idx];
            let code = reader.peek_bits(Self::BITS)?;
            let index: u64 = code.cast();
            let compact_node = node[index as usize];

            debug_assert!(!compact_node.is_empty());

            if compact_node.is_leaf() {
                // skip only the bits of the code
                let len = self.write_table.as_ref()[compact_node.symbol() as usize].len as usize;
                reader.skip_bits_after_table_lookup(len - bits_skipped);
                return Ok(compact_node.symbol() as u64);
            }

            // move to the next bits
            reader.skip_bits_after_table_lookup(Self::BITS);
            bits_skipped += Self::BITS;
            idx = compact_node.index();
        }
    }

    fn debug_page(&self, idx: usize, depth: usize) {
        let decode_tree = self.decode_tree.as_ref();
        let node = &decode_tree[idx];
        for (i, compact_node) in node.iter().enumerate() {
            if compact_node.is_empty() {
                continue;
            }
            let symbol = if compact_node.is_leaf() {
                let symbol = compact_node.symbol();
                format!("{} {}", symbol, self.write_table.as_ref()[symbol])
            } else {
                format!("idx: {}", compact_node.index())
            };
            println!("{:width$} {:0pad$b} {}", "", i, symbol, pad = Self::BITS, width = depth * 4);
            if compact_node.is_index() {
                self.debug_page(compact_node.index(), depth + 1);
            }
        }
    }

    pub fn debug(&self) {
        self.debug_page(0, 0)
    }
}

#[cfg(test)]
mod test {
    use std::io::Seek;

    use mem_dbg::DbgFlags;

    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_huffman() {
        let counts = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let huffman = HuffmanTree::new(&counts).unwrap();

        huffman.mem_dbg(DbgFlags::default()).unwrap();

        let mut writer = BufBitWriter::<LittleEndian, _>::new(WordAdapter::<u32, _>::new(
            std::io::Cursor::new(Vec::new()),
        ));

        for i in 0..counts.len() {
            huffman.encode(i as u64, &mut writer).unwrap();
        }

        let mut data = writer.into_inner().unwrap().into_inner();
        data.seek(std::io::SeekFrom::Start(0)).unwrap();

        let mut reader = BufBitReader::<LittleEndian, _>::new(WordAdapter::<u32, _>::new(data));

        for i in 0..counts.len() {
            let code = &huffman.write_table[i];
            println!("value: {} code: {:0width$b} len: {width}", i, code.code, width = code.len as usize);
            assert_eq!(huffman.decode(&mut reader).unwrap(), i as u64);
        }
    }

    const ENGLISH: [usize; 256] = [
        5948740863, 470157844, 244335113, 175340730, 242746957, 147627855, 117905741, 102849596,
        254466929, 58803884, 72725791, 61087764, 73500788, 49865552, 240726723, 537829375, 204691693,
        56549435, 43709136, 30828876, 54564741, 127435143, 29060865, 27022401, 115521953, 25512714,
        22515862, 23749293, 37804277, 22690597, 47159611, 112862399, 233418066, 31600590, 26396567,
        22081700, 406207766, 49848215, 20868877, 21033828, 109031531, 58621773, 23188149, 28259325,
        36364848, 31128034, 67245703, 34067957, 106921614, 125418362, 42939663, 34385156, 44488680,
        45020239, 29207572, 25770313, 76573115, 93723595, 33147566, 32905643, 44602452, 37491413,
        22251340, 24565514, 109586208, 265555027, 85965062, 63048024, 221132535, 139662447, 47209016,
        50443765, 980961711, 182482091, 28123174, 30301706, 284688353, 83182955, 47246162, 30515337,
        95123802, 28342457, 35589800, 73404839, 80417545, 67621151, 37836903, 34245599, 49443492,
        23357774, 29301494, 41963134, 56388302, 56283573, 28452706, 123544867, 51761257, 118288986,
        45053121, 88662519, 91505023, 181092237, 176099237, 59682936, 61100072, 110615433, 24807458,
        30183554, 100910599, 59199677, 112507609, 129474453, 97805787, 17983461, 124068483, 104002251,
        255763472, 110838547, 47732715, 31291463, 61107590, 40830411, 20113473, 24599424, 56352918,
        42495518, 27342645, 29869998, 100007073, 38470896, 21528775, 210526018, 176976628, 174758205,
        35976983, 25119328, 48957611, 554790297, 14453470, 436477030, 40171412, 231584701, 29192425,
        22829915, 70929993, 17196393, 14497654, 18221749, 30555186, 22850103, 13529504, 13744010,
        31263260, 13890540, 12819117, 13049525, 23605287, 15933384, 12562855, 18207325, 40655903,
        13681238, 12659588, 14984124, 19128117, 14342876, 11863770, 12210870, 32583580, 12216880,
        25870345, 14121591, 22050597, 14899505, 12963533, 19150597, 40003125, 13367990, 12349776,
        14358426, 29143263, 21212765, 41790281, 22402291, 47003407, 25089705, 34001516, 19295132,
        34317828, 27628082, 38470027, 30768181, 160049365, 79364237, 51816391, 73087805, 61960427,
        40865482, 56906621, 96277810, 51397125, 40291416, 26569200, 17598238, 71470374, 93929614,
        22626536, 21271031, 58635620, 26828362, 40773934, 21112099, 20519108, 18909710, 23332669,
        19252776, 40659856, 19713439, 20417058, 26818730, 19413335, 20962367, 24274574, 49178892,
        58268026, 24820590, 32869407, 18322271, 28170175, 25750532, 26692622, 35468428, 172460464,
        105599778, 26721409, 49508046, 44843508, 29229598, 31379598, 56091129, 59003136, 22720528,
        44629038, 71920482, 23476793, 24112336, 50401710, 44511171, 67809887, 34491383, 64675732,
        46125325, 51362558, 65089092, 108451156, 888065159,
    ];
}
