// an attempt to connect to j from rust.
//
// https://code.jsoftware.com/wiki/Interfaces/JFEX
// https://github.com/jsoftware/jsource/blob/master/jsrc/jlib.h
// https://github.com/jsoftware/jsource/blob/master/jsrc/io.c
// https://code.jsoftware.com/wiki/Guides/DLLs/Calling_the_J_DLL

#[macro_use] extern crate dlopen_derive;
extern crate dlopen;
use std::ffi::CStr;
use std::os::raw::c_char;

use dlopen::wrapper::{WrapperApi, Container};

#[cfg(target_os="windows")]
const JDL: &str = "j.dll";
#[cfg(target_os="linux")]
const JDL: &str = "./libj.so";

/// j string. (c string)
#[repr(C)] pub struct JS{p:*const c_char}
impl JS {
  pub fn from_ptr(p:*const c_char)->JS { JS { p }}
  pub fn to_cstr(&self)->&CStr { unsafe { CStr::from_ptr(self.p) }}
  pub fn to_str(&self)->&str { self.to_cstr().to_str().unwrap() }}
macro_rules! jstr { ($x:expr) => { JS{p:CStr::from_bytes_with_nul($x).unwrap().as_ptr() }} }

/// arbitrary untyped pointer (void* in c)
type VOIDP = *const u8;
/// J integer type (TODO: support 32-bit as well?)
pub type JI = i64;
/// opaque pointer to J interpreter.
#[repr(C)] #[derive(Clone,Copy)] pub struct JT(*const u8);

// pointer to j integer
pub type PJI = *const JI;

/// j array type
#[repr(C)] #[derive(Debug)] pub struct JA { k:JI, flag:JI, m:JI, t:JI, c:JI, n:JI, r:JI, s:PJI, v:PJI }

/// code indicating the type of session. sent to jsm()
#[repr(C)] pub struct SMTYPE(usize);
/// SMTYPE = windows (jqt? any gui platform with wd?)
pub const SMWIN:SMTYPE = SMTYPE(0);
/// SMTYPE = java frontend (not sure what this is.)
pub const SMJAVA:SMTYPE = SMTYPE(2);
/// SMTYPE = jconsole
pub const SMCON:SMTYPE = SMTYPE(3);

/// jsm callback for writing a string to the display.
type JWrFn = extern "C" fn(jt:JT, len:u32, s:JS);
/// jsm callback for the window driver.
type JWdFn = extern "C" fn(jt:JT, x:u32, *const JA, *const *const JA)->i32;
/// jsm callback for reading a string from the user.
type JRdFn = extern "C" fn(jt:JT, prompt:JS)->JS;

/// callbacks for j session manager (to create an interactive ui)
#[repr(C)] pub struct JCBs {
  /// write a string to the display
  pub wr: JWrFn,
  /// window driver
  pub wd: JWdFn,
  /// read a string from input
  pub rd: JRdFn,
  /// reserved?
  _x: VOIDP,
  /// session type code
  pub sm: SMTYPE }

/// default write().. prints to stdout
#[no_mangle] pub extern "C" fn wr(_jt:JT, len:u32, s:JS) {
  print!("wr(len:{}, s:{})", len, s.to_str()); }

/// default wd(): window driver. (this implementation does nothing)
#[no_mangle] pub extern "C" fn wd(_jt:JT, _x:u32, _a:*const JA, _z:*const *const JA)->i32 { 0 }

/// default rd(): runs i.3 3 TODO: read from stdin
#[no_mangle] pub extern "C" fn rd<'a>(_jt:JT, prompt:JS)->JS {
  print!("{}", prompt.to_str()); // TODO: actually read in some text
  jstr!(b"i.3 3\0") }

#[derive(WrapperApi)]
pub struct JAPI {
  /// initialize the j interpreter
  #[dlopen_name="JInit"] init: extern "C" fn()->JT,

  /// free the j interpreter
  #[dlopen_name="JFree"] free: extern "C" fn(jt:JT),

  /// execute a j sentence
  #[dlopen_name="JDo"]   jdo: extern "C" fn(jt:JT, s:JS)->JI,

  /// get the 'captured' response (output)
  /// Use this to get the response from J if you do not set up
  /// i/o callbacks using jsm().
  #[dlopen_name="JGetR"] getr: extern "C" fn(jt:JT)->JS,

  /// Initialize J Session Manager
  #[dlopen_name="JSM"]   jsm: extern "C" fn(jt:JT, jcbs:JCBs),
  // #[dlopen_name="JSMX"]  jsmx: extern "C"
  // fn(jt:JT, wr:JWrFn, wd:JWdFn, rd:JRdFn, _x:VOIDP, sm:SMTYPE)

  /// fetch byte representation: (3!:1) name. len seems to be the length of the name
  #[dlopen_name="JGetA"] geta: extern "C" fn(jt:JT, len:JI, nm:JS)->JA,

  /// get named noun as (type, rank, shape, data)
  #[dlopen_name="JGetM"] getm: extern "C" fn(jt:JT, nm:JS, t:&mut JI, r:&mut JI, sh:&mut PJI, d:&mut VOIDP),

  /// set named noun as (type, rank, shape, data)
  #[dlopen_name="JSetM"] setm: extern "C" fn(jt:JT, nm:JS, t:&mut JI, r:&mut JI, sh:&mut PJI, d:&mut VOIDP) }


pub type JContainer = Container<JAPI>;
pub fn load()->JContainer { unsafe { Container::load(JDL).unwrap() }}

/// run with `cargo test --lib`   # add `-- --nocapture` to see println!() calls
#[test]fn test_demo() {
  // connect to j and run a simple command:
  let c = load();
  let jt = c.init();
  c.jdo(jt, jstr!(b"m =. *: i. 2 3\0"));
  c.jdo(jt, jstr!(b"m\0"));
  assert_eq!("0  1  4\n9 16 25\n", c.getr(jt).to_str());

  // now fetch the actual data.
  let mut t:JI=0; let mut r:JI=0; let mut sh:PJI=std::ptr::null_mut(); let mut d:VOIDP=std::ptr::null_mut();
  c.getm(jt, jstr!(b"m\0"), &mut t, &mut r, &mut sh, &mut d);
  const JDT_INT:JI = 4; assert_eq!(JDT_INT, t);
  assert_eq!(2, r); // answer is a rank 2 array
  let rows = unsafe { *sh }; assert_eq!(rows, 2, "expect 2 rows");
  let cols = unsafe { sh = sh.add(1); *sh }; assert_eq!(cols, 3, "expect 3 cols");
  let mut res : Vec<JI> = vec![];
  let mut d = d as *const JI; // since t told us it's an array of J ints.
  unsafe { for _ in 0..(rows*cols) { res.push(*d); d = d.add(1); }}
  assert_eq!(res, [0, 1, 4, 9, 16, 25]);

  // all done. kill the session:
  c.free(jt); }
