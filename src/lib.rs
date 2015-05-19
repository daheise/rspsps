extern crate libc;
use std::str;
use std::ptr;
use std::ffi::CStr;

#[repr(C)]
struct Loc;
#[repr(C)]
struct Errno;
#[repr(C)]
struct Parser;

#[link(name = "spsps")]
extern{
	//This returns an owned Parser
	fn spsps_new(name: *mut libc::c_char, stream: libc::FILE) -> *mut Parser;
	fn spsps_free(parser: *mut Parser);
	//This returns an owned C string.
	fn spsps_loc_to_string(loc: *mut Loc) -> *mut libc::c_char;
	//This returns a borrowed string
	fn spsps_printchar(xch: libc::c_char) -> *mut libc::c_char;
	fn spsps_consume(parser: *mut Parser) -> libc::c_char;
	fn spsps_consume_n(parser: *mut Parser, n: libc::size_t);
	fn spsps_consume_whitespace(parser: *mut Parser);
	//Not sure what return type to use for bool
	fn spsps_eof(parser: *mut Parser) -> libc::c_int;
	//This returns an owned Loc 
	fn spsps_loc(parser: *mut Parser) -> *mut Loc;
	fn spsps_peek(parser: *mut Parser) -> libc::c_char;
	fn spsps_peek_n(parser: *mut Parser, n: libc::size_t) -> *mut libc::c_char;
	fn spsps_peek_str(parser: *mut Parser, next: *mut libc::c_char) -> libc::c_int;
	fn spsps_peek_and_consume(parser: *mut Parser, next: *mut libc::c_char) -> libc::c_int;
	
}

fn loc_to_string(loc: &mut Loc) -> String{
	let tmp = unsafe { CStr::from_ptr(spsps_loc_to_string(loc)) };
	let retval = str::from_utf8(tmp.to_bytes()).unwrap_or("").to_owned();
	unsafe {libc::free(tmp.as_ptr() as *mut libc::c_void);}
	retval
 }