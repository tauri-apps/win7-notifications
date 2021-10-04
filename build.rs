// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
  windows::build! {
      Windows::Win32::Foundation::{HWND, LPARAM, LRESULT, PWSTR, WPARAM, POINT},
      Windows::Win32::System::LibraryLoader::{GetModuleHandleW},
      Windows::Win32::System::Com::{CoCreateInstance, CoInitialize},
      Windows::Win32::Graphics::Gdi::*,
      Windows::Win32::UI::Shell::{ITaskbarList, TaskbarList},
      Windows::Win32::UI::Controls::*,
      Windows::Win32::UI::WindowsAndMessaging::*,
  };
}
