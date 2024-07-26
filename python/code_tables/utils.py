
def get_best_fitting_type(n_bits):
    """Find the smallest Rust type that can fit n_bits"""
    if n_bits <= 8:
        return "u8"
    if n_bits <= 16:
        return "u16"
    if n_bits <= 32:
        return "u32"
    if n_bits <= 64:
        return "u64"
    if n_bits <= 128:
        return "u128"
    raise ValueError(n_bits)