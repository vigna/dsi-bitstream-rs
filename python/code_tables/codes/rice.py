from .unary import read_unary, write_unary, len_unary
from .fixed import read_fixed, write_fixed
from ..gen_table import gen_table_partial

def read_rice(bitstream, log_b, be):
    """Read a rice code"""
    first,  bitstream = read_unary(bitstream, be)
    second, bitstream = read_fixed(log_b, bitstream, be)
    return (first << log_b) + second, bitstream

def write_rice(value, log_b, bitstream, be):
    """Write a pi code"""
    bitstream = write_unary(value >> log_b, bitstream, be)
    bitstream = write_fixed(value % (1 << log_b), log_b, bitstream, be)
    return bitstream

def len_rice(value, log_b):
    """Length of the rice code of `value`"""
    return len_unary(value >> log_b) + log_b

def gen_rice(path, read_bits, write_max_val, log_b, table_type="merged"):
    """Configuration of `gen_table` for rice"""
    gen_table_partial(
        "rice",
        lambda bitstream, be: read_rice(bitstream, log_b, be),
        lambda value, bitstream, be: write_rice(value, log_b, bitstream, be),
    )(
        path=path,
        read_bits=read_bits,
        write_max_val=write_max_val,
        table_type=table_type,
    )
    with open(path, "a") as f:
        f.write("/// The LOG_B of the rice codes for these tables\n")
        f.write("pub const LOG_B: u64 = {};".format(log_b))


# Test that the impl is reasonable
assert write_rice(0, 2, "", True) == "100"
assert write_rice(1, 2, "", True) == "101"
assert write_rice(2, 2, "", True) == "110"
assert write_rice(3, 2, "", True) == "111"
assert write_rice(4, 2, "", True) == "0100"
assert write_rice(5, 2, "", True) == "0101"
assert write_rice(6, 2, "", True) == "0110"
assert write_rice(7, 2, "", True) == "0111"

assert write_rice(0, 3, "", True) == "1000"
assert write_rice(1, 3, "", True) == "1001"
assert write_rice(2, 3, "", True) == "1010"
assert write_rice(3, 3, "", True) == "1011"
assert write_rice(4, 3, "", True) == "1100"
assert write_rice(5, 3, "", True) == "1101"
assert write_rice(6, 3, "", True) == "1110"
assert write_rice(7, 3, "", True) == "1111"
assert write_rice(8, 3, "", True) == "01000"

# Little consistency check
for log_b in range(2, 5):
    for i in range(256):
        for be in [True, False]:
            l = len_rice(i, log_b)  # NoQA: E741

            wbe = write_rice(i, log_b, "", be)
            rbe = read_rice(wbe, log_b, be)[0]

            assert i == rbe, "log_b: %s i: %s rbe: %s wbe: %s be?: %s" % (log_b, i, rbe, wbe, be)
            assert len(wbe) == l