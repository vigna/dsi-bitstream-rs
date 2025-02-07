from .gamma import read_gamma, write_gamma, len_gamma
from .fixed import read_fixed, write_fixed
from ..gen_table import gen_table_partial

def read_exp_golomb(bitstream, k, be):
    """Read a exp_golomb code"""
    first,  bitstream = read_gamma(bitstream, be)
    second, bitstream = read_fixed(k, bitstream, be)
    return (first << k) + second, bitstream

def write_exp_golomb(value, k, bitstream, be):
    """Write a pi code"""
    bitstream = write_gamma(value >> k, bitstream, be)
    bitstream = write_fixed(value % (1 << k), k, bitstream, be)
    return bitstream

def len_exp_golomb(value, k):
    """Length of the exp_golomb code of `value`"""
    return len_gamma(value >> k) + k

def gen_exp_golomb(path, read_bits, write_max_val, k, table_type="merged"):
    """Configuration of `gen_table` for exp_golomb"""
    gen_table_partial(
        "exp_golomb",
        lambda bitstream, be: read_exp_golomb(bitstream, k, be),
        lambda value, bitstream, be: write_exp_golomb(value, k, bitstream, be),
    )(
        path=path,
        read_bits=read_bits,
        write_max_val=write_max_val,
        table_type=table_type,
    )
    with open(path, "a") as f:
        f.write("/// The k of the exp_golomb codes for these tables\n")
        f.write("pub const K: u64 = {};".format(k))


# Test that the impl is reasonable
assert write_exp_golomb(0, 2, "", True) == "100"
assert write_exp_golomb(1, 2, "", True) == "101"
assert write_exp_golomb(2, 2, "", True) == "110"
assert write_exp_golomb(3, 2, "", True) == "111"
assert write_exp_golomb(4, 2, "", True) == "01000"
assert write_exp_golomb(5, 2, "", True) == "01001"
assert write_exp_golomb(6, 2, "", True) == "01010"
assert write_exp_golomb(7, 2, "", True) == "01011"

assert write_exp_golomb(0, 3, "", True) == "1000"
assert write_exp_golomb(1, 3, "", True) == "1001"
assert write_exp_golomb(2, 3, "", True) == "1010"
assert write_exp_golomb(3, 3, "", True) == "1011"
assert write_exp_golomb(4, 3, "", True) == "1100"
assert write_exp_golomb(5, 3, "", True) == "1101"
assert write_exp_golomb(6, 3, "", True) == "1110"
assert write_exp_golomb(7, 3, "", True) == "1111"
assert write_exp_golomb(8, 3, "", True) == "010000"

# Little consistency check
for k in range(2, 5):
    for i in range(256):
        for be in [True, False]:
            l = len_exp_golomb(i, k)  # NoQA: E741

            wbe = write_exp_golomb(i, k, "", be)
            rbe = read_exp_golomb(wbe, k, be)[0]

            assert i == rbe, "k: %s i: %s rbe: %s wbe: %s be?: %s" % (k, i, rbe, wbe, be)
            assert len(wbe) == l