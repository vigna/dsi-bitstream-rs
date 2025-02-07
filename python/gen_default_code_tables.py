#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
# SPDX-FileCopyrightText: 2023 Sebastiano Vigna
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""
To run just execute `$ python ./gen_default_code_tables.py`
To provide a build folder, pass it as the first positional argument.

This script is not implemented using the `build.rs` mechanism because 
it would significantly slow down the build process. Moreover, the tables
will be generated very rarely.
"""
import os
import subprocess
from code_tables import *


ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROOT = os.path.join(ROOT, "src", "codes")

def generate_default_tables():
    # Generate the default tables
    gen_gamma(
        os.path.join(ROOT, "gamma_tables.rs"),
        read_bits=9, # No use on Xeon/ARM, little useful on i7
        write_max_val=63,
        table_type="separated", # Irrelevant for speed, a bit smaller
    )
    gen_delta(
        os.path.join(ROOT, "delta_tables.rs"),
        read_bits=11, # No use on any architecture if 9-bit gamma tables are available, but just in case someone selects it
        write_max_val=1023, # Very useful, both tables (delta and gamma)
        table_type="separated",
    )
    for k in [3]:
        gen_zeta(
            os.path.join(ROOT, f"zeta{k}_tables.rs"),
            read_bits=12, # Necessary for all architectures
            write_max_val=1023, # Very useful   
            k=k,
            table_type="separated", # A bit better on ARM, very slightly worse on i7, same on Xeon
        )
    for k in [2, 3]:
        gen_pi(
            os.path.join(ROOT, f"pi{k}_tables.rs"),
            read_bits=12, # Necessary for all architectures
            write_max_val=1023, # Very useful   
            k=k,
            table_type="separated", # A bit better on ARM, very slightly worse on i7, same on Xeon
        )
    #for k in [2]:
    #    gen_pi_web(
    #        os.path.join(ROOT, f"pi_web{k}_tables.rs"),
    #        read_bits=12, # Necessary for all architectures
    #        write_max_val=1023, # Very useful   
    #        k=k,
    #        table_type="packed_le", # A bit better on ARM, very slightly worse on i7, same on Xeon
    #    )
    #for b in [16]:
    #    gen_golomb(
    #        os.path.join(ROOT, f"golomb{b}_tables.rs"),
    #        read_bits=12,
    #        write_max_val=1023,
    #        b=b,
    #        table_type="packed_le",
    #    )
    #for log_b in [5]:
    #    gen_rice(
    #        os.path.join(ROOT, f"rice{log_b}_tables.rs"),
    #        read_bits=12,
    #        write_max_val=1023,
    #        log_b=log_b,
    #        table_type="packed_le",
    #    )
    #for k in [2]:
    #    gen_exp_golomb(
    #        os.path.join(ROOT, f"exp_golomb{k}_tables.rs"),
    #        read_bits=12,
    #        write_max_val=1023,
    #        k=k,
    #        table_type="packed_le",
    #    )
    
    subprocess.check_call(
        "cargo fmt", shell=True,
    )

if __name__ == "__main__":
    generate_default_tables()
