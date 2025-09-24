use core::slice;
use std::ffi::{CString, c_char};
use std::path::Path;

use oxc_allocator::Allocator;

mod transpile;

#[repr(C)]
pub enum TranspileResult {
    /// Success
    Ok = 0,
    /// Provided arguments are not valid
    Invalid,
    /// An error occurred with I/O
    Io,
    /// Unable to parse the source
    Parse,
    /// Semantic issue with the source
    Semantic,
    /// Unable to transpile the source
    Transformer,
}

/// # Safety
/// Don't free the returned pointer with gnu malloc/free, instead use `oxcc_free`.
/// Also the returned pointer can be NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxcc__transpile(
    path_ptr: *const c_char,
    path_len: usize,
    output_ptr: *mut *const c_char,
    output_len: *mut usize,
) -> TranspileResult {
    if path_ptr.is_null() || output_ptr.is_null() {
        return TranspileResult::Invalid;
    }

    let path_bytes: &[u8] = unsafe { slice::from_raw_parts(path_ptr as _, path_len) };
    let path_str = match std::str::from_utf8(path_bytes) {
        Ok(s) => s,
        Err(err) => {
            println!("{err}");
            return TranspileResult::Io;
        }
    };
    let path = Path::new(path_str);
    // TODO this should be given by the C side and re-used per-worker
    let mut allocator = Allocator::new();
    match transpile::transpile(path, &mut allocator) {
        Ok(codegen) => {
            let code_len = codegen.code.len();
            match CString::new(codegen.code) {
                Ok(code) => unsafe {
                    output_ptr.write(code.into_raw());
                    output_len.write(code_len);
                    TranspileResult::Ok
                },
                Err(err) => {
                    eprintln!("{err}");
                    TranspileResult::Io
                }
            }
        }
        Err(transpile::Error::Io(err)) => {
            eprintln!("{err}");
            TranspileResult::Io
        }
        Err(transpile::Error::Parse(errors)) => {
            for error in errors {
                eprintln!("{error:?}");
            }
            TranspileResult::Parse
        }
        Err(transpile::Error::Semantic(errors)) => {
            for error in errors {
                eprintln!("{error:?}");
            }
            TranspileResult::Semantic
        }
        Err(transpile::Error::Transformer(errors)) => {
            for error in errors {
                eprintln!("{error:?}");
            }
            TranspileResult::Transformer
        }
    }
}

/// # Safety
/// Make sure you pass in a pointer to a string allocated by `demo_transpile`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxcc__free(input: *const c_char) {
    if input.is_null() {
        return;
    }
    let _ = unsafe { CString::from_raw(input as _) };
}
