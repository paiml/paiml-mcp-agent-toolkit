// FFI exports that should NOT be marked as dead code

#[no_mangle]
pub extern "C" fn exported_function() -> i32 {
    42
}

#[no_mangle]
pub extern "C" fn process_data(ptr: *const u8, len: usize) -> i32 {
    if ptr.is_null() || len == 0 {
        return -1;
    }
    
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        slice.iter().map(|&b| b as i32).sum()
    }
}

#[no_mangle]
pub static EXPORTED_STATIC: i32 = 100;

#[no_mangle]
pub static mut MUTABLE_GLOBAL: i32 = 0;

#[export_name = "custom_name"]
pub fn renamed_export() -> i32 {
    200
}

// WASM bindgen example
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_function(input: &str) -> String {
    format!("Hello, {}!", input)
}

// PyO3 example
#[cfg(feature = "python")]
#[pyfunction]
fn python_export(a: i32, b: i32) -> i32 {
    a + b
}

// Regular function that IS dead code (for comparison)
fn internal_helper() -> i32 {
    123
}