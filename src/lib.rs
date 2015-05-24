extern crate libc;
#[macro_use]
extern crate log;
extern crate env_logger as logger;
use log::LogLevel;
use std::io::prelude::*;
use std::str;
use std::char;
use std::ptr;
use std::path::Path;
use std::fs;
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
	_ptr: *mut spsps_parser_,
	_fd: *mut libc::FILE
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
	fn from_file(name: Option<&str>, path: &Path) -> Parser {
		unsafe{
			let c_path = CString::new(path.to_str().unwrap()).unwrap().as_ptr();
			debug!("from_file path: {:?}", CString::new(path.to_str().unwrap()).unwrap());
			let fd = libc::funcs::c95::stdio::fopen(c_path,  CString::new("r").unwrap().as_ptr());
			if fd.is_null() {
				panic!("Unable to open file to parse.");
			}
			match name{
			    Some(n) => Parser{ 
				    	_ptr: spsps_new(CString::new(n).unwrap().as_ptr(), fd),
				    	_fd: fd
			    	},
			    None => Parser{ 
				    	_ptr: spsps_new(ptr::null(), fd),
				    	_fd: fd
			    	}
			}
		 }
	}
	
	fn consume(&self) -> String {
		unsafe {
			char::from_u32(spsps_consume(self._ptr) as u32).unwrap().to_string()
		}
	}
	
	fn consume_n(&self, n: usize){
		unsafe { spsps_consume_n(self._ptr, n as libc::size_t); }
	}
	
	fn consume_whitespace(&self){
		unsafe { spsps_consume_whitespace(self._ptr); }
	}
	
	fn eof(&self) -> bool{
		unsafe { spsps_eof(self._ptr) as usize != 0 } 
	}
	
	fn get_loc(&self) -> Loc {
		unsafe{
			Loc { ptr: spsps_loc(self._ptr) }
		}
	}
	
	fn peek(&self) -> String{
		let c_char = unsafe { spsps_peek(self._ptr) };
		char::from_u32(c_char as u32).unwrap().to_string()
	}
	
	fn peek_n(&self, n: usize) -> String{
		let cstr = unsafe{ spsps_peek_n(self._ptr, n as libc::size_t) };
		let safe_cstr = unsafe { CStr::from_ptr(cstr) };
		let retval = str::from_utf8(safe_cstr.to_bytes()).unwrap_or("").to_owned();
		unsafe { libc::free(safe_cstr.as_ptr() as *mut libc::c_void); }
		return retval;
	}
	
	fn peek_str(&self, needle: &str) -> bool {
		unsafe { spsps_peek_str(self._ptr, CString::new(needle).unwrap().as_ptr()) != 0 }
	}
	
	fn peek_str_and_consume(&self, needle: &str) -> bool {
		unsafe { spsps_peek_and_consume(self._ptr, CString::new(needle).unwrap().as_ptr()) != 0 }
	}
}

impl Drop for Parser{
	fn drop(&mut self) {
		unsafe{
			libc::fclose(self._fd);
			spsps_free(self._ptr); 
		}		
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

fn test_parser() -> Parser {
	let mut f = File::create("./foo.txt").unwrap();
	f.write_all(b"This is a test.\n");
	f.sync_data();
	let test_path = Path::new("./foo.txt");
	Parser::from_file(Some("test"), test_path)
}

#[test]
fn test_from_file(){
	logger::init().unwrap();
	let p = test_parser();
	//fs::remove_file("foo.txt").unwrap();
	println!("{:?}", p);
}



#[test]
fn test_peek_n(){
	let p = test_parser();
	p.consume_whitespace();
	let first_chars = p.peek_n(3);
	println!("{:?}", first_chars);
	assert_eq!(first_chars, "Thi");
}

#[test]
fn test_peek(){
	let p = test_parser();
	p.consume_whitespace();
	let first_char = p.peek();
	println!("{:?}", first_char);
	assert_eq!(first_char, "T");
}

#[test]
fn test_consume(){
	let p = test_parser();
	p.consume_whitespace();
	let first_char = p.consume();
	println!("{:?}", first_char);
	assert_eq!(first_char, "T");
}

#[test]
fn test_loc_to_string(){
	let p = test_parser();
	let location = p.get_loc();
	let loc_string = location.to_string();
	println!("{}",loc_string);
	assert_eq!(loc_string, "test:1:1");
}

#[test]
fn test_peek_consume(){
	let parser = test_parser();
	parser.consume_whitespace();
	let p = parser.peek();
	let c = parser.consume();
	assert_eq!(p, c);
}

#[test]
fn test_zzz_cleanup()
{
	//fs::remove_file("foo.txt").unwrap();
}