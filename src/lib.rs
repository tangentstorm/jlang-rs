// an attempt to connect to j from rust.
//
// https://code.jsoftware.com/wiki/Interfaces/JFEX
// https://github.com/jsoftware/jsource/blob/master/jsrc/jlib.h

#[macro_use] extern crate dlopen_derive;
extern crate dlopen;
// extern crate libc;
// use std::{ffi::{CStr, CString}, os::raw::c_char};
// use std::ptr;

use dlopen::wrapper::{WrapperApi, Container};

const JDL: &str = "j.dll";  // TODO: "libj.dylib"; "libj.so"

/// j string. (c string)
pub type JS = *const u8; // libc::c_char;
// macro_rules! jstr { ($x:expr) => { CString::new($x).unwrap().as_ptr() as JS } }
// macro_rules! jstr { ($x:expr) => { std::ptr::addr_of!($x) as JS } }
macro_rules! jstr { ($x:expr) => { $x.as_ptr() } }
macro_rules! jprint { ($s:expr)=> {{
  let mut p = $s;
  while unsafe { *p } != b'\0' {
      print!( "{}", unsafe { *p } as char);
      unsafe { p = p.add(1) } }}}}
macro_rules! jprintln { ($s:expr) => { { jprint!($s); println!(); }}}


pub type JI = i64;
pub struct JT(usize); // pointer to the interpreter

/// j array type (unused so far)
pub struct JA<'a> { k:JI, flag:JI, m:JI, t:JI, c:JI, n:JI, r:JI, s:JI, v:&'a JI }

pub struct SMTYPE(usize);
pub const SMWIN:SMTYPE = SMTYPE(0);  // j.exe    Jwdw (Windows) front end
pub const SMJAVA:SMTYPE = SMTYPE(2); // j.jar    Jwdp (Java) front end
pub const SMCON:SMTYPE = SMTYPE(3);  // jconsole


/// callbacks for the j session manager
pub struct JCBs {
    /// write a string to the display
    pub wr: fn(jt:&JT, len:u32, s:&JS),
    /// window driver
    pub wd: fn(jt:&JT, x:u32, &JA, &&JA)->i32,
    /// read a string from input
    pub rd: fn(jt:&JT, prompt:JS)->JS,
    /// reserved?
    _x: usize,
    /// session type code
    pub sm: SMTYPE
}

/// default write().. prints to stdout
pub fn wr(_jt:&JT, len:u32, s:&JS) {
  println!("GOT HERE. len: {}", len);
  println!("{:?}", unsafe{ *s }); }

/// default wd(): does nothing
pub fn wd(_jt:&JT, x:u32, a:&JA, z:&&JA)->i32 { 0 }

/// default rd(): runs i.3 3 TODO: read from stdin
pub fn rd<'a>(_jt:&JT, _prompt:JS)->JS { jstr!(b"i.3 3\0") }

#[derive(WrapperApi)]
pub struct JAPI {
  #[dlopen_name="JInit"] init: extern "C" fn()->JT,
  #[dlopen_name="JFree"] free: extern "C" fn(jt:JT),
  #[dlopen_name="JDo"]   jdo: extern "C" fn(jt:&JT, s:&JS)->JI,
  #[dlopen_name="JSM"]   jsm: extern "C" fn(jt:&JT, jcbs:JCBs),
//   #[dlopen_name="JGetA"] geta: extern "C" fn(jt:&JT, ji:JI, name:JS)->JA<'a>,
//   #[dlopen_name="JGetM"] getm: extern "C" fn(jt:&JT),
//   #[dlopen_name="JSetM"] setm: extern "C" fn(jt:&JT),
}

pub fn load<'a>()->Container<JAPI> { unsafe { Container::load(JDL).unwrap() }}

/// run with `cargo test --lib -- --nocapture`
#[test]fn test_demo() {
  println!("loading dll...");
  let c = load();
  println!("calling init()...");
  let jt = c.init();
  println!("calling jsm()...");
  c.jsm(&jt, JCBs{wr, wd, rd, _x:0, sm:SMCON });
  println!("building string for jdo()...");
  let prompt = jstr!(b"j> \0");
  print!("prompt: "); jprintln!(prompt);
  let s = rd(&jt, prompt);
  print!("input: "); jprintln!(s);
  println!("calling jdo()...");
  // --- crash occurs here: ----
  c.jdo(&jt, &s);
  println!("calling free()...");
  c.free(jt);
  println!("all done."); }
