// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use bindings::Windows::Win32::{
  Foundation as w32f,
  Graphics::Gdi as w32gdi,
  System::Com,
  UI::{Controls, Shell, WindowsAndMessaging as w32wm},
};
use windows::*;

pub fn current_exe_name() -> String {
  std::env::current_exe()
    .unwrap()
    .file_name()
    .unwrap()
    .to_str()
    .unwrap()
    .to_owned()
}

pub struct Color(pub u32, pub u32, pub u32);

impl Color {
  /// conver Color to a integer based color
  pub fn to_int(&self) -> u32 {
    self.0 | (self.1 << 8) | (self.2 << 16)
  }
}

#[cfg(target_pointer_width = "32")]
pub fn get_window_long_ptr(window: w32f::HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
  unsafe { w32wm::GetWindowLongA(window, index) as _ }
}

#[cfg(target_pointer_width = "64")]
pub fn get_window_long_ptr(window: w32f::HWND, index: w32wm::WINDOW_LONG_PTR_INDEX) -> isize {
  unsafe { w32wm::GetWindowLongPtrA(window, index) }
}

#[cfg(target_pointer_width = "32")]
pub fn set_window_long_ptr(
  window: w32f::HWND,
  index: w32wm::WINDOW_LONG_PTR_INDEX,
  value: isize,
) -> isize {
  unsafe { w32wm::SetWindowLongA(window, index, value as _) as _ }
}

#[cfg(target_pointer_width = "64")]
pub fn set_window_long_ptr(
  window: w32f::HWND,
  index: w32wm::WINDOW_LONG_PTR_INDEX,
  value: isize,
) -> isize {
  unsafe { w32wm::SetWindowLongPtrA(window, index, value) }
}

/// Implementation of the `LOWORD` macro.
pub fn get_loword(dword: u32) -> u16 {
  (dword & 0xFFFF) as u16
}

pub unsafe fn primary_monitor() -> w32gdi::HMONITOR {
  let pt = w32f::POINT { x: 0, y: 0 };
  w32gdi::MonitorFromPoint(&pt, w32gdi::MONITOR_DEFAULTTOPRIMARY)
}

pub unsafe fn get_monitor_info(hmonitor: w32gdi::HMONITOR) -> w32gdi::MONITORINFOEXW {
  let mut monitor_info = w32gdi::MONITORINFOEXW::default();
  monitor_info.__AnonymousBase_winuser_L13558_C43.cbSize =
    std::mem::size_of::<w32gdi::MONITORINFOEXW>() as u32;
  w32gdi::GetMonitorInfoW(
    hmonitor,
    &mut monitor_info as *mut w32gdi::MONITORINFOEXW as *mut w32gdi::MONITORINFO,
  );
  monitor_info
}

pub unsafe fn skip_taskbar(hwnd: w32f::HWND) {
  let taskbar_list: Shell::ITaskbarList =
    Com::CoCreateInstance(&Shell::TaskbarList, None, Com::CLSCTX_SERVER)
      .expect("failed to create TaskBarList");
  taskbar_list.DeleteTab(hwnd).expect("DeleteTab failed");
}

pub unsafe fn set_font(hdc: w32gdi::HDC, name: &str, size: i32, weight: i32) {
  let hfont = w32gdi::CreateFontW(
    size,
    0,
    0,
    0,
    weight,
    false.into(),
    false.into(),
    false.into(),
    w32gdi::DEFAULT_CHARSET,
    w32gdi::OUT_DEFAULT_PRECIS,
    w32gdi::CLIP_DEFAULT_PRECIS,
    w32gdi::CLEARTYPE_QUALITY,
    w32gdi::FF_DONTCARE,
    name,
  );
  w32gdi::SelectObject(hdc, hfont);
}

pub fn get_hicon_from_buffer(buffer: &[u8], width: i32, height: i32) -> Option<w32wm::HICON> {
  unsafe {
    match w32wm::LookupIconIdFromDirectoryEx(
      buffer.as_ptr() as _,
      true,
      width,
      height,
      Controls::LR_DEFAULTCOLOR,
    ) as isize
    {
      0 => None,
      offset => {
        match w32wm::CreateIconFromResourceEx(
          buffer.as_ptr().offset(offset) as _,
          buffer.len() as _,
          true,
          0x00030000,
          0,
          0,
          Controls::LR_DEFAULTCOLOR,
        ) {
          hicon if hicon.is_invalid() => None,
          hicon => Some(hicon),
        }
      }
    }
  }
}
