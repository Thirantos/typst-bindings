use typst::diag::SourceDiagnostic;
use typst::layout::PagedDocument;
use typst_pdf::pdf;

use std::ffi::{CStr, CString, c_char};
use std::ptr::null_mut;

use crate::typst_as_lib::TypstWrapperWorld;

mod typst_as_lib;

pub struct TypstWorld {
    typst_wrapper_world: typst_as_lib::TypstWrapperWorld,
}
impl TypstWorld {
    pub fn new(typst_wrapper_world: typst_as_lib::TypstWrapperWorld) -> Self {
        Self {
            typst_wrapper_world,
        }
    }
}

pub struct TypstDocument {
    paged_document: PagedDocument,
}
impl TypstDocument {
    pub fn new(paged_document: PagedDocument) -> Self {
        Self { paged_document }
    }
}

unsafe fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let c_str = unsafe { CStr::from_ptr(ptr) };
    c_str.to_str().ok().map(|s| s.to_owned())
}

#[repr(C)]
pub struct TypstError {
    message: *mut c_char,
}

#[unsafe(no_mangle)]
pub extern "C" fn typst_world_new(root: *mut c_char, source: *mut c_char) -> *mut TypstWorld {
    if let Some(r) = unsafe { c_str_to_string(root) }
        && let Some(s) = unsafe { c_str_to_string(source) }
    {
        Box::into_raw(Box::new(TypstWorld::new(TypstWrapperWorld::new(r, s))))
    } else {
        null_mut()
    }
}

fn get_diagnostics(diagnostics: typst::ecow::EcoVec<SourceDiagnostic>) -> Option<CString> {
    CString::new(
        diagnostics
            .into_iter()
            .map(|diag| diag.message)
            .map(|diag| diag.to_owned().as_str().to_owned())
            .collect::<String>(),
    )
    .ok()
}

#[unsafe(no_mangle)]
pub extern "C" fn typst_world_compile(
    world: *mut TypstWorld,
    err: *mut TypstError,
) -> *mut TypstDocument {
    if world.is_null() {
        return null_mut();
    }
    let w = unsafe { &(*world).typst_wrapper_world };
    let d: Result<PagedDocument, _> = typst::compile(&w).output;
    match d {
        Ok(doc) => Box::into_raw(Box::new(TypstDocument::new(doc))),

        Err(e) => {
            if !err.is_null() {
                if let Some(mesg) = get_diagnostics(e) {
                    unsafe { (*err).message = mesg.into_raw() }
                }
            }
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn typst_document_to_pdf(
    document: *mut TypstDocument,
    len: *mut usize,
    data: *mut *mut u8,
    err: *mut TypstError,
) {
    if document.is_null() {
        return;
    }
    let paged_document = unsafe { &(*document).paged_document };

    let pdf = pdf(&paged_document, &typst_pdf::PdfOptions::default());

    match pdf {
        Ok(mut p) => {
            unsafe {
                *data = p.as_mut_ptr();
                *len = p.len();
            };
            std::mem::forget(p);
        }
        Err(e) => {
            if !err.is_null() {
                if let Some(mesg) = get_diagnostics(e) {
                    unsafe { (*err).message = mesg.into_raw() }
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn typst_world_free(ptr: *mut TypstWorld) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(ptr));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn typst_document_free(ptr: *mut TypstDocument) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(ptr));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn typst_pdf_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Vec::from_raw_parts(ptr, len, len);
    }
}
