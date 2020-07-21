#[no_mangle]
unsafe extern fn version(buffer: *mut u8) -> i32 {
    let mut buffer = std::slice::from_raw_parts_mut(buffer, 40);
    let version = env!("CARGO_PKG_VERSION");
    std::io::Write::write(&mut buffer, version.as_bytes()).unwrap() as _
}
