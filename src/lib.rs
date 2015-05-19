extern crate libc;
use std::str;
use std::ptr;
use std::ffi::CStr;

struct Loc;
struct Errno;

#[link(name = "spsps")]
extern{
	//This returns and owned C string.
	fn spsps_loc_to_string(ptr: *mut Loc) -> *mut libc::c_char;
}

fn loc_to_string(loc: &mut Loc) -> String{
	let tmp = unsafe { CStr::from_ptr(spsps_loc_to_string(loc)) };
	let retval = str::from_utf8(tmp.to_bytes()).unwrap_or("").to_owned();
	unsafe {libc::free(tmp.as_ptr() as *mut libc::c_void);}
	retval
 }