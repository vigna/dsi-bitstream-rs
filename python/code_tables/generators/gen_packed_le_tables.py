"""
See doc of gen_packed_be.
"""
import os

def gen_packed_le_table(f, read_codes, write_codes, metadata):
    f.write("/// Precomputed read table with packed {read_ty} and {read_len_ty}\n".format(**metadata))
    f.write(
        "pub const READ_{BO}: [u8; {array_len}] = [".format(
            array_len=len(read_codes) * (metadata["read_bytes"] + metadata["read_len_bytes"]),
            **metadata,
        )
    )
    
    for value, l in read_codes:
        value = value or 0
        for i in range(metadata["read_bytes"]):
            f.write("{}, ".format(value & 0xFF))
            value >>= 8
        l = l or metadata["missing"]
        for i in range(metadata["read_len_bytes"]):
            f.write("{}, ".format(l & 0xFF))
            l >>= 8
        
    f.write("];\n")
    
    f.write("/// Precomputed write table with packed {write_ty} and {write_len_ty}\n".format(**metadata))
    f.write(
        "pub const WRITE_{BO}: [u8; {array_len}] = [".format(
            array_len=len(write_codes) * (metadata["write_bytes"] + metadata["write_len_bytes"]),
            **metadata,
        )
    )
    for value, l in write_codes:
        for i in range(metadata["write_bytes"]):
            f.write("{}, ".format(value & 0xFF))
            value >>= 8
            
        for i in range(metadata["write_len_bytes"]):
            f.write("{}, ".format(l & 0xFF))
            l >>= 8
            
    f.write("];\n")


ROOT = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(ROOT, "packed_le_template.rs")) as f:
    funcs_packed_le = f.read()
