"""
Merged tables are (value, len) which improves cache locality, but might waste
some memory due to alignment.
"""
import os

def gen_merged_table(f, read_codes, write_codes, metadata):
    
    f.write("/// Precomputed read table\n")
    f.write(
        "pub const READ_{BO}: [({read_ty}, {read_len_ty}); {len}] = &[".format(
            len=len(read_codes),
            **metadata,
        )
    )
    for value, l in read_codes:
        f.write("({}, {}), ".format(value or 0, l or metadata["missing"]))
    f.write("];\n")
    
    f.write("/// Precomputed write table\n")
    f.write(
        "pub const WRITE_{BO}: [({write_ty}, {write_len_ty}); {len}] = [".format(
            len=len(write_codes),
            **metadata,
        )
    )
    for value, l in write_codes:
        f.write("({}, {}),".format(value, l))
    f.write("];\n")
    
ROOT = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(ROOT, "merged_template.rs")) as f:
    funcs_merged_table = f.read()
    