from math import ceil, floor, log2
from .unary import read_unary, write_unary, len_unary
from .fixed import read_fixed, write_fixed
from ..gen_table import gen_table_partial

def read_pi(bitstream, k, be):
    """Read a pi code"""
    l, bitstream = read_unary(bitstream, be)
    l += 1
    v, bitstream = read_fixed(k, bitstream, be)
    h = (l * (1 << k)) - v
    r = h - 1
    rem, bitstream = read_fixed(r, bitstream, be)
    return (1 << r) + rem - 1, bitstream

def write_pi(value, k, bitstream, be):
    """Write a pi code"""
    value += 1
    r = floor(log2(value))
    h = 1 + r
    l = ceil(h / (1 << k))
    v = l * (1 << k) - h
    rem = value & ((1 << r) - 1)
    
    bitstream = write_unary(l - 1, bitstream, be)
    bitstream = write_fixed(v, k, bitstream, be)
    bitstream = write_fixed(rem, r, bitstream, be)
    return bitstream

def len_pi(value, k):
    """Length of the pi code of `value`"""
    value += 1
    rem = floor(log2(value))
    h = 1 + rem
    l = ceil(h / (1 << k))
    return k + l + rem

# Test that the impl is reasonable
assert write_pi(1, 2, "", True) == "1100"
assert write_pi(2, 2, "", True) == "1101"
assert write_pi(3, 2, "", True) == "10100"
assert write_pi(4, 2, "", True) == "10101"
assert write_pi(5, 2, "", True) == "10110"
assert write_pi(6, 2, "", True) == "10111"
assert write_pi(7, 2, "", True) == "100000"

assert write_pi(1, 3, "", True) == "11100"
assert write_pi(2, 3, "", True) == "11101"
assert write_pi(3, 3, "", True) == "110100"
assert write_pi(4, 3, "", True) == "110101"
assert write_pi(5, 3, "", True) == "110110"
assert write_pi(6, 3, "", True) == "110111"
assert write_pi(7, 3, "", True) == "1100000"


# Little consistency check
for i in range(256):
    l = len_pi(i, 3)  # NoQA: E741

    wbe = write_pi(i, 3, "", True)
    rbe = read_pi(wbe, 3, True)[0]

    assert i == rbe, "%s %s %s" % (i, rbe, wbe)
    assert len(wbe) == l

    wle = write_pi(i, 3, "", False)
    rle = read_pi(wle, 3, False)[0]

    assert i == rle, "%s %s %s" % (i, rle, wle)
    assert len(wle) == l


def gen_pi(path, read_bits, write_max_val, k=3, table_type="merged"):
    """Configuration of `gen_table` for pi"""
    gen_table_partial(
        "pi",
        lambda bitstream, be: read_pi(bitstream, k, be),
        lambda value, bitstream, be: write_pi(value, k, bitstream, be),
    )(
        path=path,
        read_bits=read_bits,
        write_max_val=write_max_val,
        table_type=table_type,
    )
    with open(path, "a") as f:
        f.write("/// The K of the pi codes for these tables\n")
        f.write("pub const K: u64 = {};".format(k))
