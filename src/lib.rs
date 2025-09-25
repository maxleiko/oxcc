use core::slice;
use std::ffi::{CString, c_char};
use std::path::Path;

mod transpile;

#[repr(C)]
pub enum Result {
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
/// The returned pointer must be de-allocated with `oxcc_transpiler__free`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxcc_transpiler__new() -> *mut transpile::Transpiler {
    Box::into_raw(Box::new(transpile::Transpiler::default()))
}

/// # Safety
/// The given pointer must be allocated with `oxcc_transpiler__new`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxcc_transpiler__free(transpiler: *mut transpile::Transpiler) {
    if transpiler.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(transpiler) });
}

/// # Safety
/// `transpiler` must be allocated using `oxcc_transpiler__new`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxcc_transpiler__transpile(
    transpiler: *mut transpile::Transpiler,
    path_ptr: *const c_char,
    path_len: usize,
    output_ptr: *mut *const c_char,
    output_len: *mut usize,
) -> Result {
    if path_ptr.is_null() || output_ptr.is_null() {
        return Result::Invalid;
    }

    let path_bytes: &[u8] = unsafe { slice::from_raw_parts(path_ptr as _, path_len) };
    let path_str = match std::str::from_utf8(path_bytes) {
        Ok(s) => s,
        Err(err) => {
            println!("{err}");
            return Result::Io;
        }
    };
    let path = Path::new(path_str);
    let transpiler = unsafe { &mut *transpiler };
    match transpiler.transpile(path) {
        Ok(codegen) => {
            let code_len = codegen.code.len();
            match CString::new(codegen.code) {
                Ok(code) => unsafe {
                    output_ptr.write(code.into_raw());
                    output_len.write(code_len);
                    Result::Ok
                },
                Err(err) => {
                    eprintln!("{err}");
                    Result::Io
                }
            }
        }
        Err(transpile::Error::Io(err)) => {
            eprintln!("{err}");
            Result::Io
        }
        Err(transpile::Error::Parse(errors)) => {
            for error in errors {
                eprintln!("{error:?}");
            }
            Result::Parse
        }
        Err(transpile::Error::Semantic(errors)) => {
            for error in errors {
                eprintln!("{error:?}");
            }
            Result::Semantic
        }
        Err(transpile::Error::Transformer(errors)) => {
            for error in errors {
                eprintln!("{error:?}");
            }
            Result::Transformer
        }
    }
}

/// # Safety
/// Make sure you pass in a pointer to a string allocated by `demo_transpile`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxcc_string__free(input: *const c_char) {
    if input.is_null() {
        return;
    }
    let _ = unsafe { CString::from_raw(input as _) };
}
