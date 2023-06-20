#!/usr/bin/python3
"""
To run it needs the following:
cargo install rustfilt
rustup component add --toolchain nightly llvm-tools-preview
"""
import os
import re
import sys
import argparse
import subprocess
from glob import glob

def compute_coverage(FUZZING, TESTS, DOC_TESTS, SHOW, final_cov_path):
    if not any((FUZZING, TESTS, DOC_TESTS)):
        raise ValueError("You need to set at least one of FUZZING, TESTS, DOC_TESTS to True")

    ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    target_folder = os.path.join(ROOT, "target")
    # Get info about the rust installation
    rustup_info = subprocess.check_output("rustup show", shell=True).decode()
    arch = re.findall(r"Default host: (.+)", rustup_info)[0]

    # Get where LLVM is installed
    sysroot = subprocess.check_output("rustc --print sysroot", shell=True).decode().strip()
    llvm_path = os.path.join(sysroot, "lib", "rustlib", arch, "bin")

    # Check that people can read the doc
    if not os.path.exists(os.path.join(llvm_path, "llvm-profdata")):
        print("PLEASE run:")
        print("rustup component add --toolchain nightly llvm-tools-preview")
        print("cargo install rustfilt")
        sys.exit(1)

    # Settings
    test_cov_path = os.path.join(target_folder, "cargo-test-cov-%p-%m.profraw")
    final_cov_path = os.path.join(target_folder, "total_coverage.profdata")
    doc_tests_bin = os.path.join(target_folder, "doctestbins")

    cov_files = []
    exec_files = []

    # Create a folder for the test coverage
    os.makedirs(target_folder, exist_ok=True)

    if DOC_TESTS or TESTS:
        # Clean up the targets folder and ensure that everything is insturmented
        subprocess.check_call(
            "cargo clean",
            shell=True,
            cwd=ROOT,
        )

        env = {
            **os.environ,
            "CARGO_INCREMENTAL":"0",
            "RUSTFLAGS": "-C instrument-coverage",
            "LLVM_PROFILE_FILE": test_cov_path,
        }
        if DOC_TESTS:
            env["RUSTDOCFLAGS"] = "-Cinstrument-coverage -Z unstable-options --persist-doctests {}".format(doc_tests_bin)
        else:
            env["RUSTDOCFLAGS"] = "-Cinstrument-coverage -Z unstable-options"


        out = subprocess.check_output(
            "cargo test",
            shell=True,
            cwd=ROOT,
            stderr=subprocess.STDOUT,
            universal_newlines=True,
            env=env,
        )

        # For the doctests see:
        # https://github.com/rust-lang/rust/issues/79417

        # add the test binaries
        exec_files.extend(
            os.path.join(ROOT, file)
            for file in re.findall(r"Running \S+ \((target/.+?)\)", out)
        )
        if DOC_TESTS:
            # add all the doc tests binaries
            exec_files.extend(
                file
                for file in glob(os.path.join(doc_tests_bin, "**", "*"))
            )
        cov_files.extend(
            os.path.join(os.path.dirname(test_cov_path), file)
            for file in os.listdir(os.path.dirname(test_cov_path))
            if file.endswith(".profraw")
        )

    if FUZZING:
        # Get the list of fuzzing targets
        fuzz_targets = (
            subprocess.check_output(
                "cargo fuzz list",
                shell=True,
                cwd=ROOT,
            )
            .decode()
            .split("\n")[:-1]
        )
        # Generate coverage for all the targets
        for fuzz_target in fuzz_targets:
            subprocess.check_call(
                "cargo fuzz coverage {}".format(fuzz_target),
                shell=True,
                cwd=ROOT,
            )

        cov_files.extend(
            os.path.join(ROOT, "fuzz", "coverage", fuzz_target, "coverage.profdata")
            for fuzz_target in fuzz_targets
        )
        exec_files.extend(
            os.path.join(
                ROOT, "target", arch, "coverage", arch, "release", fuzz_target
            )
            for fuzz_target in fuzz_targets
        )

    # Merge the coverages into an unique file
    subprocess.check_call(
        "{}/llvm-profdata merge -sparse {} -o {}".format(
            llvm_path,
            " ".join(cov_files),
            final_cov_path,
        ),
        shell=True,
        cwd=ROOT,
    )
    if SHOW:
        # Create the report!
        subprocess.check_call(
            (
                "{}/llvm-cov report --Xdemangler=rustfilt --instr-profile={} "
                "-ignore-filename-regex='.cargo' -object {}"
            ).format(
                llvm_path,
                final_cov_path,
                " -object ".join(exec_files),
            ),
            shell=True,
            cwd=ROOT,
        )

if __name__ == "__main__":
    
    parser = argparse.ArgumentParser(description='Compute the coverage of the tests, doc test, and fuzzing')

    parser.add_argument('-f', '--fuzzing', action='store_true', help='Compute the coverage of the fuzzing')
    parser.add_argument('-t', '--tests', action='store_true', help='Compute the coverage of the tests')
    parser.add_argument('-d', '--doc-tests', action='store_true', help='Compute the coverage of the doc tests')
    parser.add_argument('-n', '--no-show', action='store_false', help='Show the coverage')

    parser.add_argument('-c', '--final-coverage-file', type=str, default="target/total_coverage.profdata", help='The file where to store the final coverage')

    args = parser.parse_args()

    if not any((args.fuzzing, args.tests, args.doc_tests)):
        parser.print_help()
        sys.exit(1)

    compute_coverage(
        FUZZING=args.fuzzing,
        TESTS=args.tests,
        DOC_TESTS=args.doc_tests,
        SHOW=args.show,
        final_cov_path=args.final_coverage_file,
    )