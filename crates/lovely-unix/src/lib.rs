use lovely_core::sys::LuaState;

use lovely_core::Lovely;
use once_cell::sync::OnceCell;

static RUNTIME: OnceCell<Lovely> = OnceCell::new();

#[no_mangle]
unsafe extern "C" fn luaL_loadbuffer(state: *mut LuaState, buf_ptr: *const u8, size: isize, name_ptr: *const u8) -> u32 {
    let rt = RUNTIME.get_unchecked();
    rt.apply_buffer_patches(state, buf_ptr, size, name_ptr)
}

#[ctor::ctor]
unsafe fn construct() {
    let ptr = libc::dlsym(libc::RTLD_NEXT, b"luaL_loadbuffer\0".as_ptr() as _);

    if ptr.is_null() {
        panic!("Failed to load luaL_loadbuffer");
    }
    let ptr: unsafe extern "C" fn(*mut libc::c_void, *const u8, isize, *const u8) -> u32 = std::mem::transmute(ptr);
    let rt = Lovely::init(ptr);
    RUNTIME.set(rt).unwrap_or_else(|_| panic!("Failed to instantiate runtime."));
}
