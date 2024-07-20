use std::{ffi::{c_char, c_void, CString}, slice};

use log::info;

use crate::LoadBuffer;

pub const LUA_GLOBALSINDEX: isize = -10002;

pub const LUA_TNIL: isize = 0;
pub const LUA_TBOOLEAN: isize = 1;

pub type LuaState = c_void;

extern "C" {
    pub fn lua_call(_: *mut LuaState, _: isize, _: isize);
    pub fn lua_pcall(_: *mut LuaState, _: isize, _: isize, _: isize) -> isize;
    pub fn lua_getfield(_: *mut LuaState, _: isize, _: *const c_char);
    pub fn lua_setfield(_: *mut LuaState, _: isize, _: *const c_char);
    pub fn lua_gettop(_: *mut LuaState) -> isize;
    pub fn lua_settop(_: *mut LuaState, _: isize) -> isize;
    pub fn lua_pushvalue(_: *mut LuaState, _: isize);
    pub fn lua_pushcclosure(_: *mut LuaState, _: *const c_void, _: isize);
    pub fn lua_tolstring(_: *mut LuaState, _: isize, _: *mut isize) -> *const c_char;
    pub fn lua_toboolean(_: *mut LuaState, _: isize) -> bool;
    pub fn lua_topointer(_: *mut LuaState, _: isize) -> *const c_void;
    pub fn lua_type(_: *mut LuaState, _: isize) -> isize;
    pub fn lua_typename(_: *mut LuaState, _: isize) -> *const c_char;
    pub fn lua_isstring(_: *mut LuaState, _: isize) -> isize;
}

/// Load the provided buffer as a lua module with the specified name.
/// # Safety
/// Makes a lot of FFI calls, mutates internal C lua state.
pub unsafe fn load_module(state: *mut LuaState, name: &str, buffer: &str, lual_loadbuffer: LoadBuffer) {
    let buf_cstr = CString::new(buffer).unwrap();
    let buf_len = buf_cstr.as_bytes().len();

    let p_name = format!("@{name}");
    let p_name_cstr = CString::new(p_name).unwrap();

    // Push the global package.loaded table onto the top of the stack, saving its index.
    let stack_top = lua_gettop(state);
    lua_getfield(state, LUA_GLOBALSINDEX, b"package\0".as_ptr() as _);
    lua_getfield(state, -1, b"loaded\0".as_ptr() as _);

    // This is the index of the `package.loaded` table.
    let field_index = lua_gettop(state);

    // Load the buffer and execute it via lua_pcall, pushing the result to the top of the stack.
    lual_loadbuffer(state, buf_cstr.into_raw() as _, buf_len as _, p_name_cstr.into_raw() as _);

    lua_pcall(state, 0, -1, 0);

    // Insert pcall results onto the package.loaded global table.
    let module_cstr = CString::new(name).unwrap();

    lua_setfield(state, field_index, module_cstr.into_raw() as _);
    lua_settop(state, stack_top);
}

/// An override print function, copied piecemeal from the Lua 5.1 source, but in Rust.
/// # Safety
/// Native lua API access. It's unsafe, it's unchecked, it will probably eat your firstborn.
pub unsafe extern "C" fn override_print(state: *mut LuaState) -> isize {
    let argc = lua_gettop(state);
    let mut out = String::new();

    for i in 0..argc {
        let mut str_len = 0_isize; 
        let arg_str = lua_tolstring(state, -1, &mut str_len);
        if arg_str.is_null() {
            out.push_str("[G] nil");
            continue;
        }
        
        let str_buf = slice::from_raw_parts(arg_str as *const u8, str_len as _);
        let arg_str = String::from_utf8_lossy(str_buf);

        if i > 1 {
            out.push('\t');
        }

        out.push_str(&format!("[G] {arg_str}"));
        lua_settop(state, -(1) - 1);
    }

    info!("{out}");

    0
}
