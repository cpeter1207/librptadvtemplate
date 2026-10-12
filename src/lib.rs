//! Generic bounded UTF-8 templates behind a versioned C descriptor.

#![deny(warnings)]
#![cfg_attr(coverage, feature(coverage_attribute))]

use std::{ffi::c_char, mem::size_of};

pub mod template;

use template::TemplateError;

const ABI_VERSION: u32 = 1;
const CAPABILITY_NAME: &[u8] = b"rptadv.template\0";
const INVALID_ARGUMENT: i32 = 3;

/// Borrowed byte sequence. Null is valid only when length is zero.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ByteSpan {
    data: *const u8,
    length: usize,
}

/// One caller-defined placeholder name and conservative maximum width.
#[repr(C)]
pub struct KeyLimit {
    name: ByteSpan,
    maximum: usize,
}

/// One caller-defined placeholder name and replacement value.
#[repr(C)]
pub struct KeyValue {
    name: ByteSpan,
    value: ByteSpan,
}

/// ABI 1 generic template function table.
#[repr(C)]
pub struct TemplateDescriptor {
    struct_size: u32,
    abi_version: u32,
    capability_name: *const c_char,
    validate: unsafe extern "C" fn(ByteSpan, *const KeyLimit, usize, usize) -> i32,
    render:
        unsafe extern "C" fn(ByteSpan, *const KeyValue, usize, *mut u8, usize, *mut usize) -> i32,
}

// SAFETY: the table and every address it references are immutable.
unsafe impl Sync for TemplateDescriptor {}

static DESCRIPTOR: TemplateDescriptor = TemplateDescriptor {
    struct_size: size_of::<TemplateDescriptor>() as u32,
    abi_version: ABI_VERSION,
    capability_name: CAPABILITY_NAME.as_ptr().cast::<c_char>(),
    validate: validate_span,
    render: render_span,
};

/// Return the immutable ABI 1 function table.
#[unsafe(no_mangle)]
pub extern "C" fn rptadv_template_descriptor() -> *const TemplateDescriptor {
    &DESCRIPTOR
}

impl ByteSpan {
    unsafe fn text<'a>(self) -> Option<&'a str> {
        if self.length == 0 {
            return Some("");
        }
        if self.data.is_null() {
            return None;
        }
        // SAFETY: the caller supplies a readable length-sized byte span.
        std::str::from_utf8(unsafe { std::slice::from_raw_parts(self.data, self.length) }).ok()
    }
}

unsafe fn records<'a, T>(pointer: *const T, count: usize) -> Option<&'a [T]> {
    if count == 0 {
        return Some(&[]);
    }
    if pointer.is_null() {
        return None;
    }
    // SAFETY: the caller supplies count readable records for this call.
    Some(unsafe { std::slice::from_raw_parts(pointer, count) })
}

fn result_code(result: Result<(), TemplateError>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(TemplateError::InvalidSyntax) => 1,
        Err(TemplateError::OutputTooLong) => 2,
    }
}

unsafe extern "C" fn validate_span(
    text: ByteSpan,
    keys: *const KeyLimit,
    count: usize,
    maximum: usize,
) -> i32 {
    let (Some(text), Some(keys)) = (unsafe { text.text() }, unsafe { records(keys, count) }) else {
        return INVALID_ARGUMENT;
    };
    if keys.iter().any(|key| unsafe { key.name.text() }.is_none()) {
        return INVALID_ARGUMENT;
    }
    result_code(template::validate_with(
        text,
        &|name| {
            keys.iter()
                .find(|key| unsafe { key.name.text() } == Some(name))
                .map(|key| key.maximum)
        },
        maximum,
    ))
}

unsafe extern "C" fn render_span(
    text: ByteSpan,
    values: *const KeyValue,
    count: usize,
    output: *mut u8,
    capacity: usize,
    written: *mut usize,
) -> i32 {
    let (Some(text), Some(values), Some(written)) = (
        unsafe { text.text() },
        unsafe { records(values, count) },
        unsafe { written.as_mut() },
    ) else {
        return INVALID_ARGUMENT;
    };
    *written = 0;
    if values.iter().any(|entry| {
        unsafe { entry.name.text() }.is_none() || unsafe { entry.value.text() }.is_none()
    }) {
        return INVALID_ARGUMENT;
    }
    if capacity != 0 && output.is_null() {
        return INVALID_ARGUMENT;
    }
    let output = if capacity == 0 {
        &mut []
    } else {
        // SAFETY: the caller supplies a writable capacity-sized nonoverlapping span.
        unsafe { std::slice::from_raw_parts_mut(output, capacity) }
    };
    match template::render_with(
        text,
        &|name| {
            values
                .iter()
                .find(|entry| unsafe { entry.name.text() } == Some(name))
                .and_then(|entry| unsafe { entry.value.text() })
        },
        output,
    ) {
        Ok(count) => {
            *written = count;
            0
        }
        Err(error) => result_code(Err(error)),
    }
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod abi_tests;
