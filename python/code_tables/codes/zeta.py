from math import floor, log2
from .unary import read_unary, write_unary, len_unary
from .minimal_binary import read_minimal_binary, write_minimal_binary, len_minimal_binary
from ..gen_table import gen_table_partial

def read_zeta(bitstream, k, be):
    """Read a zeta code"""
    h, bitstream = read_unary(bitstream, be)
    u = 2 ** ((h + 1) * k)
    l = 2 ** (h * k)  # NoQA: E741
    r, bitstream = read_minimal_binary(u - l, bitstream, be)
    return l + r - 1, bitstream


def write_zeta(value, k, bitstream, be):
    """Write a zeta code"""
    value += 1
    h = int(floor(log2(value)) / k)
    u = 2 ** ((h + 1) * k)
    l = 2 ** (h * k)  # NoQA: E741

    bitstream = write_unary(h, bitstream, be)
    bitstream = write_minimal_binary(value - l, u - l, bitstream, be)
    return bitstream


def len_zeta(value, k):
    """Length of the zeta code of `value`"""
    value += 1
    h = int(floor(log2(value)) / k)
    u = 2 ** ((h + 1) * k)
    l = 2 ** (h * k)  # NoQA: E741
    return len_unary(h) + len_minimal_binary(value - l, u - l)


# Test that the impl is reasonable
assert write_zeta(0, 3, "", True) == "100"
assert write_zeta(1, 3, "", True) == "1010"
assert write_zeta(2, 3, "", True) == "1011"
assert write_zeta(3, 3, "", True) == "1100"
assert write_zeta(4, 3, "", True) == "1101"
assert write_zeta(5, 3, "", True) == "1110"
assert write_zeta(6, 3, "", True) == "1111"
assert write_zeta(7, 3, "", True) == "0100000"
assert write_zeta(8, 3, "", True) == "0100001"

assert write_zeta(0, 3, "", False) == "001"
assert write_zeta(1, 3, "", False) == "0011"
assert write_zeta(2, 3, "", False) == "1011"
assert write_zeta(3, 3, "", False) == "0101"
assert write_zeta(4, 3, "", False) == "1101"
assert write_zeta(5, 3, "", False) == "0111"
assert write_zeta(6, 3, "", False) == "1111"
assert write_zeta(7, 3, "", False) == "0000010"
assert write_zeta(8, 3, "", False) == "0000110"

# Little consistency check
for i in range(256):
    l = len_zeta(i, 3)  # NoQA: E741

    wbe = write_zeta(i, 3, "", True)
    rbe = read_zeta(wbe, 3, True)[0]

    assert i == rbe, "%s %s %s" % (i, rbe, wbe)
    assert len(wbe) == l

    wle = write_zeta(i, 3, "", False)
    rle = read_zeta(wle, 3, False)[0]

    assert i == rle, "%s %s %s" % (i, rle, wle)
    assert len(wle) == l



def gen_zeta(path, read_bits, write_max_val, k=3, table_type="merged"):
    """Configuration of `gen_table` for zeta"""
    gen_table_partial(
        "zeta",
        lambda bitstream, be: read_zeta(bitstream, k, be),
        lambda value, bitstream, be: write_zeta(value, k, bitstream, be),
    )(
        path=path,
        read_bits=read_bits,
        write_max_val=write_max_val,
        table_type=table_type,
    )
    with open(path, "a") as f:
        f.write("/// The K of the zeta codes for these tables\n")
        f.write("pub const K: u64 = {};".format(k))
