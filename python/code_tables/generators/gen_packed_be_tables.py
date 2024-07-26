
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


funcs_packed_be = """
#[inline(always)]
/// Read a value using a decoding table.
///
/// If the result is `Some` the decoding was successful, and
/// the decoded value and the length of the code are returned.
pub fn read_table_%(bo)s<B: BitRead<%(BO)s>>(backend: &mut B) -> Option<(u64, usize)> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.cast();
        let base = (idx as usize).checked_mul(%(read_bytes)s + %(read_len_bytes)s)?;
        let len = %(read_len_ty)s::from_be_bytes(READ_%(BO)s[base + %(read_bytes)s..base + %(read_bytes)s + %(read_len_bytes)s].try_into().unwrap());
        if len != MISSING_VALUE_LEN_%(BO)s {
            let value = %(read_ty)s::from_be_bytes(READ_%(BO)s[base..base + %(read_bytes)s].try_into().unwrap());
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
        let base = (idx as usize).checked_mul(%(read_bytes)s + %(read_len_bytes)s)?;
        let len = %(read_len_ty)s::from_be_bytes(READ_%(BO)s[base + %(read_bytes)s..base + %(read_bytes)s + %(read_len_bytes)s].try_into().unwrap());
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
pub fn write_table_%(bo)s<B: BitWrite<%(BO)s>>(backend: &mut B, value: u64) 
    -> Result<Option<usize>, B::Error> {
    if value >= %(write_ty)s::MAX as u64 / (%(write_bytes)s + %(write_len_bytes)s) {
        return Ok(None);
    }
    let base = (value as usize) * (%(write_bytes)s + %(write_len_bytes)s);
    if base >= WRITE_%(BO)s.len() {
        return Ok(None);   
    }
    let bits = %(write_ty)s::from_be_bytes(WRITE_%(BO)s[base..base + %(write_bytes)s].try_into().unwrap());
    let len = %(write_len_ty)s::from_be_bytes(WRITE_%(BO)s[base + %(write_bytes)s..base + %(write_bytes)s + %(write_len_bytes)s].try_into().unwrap());
    backend.write_bits(bits as u64, len as usize)?;
    Ok(Some(len as usize))        
}

#[inline(always)]
#[allow(clippy::unnecessary_cast)]  // rationale: "*bits as u64" is flaky redundant
/// Get the length of a value using an encoding table.
///
/// If the result is `Some` the len was in the table.
pub fn len_table_%(bo)s(value: u64) -> Option<usize> {
    let base = (value as usize).checked_mul(%(write_bytes)s + %(write_len_bytes)s)?;
    if base >= WRITE_%(BO)s.len() {
        return None;   
    }
    let len = %(write_len_ty)s::from_be_bytes(WRITE_%(BO)s[base + %(write_bytes)s..base + %(write_bytes)s + %(write_len_bytes)s].try_into().unwrap());
    Some(len as usize)        
}
"""