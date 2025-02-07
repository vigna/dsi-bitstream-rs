"""
Generate separate tables for the read / write type and the len, this makes the 
table smaller because it avoids alignment problems, but it requires two memory
accesses.
"""
import os

def gen_two_table(f, read_codes, write_codes, metadata):
    f.write("/// Precomputed read table\n")
    f.write(
        "pub const READ_{BO}: [{read_ty}; {array_len}] = [".format(
            array_len=len(read_codes),
            **metadata,
        )
    )
    for value, l in read_codes:
        f.write("{}, ".format(value or 0))
    f.write("];\n")

    f.write("/// Precomputed lengths table\n")
    f.write(
        "pub const READ_LEN_{BO}: [{read_len_ty}; {array_len}] = [".format(
            array_len=len(read_codes),
            **metadata,
        )
    )
    for value, l in read_codes:
        f.write("{}, ".format(l or metadata["missing"]))
    f.write("];\n")
    
    f.write("/// Precomputed write table\n")
    f.write(
        "pub const WRITE_{BO}: [{write_ty}; {array_len}] = [".format(
            array_len=len(write_codes),
            **metadata,
        )
    )
    for value, _ in write_codes:
        f.write("{},".format(value))
    f.write("];\n")

    if metadata["BO"] == "BE":
        f.write("/// Precomputed write len table\n")
        f.write(
            "pub const WRITE_LEN: [{write_len_ty}; {array_len}] = [".format(
                array_len=len(write_codes),
                **metadata,
            )
        )
        for _, l in write_codes:
            f.write("{}, ".format(l))
        f.write("];\n")

ROOT = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(ROOT, "two_tables_template.rs")) as f:
    funcs_two_table = f.read()