//! Contains definitions from `lualib.h`.

use std::alloc;
use std::ffi::CStr;
use std::os::raw::{c_char, c_float, c_int, c_void};
use std::ptr;

use super::lua::{
    self, lua_CFunction, lua_Integer, lua_Number, lua_State, lua_Unsigned, LUA_ERRSYNTAX, LUA_OK,
    LUA_REGISTRYINDEX,
};
use super::luacode;

#[repr(C)]
pub struct luaL_Reg {
    pub name: *const c_char,
    pub func: lua_CFunction,
}

extern "C" {
    pub fn luaL_register(L: *mut lua_State, libname: *const c_char, l: *const luaL_Reg);
    #[link_name = "luaL_getmetafield"]
    pub fn luaL_getmetafield_(L: *mut lua_State, obj: c_int, e: *const c_char) -> c_int;
    pub fn luaL_callmeta(L: *mut lua_State, obj: c_int, e: *const c_char) -> c_int;
    #[link_name = "luaL_typeerrorL"]
    pub fn luaL_typeerror(L: *mut lua_State, narg: c_int, tname: *const c_char) -> !;
    #[link_name = "luaL_argerrorL"]
    pub fn luaL_argerror(L: *mut lua_State, narg: c_int, extramsg: *const c_char) -> !;
    pub fn luaL_checklstring(L: *mut lua_State, narg: c_int, l: *mut usize) -> *const c_char;
    pub fn luaL_optlstring(
        L: *mut lua_State,
        narg: c_int,
        def: *const c_char,
        l: *mut usize,
    ) -> *const c_char;
    pub fn luaL_checknumber(L: *mut lua_State, narg: c_int) -> lua_Number;
    pub fn luaL_optnumber(L: *mut lua_State, narg: c_int, def: lua_Number) -> lua_Number;

    pub fn luaL_checkboolean(L: *mut lua_State, narg: c_int) -> c_int;
    pub fn luaL_optboolean(L: *mut lua_State, narg: c_int, def: c_int) -> c_int;

    pub fn luaL_checkinteger(L: *mut lua_State, narg: c_int) -> lua_Integer;
    pub fn luaL_optinteger(L: *mut lua_State, narg: c_int, def: lua_Integer) -> lua_Integer;
    pub fn luaL_checkunsigned(L: *mut lua_State, narg: c_int) -> lua_Unsigned;
    pub fn luaL_optunsigned(L: *mut lua_State, narg: c_int, def: lua_Unsigned) -> lua_Unsigned;

    pub fn luaL_checkvector(L: *mut lua_State, narg: c_int) -> *const c_float;
    pub fn luaL_optvector(L: *mut lua_State, narg: c_int, def: *const c_float) -> *const c_float;

    #[link_name = "luaL_checkstack"]
    pub fn luaL_checkstack_(L: *mut lua_State, sz: c_int, msg: *const c_char);
    pub fn luaL_checktype(L: *mut lua_State, narg: c_int, t: c_int);
    pub fn luaL_checkany(L: *mut lua_State, narg: c_int);

    #[link_name = "luaL_newmetatable"]
    pub fn luaL_newmetatable_(L: *mut lua_State, tname: *const c_char) -> c_int;
    pub fn luaL_checkudata(L: *mut lua_State, ud: c_int, tname: *const c_char) -> *mut c_void;

    pub fn luaL_where(L: *mut lua_State, lvl: c_int);

    #[link_name = "luaL_errorL"]
    pub fn luaL_error(L: *mut lua_State, fmt: *const c_char, ...) -> !;

    pub fn luaL_checkoption(
        L: *mut lua_State,
        narg: c_int,
        def: *const c_char,
        lst: *const *const c_char,
    ) -> c_int;

    #[link_name = "luaL_tolstring"]
    pub fn luaL_tolstring_(L: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char;

    pub fn luaL_newstate() -> *mut lua_State;

    // TODO: luaL_findtable
}

//
// Some useful macros (implemented as Rust functions)
//

#[inline(always)]
pub unsafe fn luaL_argcheck(L: *mut lua_State, cond: c_int, arg: c_int, extramsg: *const c_char) {
    if cond == 0 {
        luaL_argerror(L, arg, extramsg);
    }
}

#[inline(always)]
pub unsafe fn luaL_argexpected(L: *mut lua_State, cond: c_int, arg: c_int, tname: *const c_char) {
    if cond == 0 {
        luaL_typeerror(L, arg, tname);
    }
}

#[inline(always)]
pub unsafe fn luaL_checkstring(L: *mut lua_State, n: c_int) -> *const c_char {
    luaL_checklstring(L, n, ptr::null_mut())
}

#[inline(always)]
pub unsafe fn luaL_optstring(L: *mut lua_State, n: c_int, d: *const c_char) -> *const c_char {
    luaL_optlstring(L, n, d, ptr::null_mut())
}

// TODO: luaL_opt

#[inline(always)]
pub unsafe fn luaL_typename(L: *mut lua_State, i: c_int) -> *const c_char {
    lua::lua_typename(L, lua::lua_type(L, i))
}

#[inline(always)]
pub unsafe fn luaL_getmetatable(L: *mut lua_State, n: *const c_char) {
    lua::lua_getfield_(L, LUA_REGISTRYINDEX, n);
}

#[inline(always)]
pub unsafe fn luaL_ref(L: *mut lua_State, t: c_int) -> c_int {
    assert_eq!(t, LUA_REGISTRYINDEX);
    let r = lua::lua_ref(L, -1);
    lua::lua_pop(L, 1);
    r
}

#[inline(always)]
pub unsafe fn luaL_unref(L: *mut lua_State, t: c_int, r#ref: c_int) {
    assert_eq!(t, LUA_REGISTRYINDEX);
    lua::lua_unref(L, r#ref)
}

pub unsafe fn luaL_loadbufferx(
    L: *mut lua_State,
    data: *const c_char,
    mut size: usize,
    name: *const c_char,
    mode: *const c_char,
) -> c_int {
    let chunk_is_text = (*data as u8) >= b'\n';
    if !mode.is_null() {
        let modeb = CStr::from_ptr(mode).to_bytes();
        if !chunk_is_text && !modeb.contains(&b'b') {
            lua::lua_pushfstring(
                L,
                cstr!("attempt to load a binary chunk (mode is '%s')"),
                mode,
            );
            return LUA_ERRSYNTAX;
        } else if chunk_is_text && !modeb.contains(&b't') {
            lua::lua_pushfstring(
                L,
                cstr!("attempt to load a text chunk (mode is '%s')"),
                mode,
            );
            return LUA_ERRSYNTAX;
        }
    }

    if chunk_is_text {
        let data = luacode::luau_compile(data, size, ptr::null_mut(), &mut size);
        let layout = alloc::Layout::from_size_align_unchecked(size, super::super::SYS_MIN_ALIGN);
        let ok = lua::luau_load(L, name, data, size, 0) == 0;
        alloc::dealloc(data as *mut u8, layout);
        if !ok {
            return LUA_ERRSYNTAX;
        }
    } else {
        if lua::luau_load(L, name, data, size, 0) != 0 {
            return LUA_ERRSYNTAX;
        }
    }
    LUA_OK
}

#[inline(always)]
pub unsafe fn luaL_loadbuffer(
    L: *mut lua_State,
    data: *const c_char,
    size: usize,
    name: *const c_char,
) -> c_int {
    luaL_loadbufferx(L, data, size, name, ptr::null())
}

//
// TODO: Generic Buffer Manipulation
//