// Generated by `wit-bindgen` 0.41.0. DO NOT EDIT!
// Options used:
//   * runtime_path: "wit_bindgen_rt"
#[rustfmt::skip]
#[allow(dead_code, clippy::all)]
pub mod exports {
    pub mod golem {
        pub mod api {
            /// Interface providing user-defined snapshotting capability. This can be used to perform manual update of workers
            /// when the new component incompatible with the old one.
            #[allow(dead_code, async_fn_in_trait, unused_imports, clippy::all)]
            pub mod save_snapshot {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_save_cabi<T: Guest>() -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::save();
                    let ptr1 = (&raw mut _RET_AREA.0).cast::<u8>();
                    let vec2 = (result0).into_boxed_slice();
                    let ptr2 = vec2.as_ptr().cast::<u8>();
                    let len2 = vec2.len();
                    ::core::mem::forget(vec2);
                    *ptr1.add(::core::mem::size_of::<*const u8>()).cast::<usize>() = len2;
                    *ptr1.add(0).cast::<*mut u8>() = ptr2.cast_mut();
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_save<T: Guest>(arg0: *mut u8) {
                    let l0 = *arg0.add(0).cast::<*mut u8>();
                    let l1 = *arg0
                        .add(::core::mem::size_of::<*const u8>())
                        .cast::<usize>();
                    let base2 = l0;
                    let len2 = l1;
                    _rt::cabi_dealloc(base2, len2 * 1, 1);
                }
                pub trait Guest {
                    /// Saves the component's state into a user-defined snapshot
                    fn save() -> _rt::Vec<u8>;
                }
                #[doc(hidden)]
                macro_rules! __export_golem_api_save_snapshot_1_1_7_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[unsafe (export_name =
                        "golem:api/save-snapshot@1.1.7#save")] unsafe extern "C" fn
                        export_save() -> * mut u8 { unsafe { $($path_to_types)*::
                        _export_save_cabi::<$ty > () } } #[unsafe (export_name =
                        "cabi_post_golem:api/save-snapshot@1.1.7#save")] unsafe extern
                        "C" fn _post_return_save(arg0 : * mut u8,) { unsafe {
                        $($path_to_types)*:: __post_return_save::<$ty > (arg0) } } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_golem_api_save_snapshot_1_1_7_cabi;
                #[cfg_attr(target_pointer_width = "64", repr(align(8)))]
                #[cfg_attr(target_pointer_width = "32", repr(align(4)))]
                struct _RetArea(
                    [::core::mem::MaybeUninit<
                        u8,
                    >; 2 * ::core::mem::size_of::<*const u8>()],
                );
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 2
                        * ::core::mem::size_of::<*const u8>()],
                );
            }
            /// Interface providing user-defined snapshotting capability. This can be used to perform manual update of workers
            /// when the new component incompatible with the old one.
            #[allow(dead_code, async_fn_in_trait, unused_imports, clippy::all)]
            pub mod load_snapshot {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_load_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let result1 = T::load(
                        _rt::Vec::from_raw_parts(arg0.cast(), len0, len0),
                    );
                    let ptr2 = (&raw mut _RET_AREA.0).cast::<u8>();
                    match result1 {
                        Ok(_) => {
                            *ptr2.add(0).cast::<u8>() = (0i32) as u8;
                        }
                        Err(e) => {
                            *ptr2.add(0).cast::<u8>() = (1i32) as u8;
                            let vec3 = (e.into_bytes()).into_boxed_slice();
                            let ptr3 = vec3.as_ptr().cast::<u8>();
                            let len3 = vec3.len();
                            ::core::mem::forget(vec3);
                            *ptr2
                                .add(2 * ::core::mem::size_of::<*const u8>())
                                .cast::<usize>() = len3;
                            *ptr2
                                .add(::core::mem::size_of::<*const u8>())
                                .cast::<*mut u8>() = ptr3.cast_mut();
                        }
                    };
                    ptr2
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_load<T: Guest>(arg0: *mut u8) {
                    let l0 = i32::from(*arg0.add(0).cast::<u8>());
                    match l0 {
                        0 => {}
                        _ => {
                            let l1 = *arg0
                                .add(::core::mem::size_of::<*const u8>())
                                .cast::<*mut u8>();
                            let l2 = *arg0
                                .add(2 * ::core::mem::size_of::<*const u8>())
                                .cast::<usize>();
                            _rt::cabi_dealloc(l1, l2, 1);
                        }
                    }
                }
                pub trait Guest {
                    /// Tries to load a user-defined snapshot, setting up the worker's state based on it.
                    /// The function can return with a failure to indicate that the update is not possible.
                    fn load(bytes: _rt::Vec<u8>) -> Result<(), _rt::String>;
                }
                #[doc(hidden)]
                macro_rules! __export_golem_api_load_snapshot_1_1_7_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[unsafe (export_name =
                        "golem:api/load-snapshot@1.1.7#load")] unsafe extern "C" fn
                        export_load(arg0 : * mut u8, arg1 : usize,) -> * mut u8 { unsafe
                        { $($path_to_types)*:: _export_load_cabi::<$ty > (arg0, arg1) } }
                        #[unsafe (export_name =
                        "cabi_post_golem:api/load-snapshot@1.1.7#load")] unsafe extern
                        "C" fn _post_return_load(arg0 : * mut u8,) { unsafe {
                        $($path_to_types)*:: __post_return_load::<$ty > (arg0) } } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_golem_api_load_snapshot_1_1_7_cabi;
                #[cfg_attr(target_pointer_width = "64", repr(align(8)))]
                #[cfg_attr(target_pointer_width = "32", repr(align(4)))]
                struct _RetArea(
                    [::core::mem::MaybeUninit<
                        u8,
                    >; 3 * ::core::mem::size_of::<*const u8>()],
                );
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 3
                        * ::core::mem::size_of::<*const u8>()],
                );
            }
        }
        pub mod component {
            #[allow(dead_code, async_fn_in_trait, unused_imports, clippy::all)]
            pub mod api {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_get_cabi<T: Guest>() -> i64 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::get();
                    _rt::as_i64(result0)
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_set_cabi<T: Guest>(arg0: i64) -> i64 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::set(arg0 as u64);
                    _rt::as_i64(result0)
                }
                pub trait Guest {
                    fn get() -> u64;
                    fn set(value: u64) -> u64;
                }
                #[doc(hidden)]
                macro_rules! __export_golem_component_api_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[unsafe (export_name =
                        "golem:component/api#get")] unsafe extern "C" fn export_get() ->
                        i64 { unsafe { $($path_to_types)*:: _export_get_cabi::<$ty > () }
                        } #[unsafe (export_name = "golem:component/api#set")] unsafe
                        extern "C" fn export_set(arg0 : i64,) -> i64 { unsafe {
                        $($path_to_types)*:: _export_set_cabi::<$ty > (arg0) } } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_golem_component_api_cabi;
            }
        }
    }
}
#[rustfmt::skip]
mod _rt {
    #![allow(dead_code, clippy::all)]
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen_rt::run_ctors_once();
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    pub use alloc_crate::vec::Vec;
    pub use alloc_crate::string::String;
    pub fn as_i64<T: AsI64>(t: T) -> i64 {
        t.as_i64()
    }
    pub trait AsI64 {
        fn as_i64(self) -> i64;
    }
    impl<'a, T: Copy + AsI64> AsI64 for &'a T {
        fn as_i64(self) -> i64 {
            (*self).as_i64()
        }
    }
    impl AsI64 for i64 {
        #[inline]
        fn as_i64(self) -> i64 {
            self as i64
        }
    }
    impl AsI64 for u64 {
        #[inline]
        fn as_i64(self) -> i64 {
            self as i64
        }
    }
    pub use alloc_crate::alloc;
    extern crate alloc as alloc_crate;
}
/// Generates `#[unsafe(no_mangle)]` functions to export the specified type as
/// the root implementation of all generated traits.
///
/// For more information see the documentation of `wit_bindgen::generate!`.
///
/// ```rust
/// # macro_rules! export{ ($($t:tt)*) => (); }
/// # trait Guest {}
/// struct MyType;
///
/// impl Guest for MyType {
///     // ...
/// }
///
/// export!(MyType);
/// ```
#[allow(unused_macros)]
#[doc(hidden)]
macro_rules! __export_update_test_v3_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*::
        exports::golem::api::save_snapshot::__export_golem_api_save_snapshot_1_1_7_cabi!($ty
        with_types_in $($path_to_types_root)*:: exports::golem::api::save_snapshot);
        $($path_to_types_root)*::
        exports::golem::api::load_snapshot::__export_golem_api_load_snapshot_1_1_7_cabi!($ty
        with_types_in $($path_to_types_root)*:: exports::golem::api::load_snapshot);
        $($path_to_types_root)*::
        exports::golem::component::api::__export_golem_component_api_cabi!($ty
        with_types_in $($path_to_types_root)*:: exports::golem::component::api);
    };
}
#[doc(inline)]
pub(crate) use __export_update_test_v3_impl as export;
#[cfg(target_arch = "wasm32")]
#[unsafe(
    link_section = "component-type:wit-bindgen:0.41.0:golem:component:update-test-v3:encoded world"
)]
#[doc(hidden)]
#[allow(clippy::octal_escapes)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 354] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xdd\x01\x01A\x02\x01\
A\x06\x01B\x03\x01p}\x01@\0\0\0\x04\0\x04save\x01\x01\x04\0\x1dgolem:api/save-sn\
apshot@1.1.7\x05\0\x01B\x04\x01p}\x01j\0\x01s\x01@\x01\x05bytes\0\0\x01\x04\0\x04\
load\x01\x02\x04\0\x1dgolem:api/load-snapshot@1.1.7\x05\x01\x01B\x04\x01@\0\0w\x04\
\0\x03get\x01\0\x01@\x01\x05valuew\0w\x04\0\x03set\x01\x01\x04\0\x13golem:compon\
ent/api\x05\x02\x04\0\x1egolem:component/update-test-v3\x04\0\x0b\x14\x01\0\x0eu\
pdate-test-v3\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-component\x07\
0.227.1\x10wit-bindgen-rust\x060.41.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
