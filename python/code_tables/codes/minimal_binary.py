from math import floor, log2
from .fixed import read_fixed, write_fixed

def read_minimal_binary(max, bitstream, be):
    """Read a minimal binary code code with max `max`"""
    l = int(floor(log2(max)))  # NoQA: E741
    v, bitstream = read_fixed(l, bitstream, be)
    limit = (1 << (l + 1)) - max

    if v < limit:
        return v, bitstream
    else:
        b, bitstream = read_fixed(1, bitstream, be)
        v = (v << 1) | b
        return v - limit, bitstream


def write_minimal_binary(value, max, bitstream, be):
    """Write a minimal binary code with max `max`"""
    l = int(floor(log2(max)))  # NoQA: E741
    limit = (1 << (l + 1)) - max

    if value < limit:
        return write_fixed(value, l, bitstream, be)
    else:
        to_write = value + limit
        bitstream = write_fixed(to_write >> 1, l, bitstream, be)
        return write_fixed(to_write & 1, 1, bitstream, be)


def len_minimal_binary(value, max):
    """Length of the minimal binary code of `value` with max `max`"""
    l = int(floor(log2(max)))  # NoQA: E741
    limit = (1 << (l + 1)) - max
    if value >= limit:
        return l + 1
    else:
        return l


# Test that the impl is reasonable
assert write_minimal_binary(0, 10, "", True) == "000"
assert write_minimal_binary(0, 10, "", False) == "000"
assert write_minimal_binary(1, 10, "", True) == "001"
assert write_minimal_binary(1, 10, "", False) == "001"
assert write_minimal_binary(2, 10, "", True) == "010"
assert write_minimal_binary(2, 10, "", False) == "010"
assert write_minimal_binary(3, 10, "", True) == "011"
assert write_minimal_binary(3, 10, "", False) == "011"
assert write_minimal_binary(4, 10, "", True) == "100"
assert write_minimal_binary(4, 10, "", False) == "100"
assert write_minimal_binary(5, 10, "", True) == "101"
assert write_minimal_binary(5, 10, "", False) == "101"

assert write_minimal_binary(6, 10, "", True) == "1100"
assert write_minimal_binary(6, 10, "", False) == "0110"
assert write_minimal_binary(7, 10, "", True) == "1101"
assert write_minimal_binary(7, 10, "", False) == "1110"
assert write_minimal_binary(8, 10, "", True) == "1110"
assert write_minimal_binary(8, 10, "", False) == "0111"
assert write_minimal_binary(9, 10, "", True) == "1111"
assert write_minimal_binary(9, 10, "", False) == "1111"

# Little consistency check
_max = 200
for i in range(_max):
    wbe = write_minimal_binary(i, _max, "", True)
    rbe = read_minimal_binary(_max, wbe, True)[0]
    wle = write_minimal_binary(i, _max, "", False)
    rle = read_minimal_binary(_max, wle, False)[0]
    l = len_minimal_binary(i, _max)  # NoQA: E741
    assert i == rbe
    assert i == rle
    assert len(wbe) == l
    assert len(wle) == l