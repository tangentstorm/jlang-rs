// an attempt to connect to j from rust.
//
// https://code.jsoftware.com/wiki/Interfaces/JFEX
// https://github.com/jsoftware/jsource/blob/master/jsrc/jlib.h

#[macro_use] extern crate dlopen_derive;
extern crate dlopen;
use std::ffi::CStr;
use std::os::raw::c_char;

use dlopen::wrapper::{WrapperApi, Container};

const JDL: &str = "j.dll";  // TODO: "libj.dylib"; "libj.so"

/// j string. (c string)
pub type JS = *const c_char;
macro_rules! jstr { ($x:expr) => { CStr::from_bytes_with_nul($x).unwrap().as_ptr() } }
macro_rules! jprintln { ($s:expr) => { { jprint!($s); println!(); }}}
macro_rules! jprint { ($s:expr) => {
    let cs = unsafe { CStr::from_ptr($s) };
    print!("{}", cs.to_str().unwrap()) }}


// -- other variants i've tried --
// macro_rules! jstr { ($x:expr) => { std::ptr::addr_of!($x) as JS } }
// macro_rules! jstr { ($x:expr) => { $x.as_ptr() } }
// macro_rules! jprint { ($s:expr)=> {{
//   let mut p = $s;
//   while unsafe { *p } != 0 {
//       print!( "{}", unsafe { *p as u8 } as char);
//       unsafe { p = p.add(1) } }}}}


pub type JI = i64;
pub type JT = usize; // pointer to the interpreter

/// j array type (unused so far)
#[repr(C)] pub struct JA { k:JI, flag:JI, m:JI, t:JI, c:JI, n:JI, r:JI, s:JI, v:*const JI }

pub struct SMTYPE(usize);
pub const SMWIN:SMTYPE = SMTYPE(0);  // j.exe    Jwdw (Windows) front end
pub const SMJAVA:SMTYPE = SMTYPE(2); // j.jar    Jwdp (Java) front end
pub const SMCON:SMTYPE = SMTYPE(3);  // jconsole


/// callbacks for the j session manager
#[repr(C)]
pub struct JCBs {
    /// write a string to the display
    pub wr: extern "C" fn(jt:&JT, len:u32, s:JS),
    /// window driver
    pub wd: extern "C" fn(jt:&JT, x:u32, *const JA, *const *const JA)->i32,
    /// read a string from input
    pub rd: extern "C" fn(jt:&JT, prompt:JS)->JS,
    /// reserved?
    _x: usize,
    /// session type code
    pub sm: SMTYPE
}

/// default write().. prints to stdout
#[no_mangle] pub extern "C" fn wr(_jt:&JT, len:u32, s:JS) {
  println!("GOT HERE. len: {}", len);
  print!("wr:"); jprintln!(s); }

/// default wd(): window driver. (this implementation does nothing)
#[no_mangle] pub extern "C" fn wd(_jt:&JT, _x:u32, _a:*const JA, _z:*const *const JA)->i32 { 0 }

/// default rd(): runs i.3 3 TODO: read from stdin
#[no_mangle] pub extern "C" fn rd<'a>(_jt:&JT, prompt:JS)->JS {
    jprint!(prompt);
    // TODO: actually read in some text
    jstr!(b"i.3 3\0") }

#[derive(WrapperApi)]
pub struct JAPI {
  #[dlopen_name="JInit"] init: extern "C" fn()->JT,
  #[dlopen_name="JFree"] free: extern "C" fn(jt:JT),
  #[dlopen_name="JDo"]   jdo: extern "C" fn(jt:&JT, s:JS)->JI,
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
  c.jdo(&jt, s);
  println!("calling free()...");
  c.free(jt);
  println!("all done."); }
