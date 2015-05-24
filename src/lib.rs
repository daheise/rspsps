//! A library the provides a safe Rust wrapper for 
//! [Stacy's Pathetically Simple Parsing System](https://github.com/sprowell/spsps).

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



/// For now we're treating spsps_loc_ as an opaque pointer.
enum spsps_loc_ {}
/// An opaque type for spsps_errno. Currently unused.
enum spsps_errno {}

/*
#[repr(C)]
struct Loc{
    name: *const libc::c_char,
    line: u32,
    column: u32,
}
*/

/// This is for the opaque C pointer to spsps_parser_
enum spsps_parser_ {}
/// This is the safe abstraction over a spsps parser.
#[derive(Debug)]
pub struct Parser{
    /// The raw pointer to a C spsps parser.
    _ptr: *mut spsps_parser_,
    /// A raw pointer a C file stream. May be null for stdin.
    _fd: *mut libc::FILE
}

#[derive(Debug)]
/// A safe abstruction over a raw spsps_loc_ pointer.
pub struct Loc{
    /// The *mut spsps_loc_pointer. 
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
    /// Create a new `Parser` with zero configuration. Name will use the default from libspsps,
    /// and will read from stdin.
    pub fn new() -> Parser {
        unsafe{
            Parser{ 
                _ptr: spsps_new(ptr::null(), ptr::null_mut()),
                _fd: ptr::null_mut()
            }
         }
    }
    
    /// Create a Parser that will read from the file given in `path`. If `None` is given for `name`
    /// the default spsps parser name will be used.
    pub fn from_file(name: Option<&str>, path: &Path) -> Parser {
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
    
    /// Consume and return the next character in the stream.
    pub fn consume(&self) -> String {
        unsafe {
            char::from_u32(spsps_consume(self._ptr) as u32).unwrap().to_string()
        }
    }
    
    /// Consume and discard the next `n` characters from the stream.
    pub fn consume_n(&self, n: usize){
        unsafe { spsps_consume_n(self._ptr, n as libc::size_t); }
    }
    
    /// Consume and discard all whitespace.  When this method returns the next
    /// character is the first non-whitespace character, or the end of file has
    /// been reached.
    pub fn consume_whitespace(&self){
        unsafe { spsps_consume_whitespace(self._ptr); }
    }
    
    ///Determine if the end of file has been consumed.
    pub fn eof(&self) -> bool{
        unsafe { spsps_eof(self._ptr) as usize != 0 } 
    }
    
    /// Get the current location in the stream.  This is the location of the next
    /// character to be read, unless the end of stream has been reached.
    pub fn get_loc(&self) -> Loc {
        unsafe{
            Loc { ptr: spsps_loc(self._ptr) }
        }
    }
    
    ///Peek and return the next character in the stream.  The character is not consumed.
    pub fn peek(&self) -> String{
        let c_char = unsafe { spsps_peek(self._ptr) };
        char::from_u32(c_char as u32).unwrap().to_string()
    }
    
    /// Peek ahead at the next few characters in the stream, and return them.  The
    /// The number of characters must be below the lookahead limit.  End of file causes the
    /// returned string to be populated by EOF characters. 
    pub fn peek_n(&self, n: usize) -> String{
        let cstr = unsafe{ spsps_peek_n(self._ptr, n as libc::size_t) };
        let safe_cstr = unsafe { CStr::from_ptr(cstr) };
        let retval = str::from_utf8(safe_cstr.to_bytes()).unwrap_or("").to_owned();
        unsafe { libc::free(safe_cstr.as_ptr() as *mut libc::c_void); }
        return retval;
    }
    
    /// Peek ahead and determine if the next characters in the stream are the given
    /// characters, in sequence.  That is, the given string must be the next thing in the stream.
    pub fn peek_str(&self, needle: &str) -> bool {
        unsafe { spsps_peek_str(self._ptr, CString::new(needle).unwrap().as_ptr()) != 0 }
    }
    
    ///Peek ahead at the next few characters and if they are a given string, then
    /// consume them.  Otherwise leave the stream unchanged.
    pub fn peek_str_and_consume(&self, needle: &str) -> bool {
        unsafe { spsps_peek_and_consume(self._ptr, CString::new(needle).unwrap().as_ptr()) != 0 }
    }
}

impl Drop for Parser{
    fn drop(&mut self) {
        unsafe{
            if !self._ptr.is_null() { spsps_free(self._ptr); }
            if !self._fd.is_null() { libc::fclose(self._fd); } 
        }        
    }
}

impl ToString for Loc{
    ///Convert this location to a short string.
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
    let first_chars = p.peek_n(4);
    println!("{}", first_chars);
    assert_eq!(first_chars, "This");
}

#[test]
fn test_peek(){
    let p = test_parser();
    let first_char = p.peek();
    println!("{:?}", first_char);
    assert_eq!(first_char, "T");
}

#[test]
fn test_consume(){
    let p = test_parser();
    let first_char = p.consume();
    println!("{}", first_char);
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
    let p = parser.peek();
    let c = parser.consume();
    assert_eq!(p, c);
}

#[test]
fn test_zzz_cleanup()
{
    fs::remove_file("foo.txt").unwrap();
}