from .unary import read_unary, write_unary, len_unary
from .minimal_binary import read_minimal_binary, write_minimal_binary, len_minimal_binary
from ..gen_table import gen_table_partial

def read_golomb(bitstream, b, be):
    """Read a golomb code"""
    first,  bitstream = read_unary(bitstream, be)
    second, bitstream = read_minimal_binary(b, bitstream, be)
    return first * b + second, bitstream

def write_golomb(value, b, bitstream, be):
    """Write a pi code"""
    bitstream = write_unary(value // b, bitstream, be)
    bitstream = write_minimal_binary(value % b, b, bitstream, be)
    return bitstream

def len_golomb(value, b):
    """Length of the golomb code of `value`"""
    return len_unary(value // b) + len_minimal_binary(value % b, b)


def gen_golomb(path, read_bits, write_max_val, b, table_type="merged"):
    """Configuration of `gen_table` for golomb"""
    gen_table_partial(
        "golomb",
        lambda bitstream, be: read_golomb(bitstream, b, be),
        lambda value, bitstream, be: write_golomb(value, b, bitstream, be),
    )(
        path=path,
        read_bits=read_bits,
        write_max_val=write_max_val,
        table_type=table_type,
    )
    with open(path, "a") as f:
        f.write("/// The B of the golomb codes for these tables\n")
        f.write("pub const B: u64 = {};".format(b))


# Test that the impl is reasonable
assert write_golomb(0, 2, "", True) == "10"
assert write_golomb(1, 2, "", True) == "11"
assert write_golomb(2, 2, "", True) == "010"
assert write_golomb(3, 2, "", True) == "011"
assert write_golomb(4, 2, "", True) == "0010"
assert write_golomb(5, 2, "", True) == "0011"
assert write_golomb(6, 2, "", True) == "00010"
assert write_golomb(7, 2, "", True) == "00011"

assert write_golomb(0, 3, "", True) == "10"
assert write_golomb(1, 3, "", True) == "110"
assert write_golomb(2, 3, "", True) == "111"
assert write_golomb(3, 3, "", True) == "010"
assert write_golomb(4, 3, "", True) == "0110"
assert write_golomb(5, 3, "", True) == "0111"
assert write_golomb(6, 3, "", True) == "0010"
assert write_golomb(7, 3, "", True) == "00110"

# Little consistency check
for b in range(2, 5):
    for i in range(256):
        l = len_golomb(i, b)  # NoQA: E741

        wbe = write_golomb(i, b, "", True)
        rbe = read_golomb(wbe, b, True)[0]

        assert i == rbe, "%s %s %s" % (i, rbe, wbe)
        assert len(wbe) == l

        wle = write_golomb(i, b, "", False)
        rle = read_golomb(wle, b, False)[0]

        assert i == rle, "%s %s %s" % (i, rle, wle)
        assert len(wle) == l