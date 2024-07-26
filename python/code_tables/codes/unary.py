from ..gen_table import gen_table_partial

def read_unary(bitstream, be):
    """Read an unary code"""
    if be:
        l = len(bitstream) - len(bitstream.lstrip("0"))  # NoQA: E741
        if l == len(bitstream):
            raise ValueError()
        return l, bitstream[l + 1 :]
    else:
        l = len(bitstream) - len(bitstream.rstrip("0"))  # NoQA: E741
        if l == len(bitstream):
            raise ValueError()
        return l, bitstream[: -l - 1]


def write_unary(value, bitstream, be):
    """Write an unary code"""
    if be:
        return bitstream + "0" * value + "1"
    else:
        return "1" + "0" * value + bitstream


def len_unary(value):
    """The len of an unary code for value"""
    return value + 1


# Test that the impl is reasonable
assert write_unary(0, "", True) == "1"
assert write_unary(0, "", False) == "1"
assert write_unary(1, "", True) == "01"
assert write_unary(1, "", False) == "10"
assert write_unary(2, "", True) == "001"
assert write_unary(2, "", False) == "100"
assert write_unary(3, "", True) == "0001"
assert write_unary(3, "", False) == "1000"

# Little consistency check
for i in range(256):
    wbe = write_unary(i, "", True)
    rbe = read_unary(wbe, True)[0]
    wle = write_unary(i, "", False)
    rle = read_unary(wle, False)[0]
    l = len_unary(i)  # NoQA: E741
    assert i == rbe
    assert i == rle
    assert len(wbe) == l
    assert len(wle) == l


gen_unary = gen_table_partial("unary", read_unary, write_unary)
