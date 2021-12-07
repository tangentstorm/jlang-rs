// J front-end example
// https://code.jsoftware.com/wiki/Interfaces/JFEX
// https://github.com/jsoftware/jsource/blob/master/jsrc/jlib.h

#[macro_use] extern crate dlopen_derive;
extern crate dlopen;
use std::ffi::CStr;

use dlopen::wrapper::{WrapperApi, Container};

const JDL: &str = "j.dll";  // TODO: "libj.dylib"; "libj.so"

pub type JS<'a> = &'a CStr;
pub type JI = i64;
pub struct JT(usize); // pointer to the interpreter

/// j array type (unused so far)
pub struct JA<'a> { k:JI, flag:JI, m:JI, t:JI, c:JI, n:JI, r:JI, s:JI, v:&'a JI }

pub struct SMTYPE(usize);
pub const SMWIN:SMTYPE = SMTYPE(0);  // j.exe    Jwdw (Windows) front end
pub const SMJAVA:SMTYPE = SMTYPE(2); // j.jar    Jwdp (Java) front end
pub const SMCON:SMTYPE = SMTYPE(3);  // jconsole


/// callbacks for the j session manager
pub struct JCBs<'a> {
    /// write a string to the display
    pub wr: fn(jt:&JT, len:u32, s:&CStr),
    /// window driver
    pub wd: fn(jt:&JT, x:u32, &JA, &&JA)->i32,
    /// read a string from input
    pub rd: fn(jt:&JT, prompt:&CStr)->&'a CStr,
    /// reserved?
    _x: usize,
    /// session type code
    pub sm: SMTYPE
}

/// default write().. prints to stdout
pub fn wr(_jt:&JT, len:u32, s:&CStr) {
  println!("GOT HERE");
  println!("{}", s.to_str().unwrap()) }
/// default wd(): does nothing
pub fn wd(_jt:&JT, x:u32, a:&JA, z:&&JA)->i32 { 0 }
/// default rd(): runs i.3 3 TODO: read from stdin
pub fn rd<'a>(_jt:&JT, _prompt:&CStr)->&'static CStr {
  CStr::from_bytes_with_nul(b"i.3 3\0").unwrap() }


#[derive(WrapperApi)]
pub struct JAPI<'a> {
  #[dlopen_name="JInit"] init: extern "C" fn()->JT,
  #[dlopen_name="JFree"] free: extern "C" fn(jt:JT),
  #[dlopen_name="JGetA"] geta: extern "C" fn(jt:&JT, ji:JI, name:JS<'a>)->JA<'a>,
  #[dlopen_name="JGetM"] getm: extern "C" fn(jt:&JT),
  #[dlopen_name="JSetM"] setm: extern "C" fn(jt:&JT),
  #[dlopen_name="JDo"]   jdo: extern "C" fn(jt:&JT, s:&CStr)->JI,
  #[dlopen_name="JSM"]   jsm: extern "C" fn(jt:&JT, jcbs:JCBs)}

pub fn load<'a>()->Container<JAPI<'a>> { unsafe { Container::load(JDL).unwrap() }}

/// run with `cargo test --lib -- --nocapture`
#[test]fn test_demo() {
  println!("loading dll...");
  let c = load();
  println!("calling init()...");
  let jt = c.init();
  println!("calling jsm()...");
  c.jsm(&jt, JCBs{wr, wd, rd, _x:0, sm:SMCON });
  println!("building string for jdo()...");
  let s = rd(&jt, CStr::from_bytes_with_nul(b"    \0").unwrap());
  println!("s: {:?}", s);
  println!("calling jdo()...");
  c.jdo(&jt, s);
  println!("calling free()...");
  c.free(jt);
  println!("all done."); }
