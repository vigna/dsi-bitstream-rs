"""
Like Merged, but avoid alignment problems by manually putting the bytes one
after the other. This is as small as two tables, with the cache locality of 
merged. The downside is that it requires more complex indexing, and we need
to explicitly write the bytes with an endianness, which might not match the 
target. This one writes it in big endian, while packed_le writes them in little
endian.
"""
import os

def gen_packed_be_table(f, read_codes, write_codes, metadata):
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
            f.write("{}, ".format((value >> (8 * i)) & 0xFF))
        l = l or metadata["missing"]
        for i in range(metadata["read_len_bytes"]):
            f.write("{}, ".format((l >> (8 * i)) & 0xFF))
        
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
            f.write("{}, ".format((value >> (8 * i)) & 0xFF))
            
        for i in range(metadata["write_len_bytes"]):
            f.write("{}, ".format((l >> (8 * i)) & 0xFF))
            
    f.write("];\n")


ROOT = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(ROOT, "packed_be_template.rs")) as f:
    funcs_packed_be = f.read()