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
#[allow(clippy::unnecessary_cast)]  // rationale: "*bits as u64" is flaky redundant
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