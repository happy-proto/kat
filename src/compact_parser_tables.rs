//! Decompression entry point used by generated Tree-sitter language getters.

use std::slice;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kat_decompress_parser_table(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
) -> bool {
    if input.is_null() || output.is_null() || input_len == 0 || output_len == 0 {
        return false;
    }
    // The pointers and lengths are emitted together from the build-time packer.
    let input = unsafe { slice::from_raw_parts(input, input_len) };
    let output = unsafe { slice::from_raw_parts_mut(output, output_len) };
    matches!(zstd::bulk::decompress_to_buffer(input, output), Ok(size) if size == output_len)
}
