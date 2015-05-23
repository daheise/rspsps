extern crate libc;
use std::str;
use std::char;
use std::ptr;
use std::path::Path;
use std::fs::File;
use std::ffi::CStr;
use std::ffi::CString;




enum spsps_loc_ {}
enum spsps_errno {}

/*
#[repr(C)]
struct Loc{
	name: *const libc::c_char,
	line: u32,
	column: u32,
}
*/
//This is for the opaque C pointer
enum spsps_parser_ {}
//this will be the safe abstraction
#[derive(Debug)]
struct Parser{
	ptr: *mut spsps_parser_
}

#[derive(Debug)]
struct Loc{
	ptr: *mut spsps_loc_
}


#[link(name = "spsps")]
extern{
	//This returns an owned Parser
	fn spsps_new(name: *const libc::c_char, stream: *mut libc::FILE) -> *mut spsps_parser_;
	fn spsps_free(parser: *mut spsps_parser_);
	//This returns an owned C string.
	fn spsps_loc_to_string(loc: *mut spsps_loc_) -> *mut libc::c_char;
	//This returns a borrowed string
	fn spsps_printchar(xch: libc::c_char) -> *mut libc::c_char;
	fn spsps_consume(parser: *mut spsps_parser_) -> libc::c_char;
	fn spsps_consume_n(parser: *mut spsps_parser_, n: libc::size_t);
	fn spsps_consume_whitespace(parser: *mut spsps_parser_);
	//Not sure what return type to use for bool
	fn spsps_eof(parser: *mut spsps_parser_) -> libc::c_int;
	//This returns an owned Loc 
	fn spsps_loc(parser: *mut spsps_parser_) -> *mut spsps_loc_;
	fn spsps_peek(parser: *mut spsps_parser_) -> libc::c_char;
	fn spsps_peek_n(parser: *mut spsps_parser_, n: libc::size_t) -> *mut libc::c_char;
	fn spsps_peek_str(parser: *mut spsps_parser_, next: *const libc::c_char) -> libc::c_int;
	fn spsps_peek_and_consume(parser: *mut spsps_parser_, next: *const libc::c_char) -> libc::c_int;	
}

impl Parser{
	fn from_file(name: &str, path: &Path) -> Parser {
		unsafe{
			let path = CString::new(path.to_str().unwrap()).unwrap().as_ptr();
			let fd = libc::funcs::c95::stdio::fopen(path,  CString::new("r").unwrap().as_ptr());
			Parser{ ptr: spsps_new(CString::new(name).unwrap().as_ptr(), fd) }
		 }
	}
	
	fn consume(&self) -> String {
		unsafe {
			char::from_u32(spsps_consume(self.ptr) as u32).unwrap().to_string()
		}
	}
	
	fn consume_n(&self, n: usize){
		unsafe { spsps_consume_n(self.ptr, n as libc::size_t); }
	}
	
	fn consume_whitespace(&self){
		unsafe { spsps_consume_whitespace(self.ptr); }
	}
	
	fn eof(&self) -> bool{
		unsafe { spsps_eof(self.ptr) as usize != 0 } 
	}
	
	fn get_loc(&self) -> Loc {
		unsafe{
			Loc { ptr: spsps_loc(self.ptr) }
		}
	}
	
	fn peek(&self) -> String{
		unsafe {
			char::from_u32(spsps_peek(self.ptr) as u32).unwrap().to_string()
		}
	}
	
	fn peek_n(&self, n: usize) -> String{
		let cstr = unsafe{ spsps_peek_n(self.ptr, n as libc::size_t) };
		let safe_cstr = unsafe { CStr::from_ptr(cstr) };
		let retval = str::from_utf8(safe_cstr.to_bytes()).unwrap_or("").to_owned();
		unsafe { libc::free(safe_cstr.as_ptr() as *mut libc::c_void); }
		return retval;
	}
	
	fn peek_str(&self, needle: &str) -> bool {
		unsafe { spsps_peek_str(self.ptr, CString::new(needle).unwrap().as_ptr()) != 0 }
	}
	
	fn peek_str_and_consume(&self, needle: &str) -> bool {
		unsafe { spsps_peek_and_consume(self.ptr, CString::new(needle).unwrap().as_ptr()) != 0 }
	}
}

impl Drop for Parser{
	fn drop(&mut self) {
		unsafe{ spsps_free(self.ptr); }
	}
}

impl Loc{
	fn to_string(&self) -> String{
		let tmp = unsafe { CStr::from_ptr(spsps_loc_to_string(self.ptr)) };
		let retval = str::from_utf8(tmp.to_bytes()).unwrap_or("").to_owned();
		unsafe {libc::free(tmp.as_ptr() as *mut libc::c_void);}
		retval
	}
}

impl Drop for Loc{
	fn drop(&mut self) {
		unsafe{ libc::free(self.ptr as *mut libc::c_void); }
	}
}

#[test]
fn test_from_file(){
	let test_path = Path::new(file!());
	let p = Parser::from_file("test", test_path);
	println!("{:?}", p);
}