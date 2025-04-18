// Generated by `wit-bindgen` 0.36.0. DO NOT EDIT!
// Options used:
//   * runtime_path: "wit_bindgen_rt"
#[rustfmt::skip]
#[allow(dead_code, clippy::all)]
pub mod exports {
    pub mod golem {
        pub mod it {
            #[allow(dead_code, clippy::all)]
            pub mod api {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[derive(Clone)]
                pub struct Data {
                    pub id: _rt::String,
                    pub name: _rt::String,
                    pub desc: _rt::String,
                    pub timestamp: u64,
                }
                impl ::core::fmt::Debug for Data {
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter<'_>,
                    ) -> ::core::fmt::Result {
                        f.debug_struct("Data")
                            .field("id", &self.id)
                            .field("name", &self.name)
                            .field("desc", &self.desc)
                            .field("timestamp", &self.timestamp)
                            .finish()
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_echo_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let result1 = T::echo(_rt::string_lift(bytes0));
                    let ptr2 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec3 = (result1.into_bytes()).into_boxed_slice();
                    let ptr3 = vec3.as_ptr().cast::<u8>();
                    let len3 = vec3.len();
                    ::core::mem::forget(vec3);
                    *ptr2.add(4).cast::<usize>() = len3;
                    *ptr2.add(0).cast::<*mut u8>() = ptr3.cast_mut();
                    ptr2
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_echo<T: Guest>(arg0: *mut u8) {
                    let l0 = *arg0.add(0).cast::<*mut u8>();
                    let l1 = *arg0.add(4).cast::<usize>();
                    _rt::cabi_dealloc(l0, l1, 1);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_calculate_cabi<T: Guest>(arg0: i64) -> i64 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::calculate(arg0 as u64);
                    _rt::as_i64(result0)
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_process_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let base10 = arg0;
                    let len10 = arg1;
                    let mut result10 = _rt::Vec::with_capacity(len10);
                    for i in 0..len10 {
                        let base = base10.add(i * 32);
                        let e10 = {
                            let l0 = *base.add(0).cast::<*mut u8>();
                            let l1 = *base.add(4).cast::<usize>();
                            let len2 = l1;
                            let bytes2 = _rt::Vec::from_raw_parts(l0.cast(), len2, len2);
                            let l3 = *base.add(8).cast::<*mut u8>();
                            let l4 = *base.add(12).cast::<usize>();
                            let len5 = l4;
                            let bytes5 = _rt::Vec::from_raw_parts(l3.cast(), len5, len5);
                            let l6 = *base.add(16).cast::<*mut u8>();
                            let l7 = *base.add(20).cast::<usize>();
                            let len8 = l7;
                            let bytes8 = _rt::Vec::from_raw_parts(l6.cast(), len8, len8);
                            let l9 = *base.add(24).cast::<i64>();
                            Data {
                                id: _rt::string_lift(bytes2),
                                name: _rt::string_lift(bytes5),
                                desc: _rt::string_lift(bytes8),
                                timestamp: l9 as u64,
                            }
                        };
                        result10.push(e10);
                    }
                    _rt::cabi_dealloc(base10, len10 * 32, 8);
                    let result11 = T::process(result10);
                    let ptr12 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec17 = result11;
                    let len17 = vec17.len();
                    let layout17 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec17.len() * 32,
                        8,
                    );
                    let result17 = if layout17.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout17).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout17);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec17.into_iter().enumerate() {
                        let base = result17.add(i * 32);
                        {
                            let Data {
                                id: id13,
                                name: name13,
                                desc: desc13,
                                timestamp: timestamp13,
                            } = e;
                            let vec14 = (id13.into_bytes()).into_boxed_slice();
                            let ptr14 = vec14.as_ptr().cast::<u8>();
                            let len14 = vec14.len();
                            ::core::mem::forget(vec14);
                            *base.add(4).cast::<usize>() = len14;
                            *base.add(0).cast::<*mut u8>() = ptr14.cast_mut();
                            let vec15 = (name13.into_bytes()).into_boxed_slice();
                            let ptr15 = vec15.as_ptr().cast::<u8>();
                            let len15 = vec15.len();
                            ::core::mem::forget(vec15);
                            *base.add(12).cast::<usize>() = len15;
                            *base.add(8).cast::<*mut u8>() = ptr15.cast_mut();
                            let vec16 = (desc13.into_bytes()).into_boxed_slice();
                            let ptr16 = vec16.as_ptr().cast::<u8>();
                            let len16 = vec16.len();
                            ::core::mem::forget(vec16);
                            *base.add(20).cast::<usize>() = len16;
                            *base.add(16).cast::<*mut u8>() = ptr16.cast_mut();
                            *base.add(24).cast::<i64>() = _rt::as_i64(timestamp13);
                        }
                    }
                    *ptr12.add(4).cast::<usize>() = len17;
                    *ptr12.add(0).cast::<*mut u8>() = result17;
                    ptr12
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_process<T: Guest>(arg0: *mut u8) {
                    let l0 = *arg0.add(0).cast::<*mut u8>();
                    let l1 = *arg0.add(4).cast::<usize>();
                    let base8 = l0;
                    let len8 = l1;
                    for i in 0..len8 {
                        let base = base8.add(i * 32);
                        {
                            let l2 = *base.add(0).cast::<*mut u8>();
                            let l3 = *base.add(4).cast::<usize>();
                            _rt::cabi_dealloc(l2, l3, 1);
                            let l4 = *base.add(8).cast::<*mut u8>();
                            let l5 = *base.add(12).cast::<usize>();
                            _rt::cabi_dealloc(l4, l5, 1);
                            let l6 = *base.add(16).cast::<*mut u8>();
                            let l7 = *base.add(20).cast::<usize>();
                            _rt::cabi_dealloc(l6, l7, 1);
                        }
                    }
                    _rt::cabi_dealloc(base8, len8 * 32, 8);
                }
                pub trait Guest {
                    fn echo(input: _rt::String) -> _rt::String;
                    fn calculate(input: u64) -> u64;
                    fn process(input: _rt::Vec<Data>) -> _rt::Vec<Data>;
                }
                #[doc(hidden)]
                macro_rules! __export_golem_it_api_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name = "golem:it/api#echo"] unsafe
                        extern "C" fn export_echo(arg0 : * mut u8, arg1 : usize,) -> *
                        mut u8 { $($path_to_types)*:: _export_echo_cabi::<$ty > (arg0,
                        arg1) } #[export_name = "cabi_post_golem:it/api#echo"] unsafe
                        extern "C" fn _post_return_echo(arg0 : * mut u8,) {
                        $($path_to_types)*:: __post_return_echo::<$ty > (arg0) }
                        #[export_name = "golem:it/api#calculate"] unsafe extern "C" fn
                        export_calculate(arg0 : i64,) -> i64 { $($path_to_types)*::
                        _export_calculate_cabi::<$ty > (arg0) } #[export_name =
                        "golem:it/api#process"] unsafe extern "C" fn export_process(arg0
                        : * mut u8, arg1 : usize,) -> * mut u8 { $($path_to_types)*::
                        _export_process_cabi::<$ty > (arg0, arg1) } #[export_name =
                        "cabi_post_golem:it/api#process"] unsafe extern "C" fn
                        _post_return_process(arg0 : * mut u8,) { $($path_to_types)*::
                        __post_return_process::<$ty > (arg0) } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_golem_it_api_cabi;
                #[repr(align(4))]
                struct _RetArea([::core::mem::MaybeUninit<u8>; 8]);
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 8],
                );
            }
        }
    }
}
#[rustfmt::skip]
mod _rt {
    pub use alloc_crate::string::String;
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen_rt::run_ctors_once();
    }
    pub use alloc_crate::vec::Vec;
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
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
/// Generates `#[no_mangle]` functions to export the specified type as the
/// root implementation of all generated traits.
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
macro_rules! __export_rust_component_service_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*::
        exports::golem::it::api::__export_golem_it_api_cabi!($ty with_types_in
        $($path_to_types_root)*:: exports::golem::it::api);
    };
}
#[doc(inline)]
pub(crate) use __export_rust_component_service_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.36.0:golem:it:rust-component-service:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 317] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xb0\x01\x01A\x02\x01\
A\x02\x01B\x09\x01r\x04\x02ids\x04names\x04descs\x09timestampw\x04\0\x04data\x03\
\0\0\x01@\x01\x05inputs\0s\x04\0\x04echo\x01\x02\x01@\x01\x05inputw\0w\x04\0\x09\
calculate\x01\x03\x01p\x01\x01@\x01\x05input\x04\0\x04\x04\0\x07process\x01\x05\x04\
\0\x0cgolem:it/api\x05\0\x04\0\x1fgolem:it/rust-component-service\x04\0\x0b\x1c\x01\
\0\x16rust-component-service\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0d\
wit-component\x070.220.0\x10wit-bindgen-rust\x060.36.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
