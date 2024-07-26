
def read_fixed(n_bits, bitstream, be):
    """Read a fixed number of bits"""
    if len(bitstream) < n_bits:
        raise ValueError()

    if n_bits == 0:
        return 0, bitstream

    if be:
        return int(bitstream[:n_bits], 2), bitstream[n_bits:]
    else:
        return int(bitstream[-n_bits:], 2), bitstream[:-n_bits]


def write_fixed(value, n_bits, bitstream, be):
    """Write a fixed number of bits"""
    if n_bits == 0:
        return bitstream
    if be:
        return bitstream + ("{:0%sb}" % n_bits).format(value)
    else:
        return ("{:0%sb}" % n_bits).format(value) + bitstream