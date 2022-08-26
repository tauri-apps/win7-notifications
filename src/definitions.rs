// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// credits:
// - https://github.com/rust-windowing/winit/pull/2057/files#diff-a764696e3a56ef31aaa5dc3bbc8b74b1d9e63059e3a7daa9b1bfa9b4da9a08f6
// - https://gist.github.com/feb387d01ed12b8a5c8f96972d5174f4

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::ffi::c_void;
use windows_sys::{core::*, Win32::Foundation::*};

#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IUnknown,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IUnknown) -> u32,
    pub Release: unsafe extern "system" fn(This: *mut IUnknown) -> u32,
}
#[repr(C)]
pub struct ITaskbarListVtbl {
    pub parent: IUnknownVtbl,
    pub HrInit: unsafe extern "system" fn(This: *mut ITaskbarList) -> HRESULT,
    pub AddTab: unsafe extern "system" fn(This: *mut ITaskbarList, hwnd: HWND) -> HRESULT,
    pub DeleteTab: unsafe extern "system" fn(This: *mut ITaskbarList, hwnd: HWND) -> HRESULT,
    pub ActivateTab: unsafe extern "system" fn(This: *mut ITaskbarList, hwnd: HWND) -> HRESULT,
    pub SetActiveAlt: unsafe extern "system" fn(This: *mut ITaskbarList, hwnd: HWND) -> HRESULT,
}

#[repr(C)]
pub struct ITaskbarList {
    pub lpVtbl: *const ITaskbarListVtbl,
}

pub const CLSID_TaskbarList: GUID = GUID {
    data1: 0x56fdf344,
    data2: 0xfd6d,
    data3: 0x11d0,
    data4: [0x95, 0x8a, 0x00, 0x60, 0x97, 0xc9, 0xa0, 0x90],
};

pub const IID_ITaskbarList: GUID = GUID {
    data1: 0x56FDF342,
    data2: 0xFD6D,
    data3: 0x11D0,
    data4: [0x95, 0x8A, 0x00, 0x60, 0x97, 0xC9, 0xA0, 0x90],
};
