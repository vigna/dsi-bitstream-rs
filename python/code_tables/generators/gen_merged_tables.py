
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

funcs_merged_table = """
#[inline(always)]
/// Read a value using a decoding table.
///
/// If the result is `Some` the decoding was successful, and
/// the decoded value and the length of the code are returned.
pub fn read_table_%(bo)s<B: BitRead<%(BO)s>>(backend: &mut B) -> Option<(u64, usize)> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.cast();
        let (value, len) = READ_%(BO)s[idx as usize];
        if len != MISSING_VALUE_LEN_%(BO)s {
            backend.skip_bits_after_table_lookup(len as usize);
            return Some((value as u64, len as usize));
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
        let (_value, len) = READ_%(BO)s[idx as usize];
        if len != MISSING_VALUE_LEN_%(BO)s {
            backend.skip_bits_after_table_lookup(len as usize);
            return Some(len as usize);
        }
    }
    None
}

#[inline(always)]
#[allow(clippy::unnecessary_cast)]  // rationale: "*bits as u64" is flaky redundant
/// Write a value using an encoding table.
///
/// If the result is `Some` the encoding was successful, and
/// length of the code is returned.
pub fn write_table_%(bo)s<B: BitWrite<%(BO)s>>(backend: &mut B, value: u64) -> Result<Option<usize>, B::Error> {
    Ok(if let Some((bits, len)) = WRITE_%(BO)s.get(value as usize) {
        backend.write_bits(*bits as u64, *len as usize)?;
        Some(*len as usize)        
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
    WRITE_%(BO)s.get(value as usize).map(|x| x.1 as usize)
}
"""