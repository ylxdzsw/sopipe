use oh_my_rust::*;
use core::ffi::c_void;
use libloading::Symbol;
use crate::{ByteDict, Actor, Component};

pub(crate) struct Plugin {
    init: Symbol<'static, unsafe extern fn(*const c_void, *const c_void)>,
    create: Symbol<'static, unsafe extern fn(*const c_void) -> *mut c_void>,
    version: Symbol<'static, unsafe extern fn(*mut i32, *mut [u8])>
}

impl Plugin {
    pub(crate) unsafe fn load(libname: &str) -> &'static Plugin {
        let name = if cfg!(windows) {
            format!("lib{}.dll", libname)
        } else if cfg!(unix) {
            format!("lib{}.so", libname)
        } else {
            unimplemented!()
        };

        let library = leak(libloading::Library::new(name).unwrap());

        let plugin = Plugin {
            init: library.get(b"init\0").unwrap(),
            create: library.get(b"create\0").unwrap(),
            version: library.get(b"version\0").unwrap(),
        };

        (plugin.init)(
            sopipe_get as *const _,
            sopipe_set as *const _
        );

        leak(plugin)
    }

    pub(crate) unsafe fn version(&self) -> String {
        let mut len = 0;
        let mut buf = [0; 40];
        (self.version)(&mut len, &mut buf);
        std::str::from_utf8(&buf[0..len as _]).unwrap().to_owned()
    }
}

#[async_trait::async_trait]
impl Component for Plugin {
    async fn init(&'static self, args: &ByteDict) -> *mut c_void {
        unsafe { (self.create)(args as *const _ as _) }
    }

    async fn start(&'static self, actor: &mut Actor) {
        todo!()
    }
}

unsafe extern fn sopipe_get(dict: *mut ByteDict, key_len: i32, key_ptr: *const u8, value_len: *mut i32, value_ptr: *mut u8) {
    let dict = &mut *dict;
    let key = core::slice::from_raw_parts(key_ptr, key_len as _);
    if let Some(value) = dict.get(key) {
        if *value_len >= value.len() as _ {
            core::slice::from_raw_parts_mut(value_ptr, *value_len as _).copy_from_slice(value)
        }
        *value_len = value.len() as _;
    } else {
        *value_len = -1
    }
}

unsafe extern fn sopipe_set(dict: *mut ByteDict, key_len: i32, key_ptr: *const u8, value_len: i32, value_ptr: *mut u8) {
    let dict = &mut *dict;
    let key = core::slice::from_raw_parts(key_ptr, key_len as _);
    if value_len < 0 {
        dict.remove(key);
        return
    }

    let value = core::slice::from_raw_parts(value_ptr, value_len as _);
    dict.insert(key.into(), value.into());
}
