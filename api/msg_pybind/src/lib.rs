use msg_internal;

#[repr(C)]
#[derive(Debug)]
pub struct FFIVec {
    len: usize,
    data: *const u8,
    storage: *const Vec<u8>,
}

impl FFIVec {
    unsafe fn raw_to_slice<'a>(ptr: *mut Self) -> &'a [u8] {
        let v = Box::from_raw(ptr);
        let slice = core::slice::from_raw_parts(v.data, v.len);
        std::mem::forget(v);
        slice
    }
    fn from_vec(vec: Vec<u8>) -> Box<Self> {
        let vec = Box::new(vec);
        Box::new(Self {
            len: vec.len(),
            data: vec.as_ptr(),
            storage: Box::into_raw(vec),
        })
    }
}

#[no_mangle]
pub extern "C" fn free_ffi_vec(v: *mut FFIVec) {
    if v != 0 as *mut FFIVec {
        unsafe { drop(Box::from_raw(v)) }
    }
}

macro_rules! catch_or {
    ($x:expr,$def:expr) => {{
        match $x {
            Ok(t) => t,
            Err(msg_internal::Error(msg)) => {
                eprintln!("{}", msg);
                return $def;
            }
        }
    }};
}

#[no_mangle]
pub unsafe extern "C" fn mod_msg_items(items: *mut FFIVec, mods: *mut FFIVec) -> *const FFIVec {
    let items = FFIVec::raw_to_slice(items);
    let mods = FFIVec::raw_to_slice(mods);
    let new_items = catch_or!(msg_internal::mod_items(items, mods), 0 as *const FFIVec);
    // msg_internal::display_items(&new_items);
    let new_items = msg_internal::serialize_items(new_items);
    Box::into_raw(FFIVec::from_vec(new_items))
}

#[no_mangle]
pub unsafe extern "C" fn display_msg_items(items: *mut FFIVec) {
    let items = FFIVec::raw_to_slice(items);
    let items = catch_or!(msg_internal::parse_items(items), ());
    msg_internal::display_items(&items);
}
