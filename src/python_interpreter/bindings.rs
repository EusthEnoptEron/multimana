#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use anyhow::{bail, Context};
use std::ffi::{c_char, c_void, CString};
use std::sync::LazyLock;
use tracing::info;
/* These definitions must match corresponding definitions in graminit.h. */
const PY_SINGLE_INPUT: i32 = 256;
const PY_FILE_INPUT: i32 = 257;
const PY_EVAL_INPUT: i32 = 258;
const PY_FUNC_TYPE_INPUT: i32 = 345;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PyGILState_STATE {
    PyGILState_LOCKED,
    PyGILState_UNLOCKED,
}

type Py_CompileString_Fn = extern "cdecl" fn(data: *const c_char, filename: *const c_char, start: i32) -> *const c_void;
type PyImport_ExecCodeModule_Fn = extern "cdecl" fn(name: *const c_char, obj: *const c_void) -> *const c_void;
type PyEval_Fn = extern "cdecl" fn();
type PyEnsure_Fn = extern "cdecl" fn() -> PyGILState_STATE;
type PyRelease_Fn = extern "cdecl" fn(PyGILState_STATE);

static Py_CompileString: LazyLock<Py_CompileString_Fn> = LazyLock::new(|| {
    let module = libmem::find_module("python37.dll").expect("Python module not found");
    unsafe {
        std::mem::transmute(libmem::find_symbol_address(&module, "Py_CompileString")
            .expect("Symbol not found"))
    }
});

static PyImport_ExecCodeModule: LazyLock<PyImport_ExecCodeModule_Fn> = LazyLock::new(|| {
    let module = libmem::find_module("python37.dll").expect("Python module not found");
    unsafe {
        std::mem::transmute(libmem::find_symbol_address(&module, "PyImport_ExecCodeModule")
            .expect("Symbol not found"))
    }
});

static PyErr_Print: LazyLock<PyEval_Fn> = LazyLock::new(|| {
    let module = libmem::find_module("python37.dll").expect("Python module not found");
    unsafe {
        std::mem::transmute(libmem::find_symbol_address(&module, "PyErr_Print")
            .expect("Symbol not found"))
    }
});

static PyGILState_Ensure: LazyLock<PyEnsure_Fn> = LazyLock::new(|| {
    let module = libmem::find_module("python37.dll").expect("Python module not found");
    unsafe {
        std::mem::transmute(libmem::find_symbol_address(&module, "PyGILState_Ensure")
            .expect("Symbol not found"))
    }
});

static PyGILState_Release: LazyLock<PyRelease_Fn> = LazyLock::new(|| {
    let module = libmem::find_module("python37.dll").expect("Python module not found");
    unsafe {
        std::mem::transmute(libmem::find_symbol_address(&module, "PyGILState_Release")
            .expect("Symbol not found"))
    }
});


pub fn py_compile_string(code: &str, filename: &str) -> anyhow::Result<*const c_void> {
    info!("Compiling python script ({filename})");
    let code = CString::new(code).context("Unable to turn code into CString")?;
    let filename = CString::new(filename).context("Unable to turn filename into CString")?;

    let state = PyGILState_Ensure();
    let co = Py_CompileString(code.as_ptr(), filename.as_ptr(), PY_FILE_INPUT);
    
    if co.is_null() {
        // Clear error
        PyErr_Print();
        PyGILState_Release(state);
        bail!("Unable to compile script");
    }
    PyGILState_Release(state);

    Ok(co)
}

pub fn py_exec_code_module(name: &str, co: *const c_void) -> anyhow::Result<*const c_void> {
    info!("Executing python script ({name})");
    let name = CString::new(name).context("Unable to turn name into CString")?;

    let state = PyGILState_Ensure();
    let result = PyImport_ExecCodeModule(name.as_ptr(), co);
    if result.is_null() {
        // Clear error
        PyErr_Print();
        PyGILState_Release(state);

        bail!("Unable to exec code module");
    }
    PyGILState_Release(state);

    Ok(result)
}