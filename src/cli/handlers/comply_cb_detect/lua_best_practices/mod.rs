#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-600 Series: Lua Best Practices Detection
//!
//! Generic Lua defect detection for `pmat comply check`.
//! Based on: LuaTaint (Xiang et al. 2025), FLuaScan (Gao et al. 2023),
//! Luau type system (Brown et al. 2021/2023), luacheck W111/W113/W211.

mod cb600_cb601;
mod cb602_cb603;
mod cb604_cb605;
mod cb606_cb607;
mod cb608_cb609_cb610;
mod cb611_cb612_cb613;
mod cb614_cb615;
mod cb616_cb617;
mod cb618_cb619;
mod constants;
mod detection_helpers;
mod helpers;

// Re-export helpers (pub)
pub use helpers::{
    compute_lua_production_lines, count_consecutive_field_access, is_lua_test_file,
    walkdir_lua_files,
};

// Re-export detection helpers
pub use detection_helpers::extract_pcall_status_var;

// Re-export CB-600/601
pub use cb600_cb601::{detect_cb600_implicit_globals, detect_cb601_nil_unsafe_access};

// Re-export CB-602/603
pub use cb602_cb603::{detect_cb602_pcall_error_handling, detect_cb603_deprecated_dangerous_api};

// Re-export CB-604/605
pub use cb604_cb605::{detect_cb604_unused_variables, detect_cb605_string_concat_in_loop};

// Re-export CB-606/607
pub use cb606_cb607::{detect_cb606_missing_module_return, detect_cb607_colon_dot_confusion};

// Re-export CB-608/609/610
pub use cb608_cb609_cb610::{
    detect_cb608_unchecked_nil_err, detect_cb609_assert_in_library,
    detect_cb610_string_accumulator_in_loop,
};

// Re-export CB-611/612/613
pub use cb611_cb612_cb613::{
    detect_cb611_weak_table_misuse, detect_cb612_test_framework, detect_cb613_require_cycles,
    detect_lua_test_frameworks, LuaTestFramework,
};

// Re-export CB-614/615
pub use cb614_cb615::{detect_cb614_global_protection, detect_cb615_coroutine_checks};

// Re-export CB-616/617
pub use cb616_cb617::{
    detect_cb616_type_annotations, detect_cb617_openresty_checks, LuaAnnotationSystem,
};

// Re-export CB-618/619
pub use cb618_cb619::{detect_cb618_ffi_safety, detect_cb619_oop_patterns, LuaOopPattern};
