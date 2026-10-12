use super::*;

#[test]
fn descriptor_validates_and_renders_borrowed_spans() {
    let api = unsafe { rptadv_template_descriptor().as_ref() }.expect("descriptor");
    assert_eq!(api.abi_version, 1);
    assert_eq!(
        api.struct_size as usize,
        std::mem::size_of::<TemplateDescriptor>()
    );
    assert_eq!(
        unsafe { std::ffi::CStr::from_ptr(api.capability_name) }.to_bytes(),
        b"rptadv.template"
    );
    let span = |text: &str| ByteSpan {
        data: text.as_ptr(),
        length: text.len(),
    };
    let keys = [KeyLimit {
        name: span("name"),
        maximum: 4,
    }];
    assert_eq!(
        unsafe { (api.validate)(span("Hi ${name}!"), keys.as_ptr(), 1, 9) },
        0
    );
    let values = [KeyValue {
        name: span("name"),
        value: span("Jöhn"),
    }];
    let mut output = [0u8; 16];
    let mut written = usize::MAX;
    assert_eq!(
        unsafe {
            (api.render)(
                span("Hi ${name}!"),
                values.as_ptr(),
                1,
                output.as_mut_ptr(),
                output.len(),
                &mut written,
            )
        },
        0
    );
    assert_eq!(&output[..written], "Hi Jöhn!".as_bytes());
}

#[test]
fn descriptor_rejects_invalid_spans_and_leaves_failed_output_untouched() {
    let api = unsafe { &*rptadv_template_descriptor() };
    let span = |text: &str| ByteSpan {
        data: text.as_ptr(),
        length: text.len(),
    };
    let empty = ByteSpan {
        data: std::ptr::null(),
        length: 0,
    };
    let invalid = ByteSpan {
        data: std::ptr::null(),
        length: 1,
    };
    let invalid_utf8 = [0xffu8];
    let non_utf8 = ByteSpan {
        data: invalid_utf8.as_ptr(),
        length: 1,
    };
    let keys = [KeyLimit {
        name: span("name"),
        maximum: 4,
    }];
    for (text, records, count) in [
        (invalid, keys.as_ptr(), 1),
        (non_utf8, keys.as_ptr(), 1),
        (span("${name}"), std::ptr::null(), 1),
    ] {
        assert_eq!(unsafe { (api.validate)(text, records, count, 127) }, 3);
    }
    for name in [invalid, non_utf8] {
        let invalid_keys = [KeyLimit { name, maximum: 4 }];
        assert_eq!(
            unsafe { (api.validate)(span("${name}"), invalid_keys.as_ptr(), 1, 127) },
            3
        );
    }
    assert_eq!(
        unsafe { (api.validate)(span("${bad}"), keys.as_ptr(), 1, 127) },
        1
    );
    assert_eq!(
        unsafe { (api.validate)(span("${name"), keys.as_ptr(), 1, 127) },
        1
    );
    assert_eq!(
        unsafe { (api.validate)(span("${name}"), keys.as_ptr(), 1, 3) },
        2
    );
    assert_eq!(unsafe { (api.validate)(empty, std::ptr::null(), 0, 0) }, 0);

    let values = [KeyValue {
        name: span("name"),
        value: span("Jöhn"),
    }];
    let mut output = [0xa5u8; 8];
    for (text, records, count, capacity, result) in [
        (invalid, values.as_ptr(), 1, 8, 3),
        (non_utf8, values.as_ptr(), 1, 8, 3),
        (span("${name}"), std::ptr::null(), 1, 8, 3),
        (span("${other}"), values.as_ptr(), 1, 8, 1),
        (span("${name"), values.as_ptr(), 1, 8, 1),
        (span("${name}"), values.as_ptr(), 1, 4, 2),
    ] {
        let mut written = usize::MAX;
        assert_eq!(
            unsafe {
                (api.render)(
                    text,
                    records,
                    count,
                    output.as_mut_ptr(),
                    capacity,
                    &mut written,
                )
            },
            result
        );
        if result != 3 {
            assert_eq!(written, 0);
        }
        assert_eq!(output, [0xa5; 8]);
    }
    let mut written = usize::MAX;
    for (name, value) in [
        (invalid, span("ok")),
        (span("name"), invalid),
        (non_utf8, span("ok")),
        (span("name"), non_utf8),
    ] {
        let invalid_values = [KeyValue { name, value }];
        assert_eq!(
            unsafe {
                (api.render)(
                    span("${name}"),
                    invalid_values.as_ptr(),
                    1,
                    output.as_mut_ptr(),
                    8,
                    &mut written,
                )
            },
            3
        );
        assert_eq!(written, 0);
    }
    assert_eq!(
        unsafe {
            (api.render)(
                span("ok"),
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                2,
                &mut written,
            )
        },
        3
    );
    assert_eq!(written, 0);
    assert_eq!(
        unsafe {
            (api.render)(
                span("ok"),
                std::ptr::null(),
                0,
                output.as_mut_ptr(),
                8,
                std::ptr::null_mut(),
            )
        },
        3
    );
    assert_eq!(
        unsafe {
            (api.render)(
                empty,
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                0,
                &mut written,
            )
        },
        0
    );
    assert_eq!(written, 0);
    assert_eq!(output, [0xa5; 8]);
}
