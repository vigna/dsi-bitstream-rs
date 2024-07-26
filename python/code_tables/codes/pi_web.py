from math import ceil, floor, log2
from .pi import read_pi, write_pi, len_pi
from .fixed import read_fixed, write_fixed
from ..gen_table import gen_table_partial

def read_pi_web(bitstream, k, be):
    """Read a pi_web code"""
    flag, bitstream = read_fixed(1,bitstream, be)
    if flag != 0:
        return 0, bitstream
    return read_pi(bitstream, k, be)

def write_pi_web(value, k, bitstream, be):
    """Write a pi code"""
    if value == 0:
        return write_fixed(1, 1, bitstream, be)
    bitstream = write_fixed(0, 1, bitstream, be)
    return write_pi(value - 1, k, bitstream, be)

def len_pi_web(value, k):
    """Length of the pi_web code of `value`"""
    if value == 0:
        return 1
    else:
        return 1 + len_pi(value - 1, k) 


def gen_pi_web(path, read_bits, write_max_val, k=3, table_type="merged"):
    """Configuration of `gen_table` for pi_web"""
    gen_table_partial(
        "pi",
        lambda bitstream, be: read_pi_web(bitstream, k, be),
        lambda value, bitstream, be: write_pi_web(value, k, bitstream, be),
    )(
        path=path,
        read_bits=read_bits,
        write_max_val=write_max_val,
        table_type=table_type,
    )
    with open(path, "a") as f:
        f.write("/// The K of the pi_web codes for these tables\n")
        f.write("pub const K: u64 = {};".format(k))
