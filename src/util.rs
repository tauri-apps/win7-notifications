// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use windows::{
  core::*,
  Win32::{
    Foundation as w32f,
    Graphics::Gdi,
    System::Com,
    UI::{Shell, WindowsAndMessaging as w32wm},
  },
};
pub fn current_exe_name() -> String {
  std::env::current_exe()
    .unwrap()
    .file_name()
    .unwrap()
    .to_str()
    .unwrap()
    .to_owned()
}

/// Implementation of the `RGB` macro.
#[allow(non_snake_case)]
#[inline]
pub const fn RGB(r: u32, g: u32, b: u32) -> u32 {
  r | g << 8 | b << 16
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
pub fn GetWindowLongPtrW(window: w32f::HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
  unsafe { w32wm::GetWindowLongW(window, index) as _ }
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
pub fn GetWindowLongPtrW(window: w32f::HWND, index: w32wm::WINDOW_LONG_PTR_INDEX) -> isize {
  unsafe { w32wm::GetWindowLongPtrW(window, index) }
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
pub fn SetWindowLongPtrW(
  window: w32f::HWND,
  index: w32wm::WINDOW_LONG_PTR_INDEX,
  value: isize,
) -> isize {
  unsafe { w32wm::SetWindowLongW(window, index, value as _) as _ }
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
pub fn SetWindowLongPtrW(
  window: w32f::HWND,
  index: w32wm::WINDOW_LONG_PTR_INDEX,
  value: isize,
) -> isize {
  unsafe { w32wm::SetWindowLongPtrW(window, index, value) }
}

/// Implementation of the `GET_X_LPARAM` macro.
#[allow(non_snake_case)]
#[inline]
pub fn GET_X_LPARAM(lparam: w32f::LPARAM) -> i16 {
  ((lparam.0 as usize) & 0xFFFF) as u16 as i16
}

/// Implementation of the `GET_Y_LPARAM` macro.
#[allow(non_snake_case)]
#[inline]
pub fn GET_Y_LPARAM(lparam: w32f::LPARAM) -> i16 {
  (((lparam.0 as usize) & 0xFFFF_0000) >> 16) as u16 as i16
}

pub unsafe fn primary_monitor() -> Gdi::HMONITOR {
  let pt = w32f::POINT { x: 0, y: 0 };
  Gdi::MonitorFromPoint(&pt, Gdi::MONITOR_DEFAULTTOPRIMARY)
}

pub unsafe fn get_monitor_info(hmonitor: Gdi::HMONITOR) -> Gdi::MONITORINFOEXW {
  let mut monitor_info = Gdi::MONITORINFOEXW::default();
  monitor_info.monitorInfo.cbSize = std::mem::size_of::<Gdi::MONITORINFOEXW>() as u32;
  Gdi::GetMonitorInfoW(
    hmonitor,
    &mut monitor_info as *mut Gdi::MONITORINFOEXW as *mut Gdi::MONITORINFO,
  );
  monitor_info
}

pub unsafe fn skip_taskbar(hwnd: w32f::HWND) -> Result<()> {
  let _ = Com::CoInitialize(std::ptr::null());
  let taskbar_list: Shell::ITaskbarList =
    Com::CoCreateInstance(&Shell::TaskbarList, None, Com::CLSCTX_SERVER)?;
  taskbar_list.DeleteTab(hwnd)?;
  Ok(())
}

pub unsafe fn set_font(hdc: Gdi::HDC, name: &str, size: i32, weight: i32) {
  let hfont = Gdi::CreateFontW(
    size,
    0,
    0,
    0,
    weight,
    false.into(),
    false.into(),
    false.into(),
    Gdi::DEFAULT_CHARSET,
    Gdi::OUT_DEFAULT_PRECIS,
    Gdi::CLIP_DEFAULT_PRECIS,
    Gdi::CLEARTYPE_QUALITY,
    Gdi::FF_DONTCARE,
    name,
  );
  Gdi::SelectObject(hdc, hfont);
}

#[allow(dead_code)]
pub(crate) struct Pixel {
  pub(crate) r: u8,
  pub(crate) g: u8,
  pub(crate) b: u8,
  pub(crate) a: u8,
}

impl Pixel {
  fn to_bgra(&mut self) {
    std::mem::swap(&mut self.r, &mut self.b);
  }
}

pub(crate) const PIXEL_SIZE: usize = std::mem::size_of::<Pixel>();

pub fn get_hicon_from_buffer(rgba: Vec<u8>, width: u32, height: u32) -> w32wm::HICON {
  let mut rgba = rgba;
  let pixel_count = rgba.len() / PIXEL_SIZE;
  let mut and_mask = Vec::with_capacity(pixel_count);
  let pixels =
    unsafe { std::slice::from_raw_parts_mut(rgba.as_mut_ptr() as *mut Pixel, pixel_count) };
  for pixel in pixels {
    and_mask.push(pixel.a.wrapping_sub(std::u8::MAX)); // invert alpha channel
    pixel.to_bgra();
  }
  assert_eq!(and_mask.len(), pixel_count);
  unsafe {
    w32wm::CreateIcon(
      w32f::HINSTANCE::default(),
      width as i32,
      height as i32,
      1,
      (4 * 8) as u8,
      and_mask.as_ptr() as *const u8,
      rgba.as_ptr() as *const u8,
    ) as w32wm::HICON
  }
}
