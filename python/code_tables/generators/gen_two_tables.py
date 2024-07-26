
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

funcs_two_table = """
#[inline(always)]
/// Read a value using a decoding table.
///
/// If the result is `Some` the decoding was successful, and
/// the decoded value and the length of the code are returned.
pub fn read_table_%(bo)s<B: BitRead<%(BO)s>>(backend: &mut B) -> Option<(u64, usize)> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.cast();
        let len = READ_LEN_%(BO)s[idx as usize];
        if len != MISSING_VALUE_LEN_%(BO)s {
            backend.skip_bits_after_table_lookup(len as usize);
            return Some((READ_%(BO)s[idx as usize] as u64, len as usize));
        }
    }
    None
}

#[inline(always)]
/// Skip a value using a decoding table.
///
/// If the result is `Some` the lookup was successful, and
/// the length of the code is returned.
pub fn skip_table_%(bo)s<B: BitRead<%(BO)s>>(backend: &mut B) -> Option<usize> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.cast();
        let len = READ_LEN_%(BO)s[idx as usize];
        if len != MISSING_VALUE_LEN_%(BO)s {
            backend.skip_bits_after_table_lookup(len as usize);
            return Some(len as usize);
        }
    }
    None
}

#[inline(always)]
/// Write a value using an encoding table.
///
/// If the result is `Some` the encoding was successful, and
/// length of the code is returned.
pub fn write_table_%(bo)s<B: BitWrite<%(BO)s>>(backend: &mut B, value: u64) -> Result<Option<usize>, B::Error> {
    Ok(if let Some(bits) = WRITE_%(BO)s.get(value as usize) {
        let len = WRITE_LEN[value as usize] as usize;
        backend.write_bits(*bits as u64, len)?;
        Some(len)
    } else {
        None
    })
}

#[inline(always)]
#[allow(clippy::unnecessary_cast)]  // rationale: "*bits as u64" is flaky redundant
/// Get the length of a value using an encoding table.
///
/// If the result is `Some` the len was in the table.
pub fn len_table_%(bo)s(value: u64) -> Option<usize> {
    WRITE_LEN.get(value as usize).map(|x| *x as usize)
}
"""