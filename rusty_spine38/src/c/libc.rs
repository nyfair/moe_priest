#![allow(clippy::missing_const_for_fn)]

pub mod types {
    #[allow(non_camel_case_types)]
    pub type c_short = libc::c_short;
    #[allow(non_camel_case_types)]
    pub type c_ushort = libc::c_ushort;
    #[allow(non_camel_case_types)]
    pub type c_int = libc::c_int;
    #[allow(non_camel_case_types)]
    pub type c_uint = libc::c_uint;
    #[allow(non_camel_case_types)]
    pub type c_long = libc::c_long;
    #[allow(non_camel_case_types)]
    pub type c_ulong = libc::c_ulong;
    #[allow(non_camel_case_types)]
    pub type c_schar = libc::c_schar;
    #[allow(non_camel_case_types)]
    pub type c_char = libc::c_char;
    #[allow(non_camel_case_types)]
    pub type c_uchar = libc::c_uchar;
    #[allow(non_camel_case_types)]
    pub type c_float = libc::c_float;
    #[allow(non_camel_case_types)]
    pub type c_double = libc::c_double;
    #[allow(non_camel_case_types)]
    pub type c_void = libc::c_void;
}
use types::*;

#[allow(non_camel_case_types)]
type size_t = libc::size_t;
#[allow(non_camel_case_types)]
type FILE = libc::FILE;

#[no_mangle]
unsafe extern "C" fn spine_strlen_38(s: *const c_char) -> size_t {
    libc::strlen(s)
}

#[no_mangle]
unsafe extern "C" fn spine_strcmp_38(s1: *const c_char, s2: *const c_char) -> c_int {
    libc::strcmp(s1, s2)
}

#[no_mangle]
unsafe extern "C" fn spine_strncmp_38(s1: *const c_char, s2: *const c_char, n: size_t) -> c_int {
    libc::strncmp(s1, s2, n)
}

#[no_mangle]
unsafe extern "C" fn spine_strcasecmp_38(s1: *const c_char, s2: *const c_char) -> c_int {
    #[cfg(target_env = "msvc")]
    {
        libc::stricmp(s1, s2)
    }
    #[cfg(not(target_env = "msvc"))]
    {
        libc::strcasecmp(s1, s2)
    }
}

#[no_mangle]
unsafe extern "C" fn spine_strcpy_38(dest: *mut c_char, src: *const c_char) -> *mut c_char {
    libc::strcpy(dest, src)
}

#[no_mangle]
unsafe extern "C" fn spine_strncpy_38(
    dest: *mut c_char,
    src: *const c_char,
    num: size_t,
) -> *mut c_char {
    libc::strncpy(dest, src, num)
}

#[no_mangle]
unsafe extern "C" fn spine_strncat_38(
    dest: *mut c_char,
    src: *const c_char,
    n: size_t,
) -> *mut c_char {
    libc::strncat(dest, src, n)
}

#[no_mangle]
unsafe extern "C" fn spine_strtol_38(
    nptr: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> c_long {
    libc::strtol(nptr, endptr, base)
}

#[no_mangle]
unsafe extern "C" fn spine_strtoul_38(
    nptr: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> c_ulong {
    libc::strtoul(nptr, endptr, base)
}

#[no_mangle]
unsafe extern "C" fn spine_strrchr_38(s: *const c_char, c: c_int) -> *mut c_char {
    libc::strrchr(s, c)
}

#[no_mangle]
unsafe extern "C" fn spine_rand_38() -> c_int {
    libc::rand()
}

#[no_mangle]
extern "C" fn spine_sqrtf_38(x: c_float) -> c_float {
    x.sqrt()
}

#[no_mangle]
extern "C" fn spine_ceil_38(x: c_double) -> c_double {
    x.ceil()
}

#[no_mangle]
extern "C" fn spine_acosf_38(x: c_float) -> c_float {
    x.acos()
}

#[no_mangle]
extern "C" fn spine_atan2f_38(x: c_float, y: c_float) -> c_float {
    x.atan2(y)
}

#[no_mangle]
extern "C" fn spine_cosf_38(x: c_float) -> c_float {
    x.cos()
}

#[no_mangle]
extern "C" fn spine_sinf_38(x: c_float) -> c_float {
    x.sin()
}

#[no_mangle]
extern "C" fn spine_pow_38(x: c_double, y: c_double) -> c_double {
    x.powf(y)
}

#[no_mangle]
extern "C" fn spine_fmodf_38(x: c_float, y: c_float) -> c_float {
    x % y
}

#[no_mangle]
unsafe extern "C" fn spine_malloc_38(size: size_t) -> *mut c_void {
    libc::malloc(size)
}

#[no_mangle]
unsafe extern "C" fn spine_realloc_38(ptr: *mut c_void, size: size_t) -> *mut c_void {
    libc::realloc(ptr, size)
}

#[no_mangle]
unsafe extern "C" fn spine_free_38(ptr: *mut c_void) {
    libc::free(ptr)
}

#[no_mangle]
unsafe extern "C" fn spine_memcpy_38(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void {
    libc::memcpy(dest, src, n)
}

#[no_mangle]
unsafe extern "C" fn spine_memmove_38(
    dest: *mut c_void,
    src: *const c_void,
    n: size_t,
) -> *mut c_void {
    libc::memmove(dest, src, n)
}

#[no_mangle]
unsafe extern "C" fn spine_memset_38(s: *mut c_void, c: c_int, n: size_t) -> *mut c_void {
    libc::memset(s, c, n)
}

macro_rules! spine_sprintf {
    ($str:expr, $format:expr) => {
        #[cfg(target_env = "msvc")]
        sprintf($str as *mut c_char, $format as *const c_char);
        #[cfg(not(target_env = "msvc"))]
        libc::sprintf($str, $format);
    };
    ($str:expr, $format:expr, $($arg:expr),+ $(,)?) => {
        #[cfg(target_env = "msvc")]
        sprintf($str as *mut c_char, $format as *const c_char, $($arg),*);
        #[cfg(not(target_env = "msvc"))]
        libc::sprintf($str, $format, $($arg),*)
    };
}

macro_rules! spine_sscanf {
    ($str:expr, $format:expr) => {
        #[cfg(target_env = "msvc")]
        sscanf($str as *const c_char, $format as *const c_char);
        #[cfg(not(target_env = "msvc"))]
        libc::sscanf($str, $format);
    };
    ($str:expr, $format:expr, $($arg:expr),+ $(,)? ) => {
        #[cfg(target_env = "msvc")]
        sscanf($str as *const c_char, $format as *const c_char, $($arg),*);
        #[cfg(not(target_env = "msvc"))]
        libc::sscanf($str, $format, $($arg),+);
    };
}

#[no_mangle]
unsafe extern "C" fn spine_fopen_38(filename: *const c_char, modes: *const c_char) -> *mut FILE {
    libc::fopen(filename, modes)
}

#[no_mangle]
unsafe extern "C" fn spine_fclose_38(stream: *mut FILE) -> c_int {
    libc::fclose(stream)
}

#[no_mangle]
unsafe extern "C" fn spine_fread_38(
    ptr: *mut c_void,
    size: size_t,
    n: size_t,
    stream: *mut FILE,
) -> size_t {
    libc::fread(ptr, size, n, stream)
}

#[no_mangle]
unsafe extern "C" fn spine_fseek_38(stream: *mut FILE, off: c_long, whence: c_int) -> c_int {
    libc::fseek(stream, off, whence)
}

#[no_mangle]
unsafe extern "C" fn spine_ftell_38(stream: *mut FILE) -> c_long {
    libc::ftell(stream)
}
