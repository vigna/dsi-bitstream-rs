from math import floor, log2
from .unary import read_unary, write_unary
from .fixed import read_fixed, write_fixed
from ..gen_table import gen_table_partial

def read_gamma(bitstream, be):
    """Read a gamma code"""
    l, bitstream = read_unary(bitstream, be)
    if l == 0:
        return 0, bitstream
    f, bitstream = read_fixed(l, bitstream, be)
    v = f + (1 << l) - 1
    return v, bitstream


def write_gamma(value, bitstream, be):
    """Write a gamma code"""
    value += 1
    l = floor(log2(value))  # NoQA: E741
    s = value - (1 << l)
    bitstream = write_unary(l, bitstream, be)
    if l != 0:
        bitstream = write_fixed(s, l, bitstream, be)
    return bitstream


def len_gamma(value):
    """Length of the gamma code of `value`"""
    value += 1
    l = floor(log2(value))  # NoQA: E741
    return 2 * l + 1


# Test that the impl is reasonable
assert write_gamma(0, "", True) == "1"
assert write_gamma(0, "", False) == "1"
assert write_gamma(1, "", True) == "010"
assert write_gamma(1, "", False) == "010"
assert write_gamma(2, "", True) == "011"
assert write_gamma(2, "", False) == "110"
assert write_gamma(3, "", True) == "00100"
assert write_gamma(3, "", False) == "00100"
assert write_gamma(4, "", True) == "00101"
assert write_gamma(4, "", False) == "01100"
assert write_gamma(5, "", True) == "00110"
assert write_gamma(5, "", False) == "10100"

# Little consistency check
for i in range(256):
    wbe = write_gamma(i, "", True)
    rbe = read_gamma(wbe, True)[0]
    wle = write_gamma(i, "", False)
    rle = read_gamma(wle, False)[0]
    l = len_gamma(i)  # NoQA: E741
    assert i == rbe
    assert i == rle
    assert len(wbe) == l
    assert len(wle) == l

gen_gamma = gen_table_partial("gamma", read_gamma, write_gamma)
