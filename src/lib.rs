// tool for calling j from rust.
//
// https://code.jsoftware.com/wiki/Interfaces/JFEX
// https://github.com/jsoftware/jsource/blob/master/jsrc/jlib.h
// https://github.com/jsoftware/jsource/blob/master/jsrc/io.c
// https://code.jsoftware.com/wiki/Guides/DLLs/Calling_the_J_DLL

#[macro_use] extern crate dlopen_derive;
extern crate dlopen;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;

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

/// c-style j array type. defined in jtype.h in the j source.
#[repr(C)] #[derive(Debug)] pub struct JAD {
  /// k: union field used when block is on free chain
  k:JI,
  /// f: flag bits for inplacing
  f:JI,
  /// m: "multi-use field"
  m:JI,
  /// t: type (or proxychain when on free chain)
  t:JI,
  /// c: usecount
  c:JI,
  /// n: number of atoms, or 1 for sparse array.
  n:JI,
  /// r: rank
  r:JI,
  /// s: shape
  s:PJI,
  /// v: first integer of the value
  v:[JI;1] }

pub type JA = *const JAD;

/// rust-style representation of j array
#[derive(PartialEq, Eq, Debug)]
pub struct JBin {
  rank: JI,
  shape: Vec<usize>,
  jtype: JI,
  data: Vec<u8> }

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

  /// fetch byte representation: (3!:1) of nm. len is the length of the string.
  #[dlopen_name="JGetA"] geta: extern "C" fn(jt:JT, len:JI, nm:JS)->JA,

  /// get named noun as (type, rank, shape, data)
  #[dlopen_name="JGetM"] getm: extern "C" fn(jt:JT, nm:JS, t:&mut JI, r:&mut JI, sh:&mut PJI, d:&mut VOIDP),

  /// set named noun as (type, rank, shape, data)
  #[dlopen_name="JSetM"] setm: extern "C" fn(jt:JT, nm:JS, t:&mut JI, r:&mut JI, sh:&mut PJI, d:&mut VOIDP) }


#[derive(PartialEq, Eq, Debug)]
pub enum JData {
  Lit(u8),  LitV(Vec<u8>),
  Int(JI),  IntV(Vec<JI>),
  Boxed(Vec<JData>),
  Other }

/// Rust representation of a J noun value.
#[derive(PartialEq, Eq, Debug)]
pub struct JVal {
  pub rank: JI,
  pub shape: Vec<JI>,
  pub data: JData }

pub struct JProc {
  pub c : Container<JAPI>,
  pub jt : JT,
  pub bin_path: String }

impl JProc {

  pub fn load()->JProc {
    use std::{env, path::PathBuf};
    let jh : Result<String,env::VarError> =
      env::var("J_HOME").or_else(|_| Ok(".".to_string()));
    if let Ok(jh) = jh {
      let mut p = PathBuf::from(&jh); p.push(JDL);
      if p.exists() { Self::load_from_path(&p) }
      else { panic!("!! NO J LIBRARY FOUND (at {:?})\n\
        !! Try setting J_HOME environment variable to directory of {}", p, JDL); }}
    else { panic!() }}

  pub fn load_from_path(p:&Path)->JProc {
    let c = unsafe { Container::<JAPI>::load(p.as_os_str()).unwrap() };
    let jt = c.init();
    let jp = JProc { c, jt, bin_path:p.parent().or(Some(Path::new("."))).unwrap().display().to_string() };
    jp.cmd(&("BINPATH_z_ =: '".to_string() + &jp.bin_path + &"'".to_string()));
    jp.cmd("0!:0<BINPATH_z_,'/profile.ijs'");
    jp }

  /// call c.getm internally and convert result to JVal
  pub fn get_v(&self, name: &str)->JVal {
    let mut t:JI=0; let mut rank:JI=0;
    let mut sh:PJI=std::ptr::null_mut();
    let mut d:VOIDP=std::ptr::null_mut();

    // !! str_to_jstr: how to extract macro/function? dropping the cs frees the memory,
    //    so if you just extract these 2 lines directly, you get an empty string.
    let cs = std::ffi::CString::new(name).unwrap();
    let js = JS::from_ptr(cs.as_ptr());

    self.c.getm(self.jt, js, &mut t, &mut rank, &mut sh, &mut d);

    // -- copy shape
    let mut count = 1; // so we can multiply
    let mut shape:Vec<JI> = vec![];
    for _ in 0..rank { unsafe { count *= *sh; shape.push(*sh); sh = sh.add(1); }}

    // -- copy data
    let mut data = JData::Other;
    if t == 32 { // boxed. always a vector so JData stays small
      data = JData::Boxed(vec![]) }
    else if shape.is_empty() { // scalar
      if t == 4 { data = JData::Int( unsafe { *(d as *const JI) })}}
    else { // vector
      if t == 4 {
        let mut v:Vec<JI> = vec![]; let mut p = d as *const JI;
        unsafe { for _ in 0..count { v.push(*p); p = p.add(1); }}
        data = JData::IntV(v); }}
    JVal { rank, shape, data } }

  pub fn get_b(&self, name:&str)->JBin {
    // !! str_to_jstr
    let cs = std::ffi::CString::new(name).unwrap();
    let js = JS::from_ptr(cs.as_ptr());
    let ja = self.c.geta(self.jt, name.len() as JI, js);
    let a = unsafe { &*ja };
    assert_eq!(a.t, 2, "expected literal after jgeta.");
    let mut p = std::ptr::addr_of!(a.v) as *const usize;
    let _f = unsafe { *p }; // flag field
    let t = unsafe { p=p.add(1); *p }; // jtype
    let _c = unsafe { p=p.add(1); *p }; // count
    let r = unsafe { p=p.add(1); *p }; // rank
    // read the shape
    let mut shape = vec![0usize; r];
    for _ in 0..r { unsafe { p = p.add(1); shape.push(*p); } }
    // read the bytes
    unsafe { p = p.add(1); }
    let mut b = p as *const u8;
    let mut data = vec![unsafe { *b } ];
    for _ in 0..(unsafe { (*ja).c }-1) { unsafe { b = b.add(1); data.push(*b); } }
    // return compiled data
    JBin { rank: r as JI, jtype: t as JI, shape, data }}

  /// run a command, returning the status code
  pub fn cmd(&self, s:&str)->JI {
    // !! str_to_jstr
    let cs = std::ffi::CString::new(s).unwrap();
    let js = JS::from_ptr(cs.as_ptr());
    self.c.jdo(self.jt, js)}

  /// run a command and return the binary representation (3!:1)
  pub fn cmd_b(&self, s:&str)->JBin {
    self.cmd(&("RESULT_jrs_ =: ".to_string() + &s.to_string()));
    self.get_b("RESULT_jrs_") }

  /// run a cmd, return result as jval
  pub fn cmd_v(&self, s:&str)->JVal {
    self.cmd(&("RESULT_jrs_ =: ".to_string() + &s.to_string()));
    self.get_v("RESULT_jrs_")}

  /// run a cmd, return result as String
  pub fn cmd_s(&self, s:&str)->String {
    self.cmd(&s);
    let js = self.c.getr(self.jt);
    let mut c = js.to_str().chars();
    c.next_back(); // strip final newline
    c.as_str().to_string() }}

/// run with `cargo test --lib`   # add `-- --nocapture` to see println!() calls
#[test] fn test_demo() {
  // connect to j and run a simple command:
  let jp = JProc::load();
  jp.cmd("m =. *: i. 2 3");
  assert_eq!("0  1  4\n9 16 25", jp.cmd_s("m"));

  // now fetch the actual data.
  let res = jp.get_v("m");
  assert_eq!(res, JVal{ rank:2, shape:vec![2, 3],
    data:JData::IntV(vec![0, 1, 4, 9, 16, 25]) });

  // all done. kill the session:
  jp.c.free(jp.jt); }

#[test] fn test_profile() {
  let jp = JProc::load();
  assert_eq!(jp.bin_path, jp.cmd_s("BINPATH_z_"));
  assert_eq!("&.>", jp.cmd_s("each")); }

#[test] fn test_jget_b() {
  let jp = JProc::load();
  let b = jp.cmd_b("'a'");
  assert_eq!(b, JBin { rank:0, shape:vec![], jtype:2, data:vec![97]})}
