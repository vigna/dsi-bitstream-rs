from math import floor, log2
from .gamma import read_gamma, write_gamma, len_gamma
from .fixed import read_fixed, write_fixed
from ..gen_table import gen_table_partial

def read_delta(bitstream, be):
    """Read a delta code"""
    l, bitstream = read_gamma(bitstream, be)
    if l == 0:
        return 0, bitstream
    f, bitstream = read_fixed(l, bitstream, be)
    v = f + (1 << l) - 1
    return v, bitstream


def write_delta(value, bitstream, be):
    """Write a delta code"""
    value += 1
    l = floor(log2(value))  # NoQA: E741
    s = value - (1 << l)
    bitstream = write_gamma(l, bitstream, be)
    if l != 0:
        bitstream = write_fixed(s, l, bitstream, be)
    return bitstream


def len_delta(value):
    """Length of the delta code of `value`"""
    value += 1
    l = floor(log2(value))  # NoQA: E741
    return l + len_gamma(l)


# Test that the impl is reasonable
assert write_delta(0, "", True) == "1"
assert write_delta(0, "", False) == "1"
assert write_delta(1, "", True) == "0100"
assert write_delta(1, "", False) == "0010"
assert write_delta(2, "", True) == "0101"
assert write_delta(2, "", False) == "1010"
assert write_delta(3, "", True) == "01100"
assert write_delta(3, "", False) == "00110"
assert write_delta(4, "", True) == "01101"
assert write_delta(4, "", False) == "01110"
assert write_delta(5, "", True) == "01110"
assert write_delta(5, "", False) == "10110"

# Little consistency check
for i in range(256):
    wbe = write_delta(i, "", True)
    rbe = read_delta(wbe, True)[0]
    wle = write_delta(i, "", False)
    rle = read_delta(wle, False)[0]
    l = len_delta(i)  # NoQA: E741
    assert i == rbe
    assert i == rle
    assert len(wbe) == l
    assert len(wle) == l

gen_delta = gen_table_partial("delta", read_delta, write_delta)