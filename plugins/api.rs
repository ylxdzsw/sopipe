// Rust helper for implementing the API.

use core::ffi::c_void;

// Note: fn pointers can only hold non-0 addresses, so the Option will still be one word long.
#[allow(non_upper_case_globals, clippy::type_complexity)]
static mut _sopipe_get: Option<extern fn(*mut c_void, i32, *const u8, *mut i32, *mut u8)> = None;

#[allow(non_upper_case_globals, clippy::type_complexity)]
static mut _sopipe_set: Option<extern fn(*mut c_void, i32, *const u8, i32, *const u8)> = None;

#[allow(non_upper_case_globals)]
static mut _sopipe_write: Option<extern fn(*mut c_void, i32, *const u8)> = None;

#[allow(dead_code)]
fn sopipe_get(stream: *mut c_void, key: &[u8], value: &mut [u8]) -> Option<usize> {
    let f = unsafe { _sopipe_get.unwrap_or_else(|| core::hint::unreachable_unchecked()) };
    let mut len = value.len() as i32;
    f(stream, key.len() as _, key.as_ptr(), &mut len, value.as_mut_ptr());
    if len == -1 {
        None
    } else {
        Some(len as _)
    }
}

#[allow(dead_code)]
fn sopipe_get_ptr<T>(stream: *mut c_void, key: &[u8]) -> Option<*mut T> {
    let mut buffer = [0; core::mem::size_of::<*const T>()];
    sopipe_get(stream, key, &mut buffer).map(|x| {
        assert_eq!(x, buffer.len());
        unsafe { core::mem::transmute(buffer) }
    })
}

#[allow(dead_code)]
fn sopipe_get_alloc(stream: *mut c_void, key: &[u8], size_hint: Option<usize>) -> Option<Box<[u8]>> {
    let f = unsafe { _sopipe_get.unwrap_or_else(|| core::hint::unreachable_unchecked()) };
    let mut len = -1;
    if let Some(size_hint) = size_hint {
        len = size_hint as _;
        let mut buf = vec![0; size_hint as usize]; // TODO: uninitialized?
        f(stream, key.len() as _, key.as_ptr(), &mut len, buf.as_mut_ptr());
        if len == -1 {
            return None
        }
        if len <= size_hint as _ {
            buf.truncate(len as _); // Note: truncate run drop on traling elements, but it should be fine for u8.
            return Some(buf.into_boxed_slice())
        }
    } else {
        f(stream, key.len() as _, key.as_ptr(), &mut len, core::ptr::null_mut());
        if len == -1 {
            return None
        }
    }

    let mut buf = vec![0; len as usize]; // TODO: uninitialized?
    f(stream, key.len() as _, key.as_ptr(), &mut len, buf.as_mut_ptr());
    Some(buf.into_boxed_slice())
}

#[allow(dead_code)]
fn sopipe_set(stream: *mut c_void, key: &[u8], value: &[u8]) {
    let f = unsafe { _sopipe_set.unwrap_or_else(|| core::hint::unreachable_unchecked()) };
    f(stream, key.len() as _, key.as_ptr(), value.len() as _, value.as_ptr());
}

#[allow(dead_code)]
fn sopipe_set_ptr<T>(stream: *mut c_void, key: &[u8], value: &T) {
    let buffer = unsafe { &core::mem::transmute::<_, [u8; core::mem::size_of::<*const ()>()]>(value) };
    sopipe_set(stream, key, buffer)
}

#[allow(dead_code)]
fn sopipe_del(stream: *mut c_void, key: &[u8]) {
    let f = unsafe { _sopipe_set.unwrap_or_else(|| core::hint::unreachable_unchecked()) };
    f(stream, key.len() as _, key.as_ptr(), -1, core::ptr::null());
}

#[allow(dead_code)]
fn sopipe_write(stream: *mut c_void, data: &[u8]) {
    let f = unsafe { _sopipe_write.unwrap_or_else(|| core::hint::unreachable_unchecked()) };
    f(stream, data.len() as _, data.as_ptr());
}

#[no_mangle]
unsafe extern fn api_version() -> i32 {
    1
}

#[no_mangle]
unsafe extern fn version(len: *mut i32, buffer: *mut u8) {
    let mut buffer = core::slice::from_raw_parts_mut(buffer, 40);
    let version = env!("CARGO_PKG_VERSION");
    *len = std::io::Write::write(&mut buffer, version.as_bytes()).unwrap() as _;
}

#[no_mangle]
unsafe extern fn init(get: *mut c_void, set: *mut c_void, write: *mut c_void) {
    _sopipe_get = Some(core::mem::transmute(get));
    _sopipe_set = Some(core::mem::transmute(set));
    _sopipe_write = Some(core::mem::transmute(write));
}

#[no_mangle]
unsafe extern fn forward(stream: *mut c_void, len: i32, buffer: *const u8) {
    let key = concat!(env!("CARGO_PKG_NAME"), "_forward_ctx").as_bytes();

    if len == -1 {
        if let Some(x) = sopipe_get_ptr::<ForwardContext>(stream, key) {
            Box::from_raw(x);
            sopipe_del(stream, key)
        }
        return
    }

    let ctx = {
        sopipe_get_ptr(stream, key).map(|x| &mut *x).unwrap_or_else(|| {
            let ctx = Box::leak(ForwardContext::new());
            sopipe_set_ptr(stream, key, ctx);
            ctx
        })
    };

    let data = core::slice::from_raw_parts(buffer, len as _);

    ctx.write(stream, data);
}

#[no_mangle]
unsafe extern fn backward(stream: *mut c_void, len: i32, buffer: *const u8) {
    let key = concat!(env!("CARGO_PKG_NAME"), "_backward_ctx").as_bytes();

    if len == -1 {
        if let Some(x) = sopipe_get_ptr::<BackwardContext>(stream, key) {
            Box::from_raw(x);
            sopipe_del(stream, key)
        }
        return
    }

    let ctx = {
        sopipe_get_ptr(stream, key).map(|x| &mut *x).unwrap_or_else(|| {
            let ctx = Box::leak(BackwardContext::new());
            sopipe_set_ptr(stream, key, ctx);
            ctx
        })
    };

    let data = core::slice::from_raw_parts(buffer, len as _);

    ctx.write(stream, data);
}
