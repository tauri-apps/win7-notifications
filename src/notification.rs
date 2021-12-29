// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use once_cell::sync::Lazy;
use std::{
  sync::Mutex,
  thread::{sleep, spawn},
  time::Duration,
};
use windows::Win32::{
  Foundation as w32f,
  Graphics::Dwm,
  Graphics::Gdi,
  System::{Diagnostics::Debug, LibraryLoader},
  UI::WindowsAndMessaging as w32wm,
};
use windows::{core::*, Win32::UI::Controls};

use crate::{
  timeout::Timeout,
  util::{self, Color},
};

/// Close button callback id
const CLOSE_BTN_ID: isize = 554;
/// notification width
const N_WIDTH: i32 = 360;
/// notification height
const N_HEIGHT: i32 = 170;
/// notification margin
const N_MARGIN: i32 = 16;
/// notification icon size (width/height)
const N_ICON_SIZE: i32 = 16;
/// notification window bg color
const WINDOW_COLOR: Color = Color(50, 57, 69);
/// used for notification summary(title)
const TITILE_COLOR: Color = Color(255, 255, 255);
/// used for notification body
const SUBTITLE_COLOR: Color = Color(175, 175, 175);
/// used to track existing notifications
static ACTIVE_NOTIFICATIONS: Lazy<Mutex<Vec<w32f::HWND>>> = Lazy::new(|| Mutex::new(Vec::new()));
/// cached primary monitor info
static PRIMARY_MONITOR: Lazy<Mutex<Gdi::MONITORINFOEXW>> =
  Lazy::new(|| Mutex::new(Gdi::MONITORINFOEXW::default()));

/// Describes The notification
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Notification {
  pub icon: Vec<u8>,
  pub appname: String,
  pub summary: String,
  pub body: String,
  pub timeout: Timeout,
}

impl Default for Notification {
  fn default() -> Notification {
    Notification {
      appname: util::current_exe_name(),
      summary: String::new(),
      body: String::new(),
      icon: Vec::new(),
      timeout: Timeout::Default,
    }
  }
}

impl Notification {
  /// Constructs a new Notification.
  ///
  /// Most fields are empty by default, only `appname` is initialized with the name of the current
  /// executable.
  pub fn new() -> Notification {
    Notification::default()
  }

  /// Overwrite the appname field used for Notification.
  pub fn appname(&mut self, appname: &str) -> &mut Notification {
    self.appname = appname.to_owned();
    self
  }

  /// Set the `summary`.
  ///
  /// Often acts as title of the notification. For more elaborate content use the `body` field.
  pub fn summary(&mut self, summary: &str) -> &mut Notification {
    self.summary = summary.to_owned();
    self
  }

  /// Set the content of the `body` field.
  ///
  /// Multiline textual content of the notification.
  /// Each line should be treated as a paragraph.
  /// html markup is not supported.
  pub fn body(&mut self, body: &str) -> &mut Notification {
    self.body = body.to_owned();
    self
  }

  /// Set the `icon` field must be `.ico` byte array.
  pub fn icon(&mut self, icon: Vec<u8>) -> &mut Notification {
    self.icon = icon;
    self
  }

  /// Set the `timeout` field.
  pub fn timeout(&mut self, timeout: Timeout) -> &mut Notification {
    self.timeout = timeout;
    self
  }

  /// Shows the Notification.
  ///
  /// Requires an win32 event_loop to be running on the thread, otherwise the notification will close immediately.
  pub fn show(&self) -> Result<()> {
    unsafe {
      let hinstance = LibraryLoader::GetModuleHandleW(w32f::PWSTR::default());

      let mut class_name = String::from("win7-notify-rust");
      let wnd_class = w32wm::WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        lpszClassName: w32f::PWSTR(class_name.as_mut_ptr() as _),
        hInstance: hinstance,
        hbrBackground: Gdi::CreateSolidBrush(WINDOW_COLOR.to_int()),
        ..Default::default()
      };
      w32wm::RegisterClassW(&wnd_class);

      // cache primary monitor info
      if let Ok(mut pm) = PRIMARY_MONITOR.lock() {
        if pm.monitorInfo.rcWork.bottom == 0 {
          *pm = util::get_monitor_info(util::primary_monitor());
        }
        let w32f::RECT { right, bottom, .. } = pm.monitorInfo.rcWork;

        let data = WindowData {
          window: w32f::HWND::default(),
          close_btn: w32f::HWND::default(),
          appname_control: w32f::HWND::default(),
          summary_control: w32f::HWND::default(),
          body_control: w32f::HWND::default(),
          _mouse_hovering_close_btn: false,
          notification: self.clone(),
        };

        let hwnd = w32wm::CreateWindowExW(
          w32wm::WS_EX_TOPMOST,
          w32f::PWSTR(class_name.as_mut_ptr() as _),
          "win7-notify-rust-window",
          w32wm::WS_SYSMENU | w32wm::WS_CAPTION | w32wm::WS_VISIBLE,
          right - N_WIDTH - 15,
          bottom - N_HEIGHT - 15,
          N_WIDTH,
          N_HEIGHT,
          w32f::HWND::default(),
          w32wm::HMENU::default(),
          hinstance,
          Box::into_raw(Box::new(data)) as _,
        );

        if hwnd == 0 {
          return Err(Error::from_win32());
        }

        // re-order active notifications and make room for new one
        if let Ok(mut active_noti) = ACTIVE_NOTIFICATIONS.lock() {
          for (i, h) in active_noti.iter().rev().enumerate() {
            w32wm::SetWindowPos(
              h,
              w32f::HWND::default(),
              right - N_WIDTH - 15,
              bottom - (N_HEIGHT * (i + 2) as i32) - 15,
              0,
              0,
              w32wm::SWP_NOOWNERZORDER | w32wm::SWP_NOSIZE | w32wm::SWP_NOZORDER,
            );
          }
          active_noti.push(hwnd);
        }

        // shadows
        if Dwm::DwmIsCompositionEnabled()?.as_bool() {
          let mut margins = Controls::MARGINS::default();
          margins.cxLeftWidth = 1;
          Dwm::DwmExtendFrameIntoClientArea(hwnd, &margins)?;
        }

        util::skip_taskbar(hwnd)?;
        w32wm::ShowWindow(hwnd, w32wm::SW_SHOW);
        Debug::MessageBeep(w32wm::MB_OK);

        let timeout = self.timeout;
        spawn(move || {
          sleep(Duration::from_millis(timeout.into()));
          if timeout != Timeout::Never {
            close_notification(hwnd);
          };
        });
      }
    }

    Ok(())
  }
}

struct WindowData {
  window: w32f::HWND,
  close_btn: w32f::HWND,
  appname_control: w32f::HWND,
  body_control: w32f::HWND,
  summary_control: w32f::HWND,
  _mouse_hovering_close_btn: bool,
  notification: Notification,
}

pub unsafe extern "system" fn window_proc(
  hwnd: w32f::HWND,
  msg: u32,
  wparam: w32f::WPARAM,
  lparam: w32f::LPARAM,
) -> w32f::LRESULT {
  let mut userdata = util::get_window_long_ptr(hwnd, w32wm::GWL_USERDATA);

  match msg {
    w32wm::WM_NCCREATE => {
      if userdata == 0 {
        let createstruct = &*(lparam as *const w32wm::CREATESTRUCTW);
        userdata = createstruct.lpCreateParams as isize;
        util::set_window_long_ptr(hwnd, w32wm::GWL_USERDATA, userdata);
      }
      w32wm::DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    // make the window borderless
    w32wm::WM_NCCALCSIZE => 0,

    w32wm::WM_CREATE => {
      let userdata = userdata as *mut WindowData;

      // fill userdata with missing info
      (*userdata).window = hwnd;

      // create the notification close button
      (*userdata).close_btn = w32wm::CreateWindowExW(
        0,
        "BUTTON",
        "",
        w32wm::WS_VISIBLE | w32wm::WS_CHILD | (w32wm::BS_OWNERDRAW | w32wm::BS_CENTER) as u32,
        N_WIDTH - N_ICON_SIZE - N_MARGIN,
        N_MARGIN,
        N_ICON_SIZE,
        N_ICON_SIZE,
        hwnd,
        CLOSE_BTN_ID as w32wm::HMENU,
        w32f::HINSTANCE::default(),
        std::ptr::null_mut(),
      );

      // create the notification appname text control
      (*userdata).appname_control = w32wm::CreateWindowExW(
        0,
        "STATIC",
        "",
        w32wm::WS_VISIBLE | w32wm::WS_CHILD | w32wm::SS_OWNERDRAW as u32,
        N_MARGIN + N_ICON_SIZE + (N_MARGIN / 4),
        N_MARGIN,
        N_WIDTH - N_MARGIN + N_ICON_SIZE + (N_MARGIN / 4) - N_ICON_SIZE - N_MARGIN,
        N_ICON_SIZE,
        hwnd,
        w32wm::HMENU::default(),
        w32f::HINSTANCE::default(),
        std::ptr::null_mut(),
      );

      // create the notification summary(title) text control
      (*userdata).summary_control = w32wm::CreateWindowExW(
        0,
        "STATIC",
        "",
        w32wm::WS_VISIBLE | w32wm::WS_CHILD | w32wm::SS_OWNERDRAW as u32,
        N_MARGIN,
        N_MARGIN + N_ICON_SIZE + (N_MARGIN / 2),
        N_WIDTH - N_MARGIN - (N_MARGIN / 2) - N_ICON_SIZE - N_MARGIN,
        N_ICON_SIZE,
        hwnd,
        w32wm::HMENU::default(),
        w32f::HINSTANCE::default(),
        std::ptr::null_mut(),
      );

      // create the notification body text control
      (*userdata).body_control = w32wm::CreateWindowExW(
        0,
        "STATIC",
        "",
        w32wm::WS_VISIBLE | w32wm::WS_CHILD | w32wm::SS_OWNERDRAW as u32,
        N_MARGIN,
        N_MARGIN + N_ICON_SIZE + (N_MARGIN / 2) + N_ICON_SIZE + (N_MARGIN / 2),
        N_WIDTH - N_MARGIN - (N_MARGIN / 2) - N_ICON_SIZE - N_MARGIN,
        N_HEIGHT - N_MARGIN - N_ICON_SIZE - (N_MARGIN / 2) - N_ICON_SIZE - (N_MARGIN / 2),
        hwnd,
        w32wm::HMENU::default(),
        w32f::HINSTANCE::default(),
        std::ptr::null_mut(),
      );
      w32wm::DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_CTLCOLORBTN => {
      let userdata = userdata as *mut WindowData;

      if lparam == (*userdata).close_btn {
        // change the close button control background color to match the window color
        return Gdi::CreateSolidBrush(WINDOW_COLOR.to_int());
      }

      w32wm::DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_DRAWITEM => {
      let userdata = userdata as *mut WindowData;
      let lpds = lparam as *mut Controls::DRAWITEMSTRUCT;

      Gdi::SetBkMode((*lpds).hDC, Gdi::TRANSPARENT);

      // draw notification close button
      if (*lpds).hwndItem == (*userdata).close_btn {
        Gdi::SetTextColor((*lpds).hDC, Color(150, 150, 150).to_int());
        Gdi::TextOutW((*lpds).hDC, 5, 1, "x", 1);
      }

      // draw notification app name
      if (*lpds).hwndItem == (*userdata).appname_control {
        util::set_font((*lpds).hDC, "Segeo UI", 15, 400);
        Gdi::SetTextColor((*lpds).hDC, TITILE_COLOR.to_int());
        Gdi::TextOutW(
          (*lpds).hDC,
          5,
          1,
          (*userdata).notification.appname.clone(),
          (*userdata).notification.appname.len() as _,
        );
      }

      // draw notification summary (title)
      if (*lpds).hwndItem == (*userdata).summary_control {
        util::set_font((*lpds).hDC, "Segeo UI", 18, 700);
        Gdi::SetTextColor((*lpds).hDC, TITILE_COLOR.to_int());
        Gdi::TextOutW(
          (*lpds).hDC,
          0,
          0,
          (*userdata).notification.summary.clone(),
          (*userdata).notification.summary.len() as _,
        );
      }

      // draw notification body
      if (*lpds).hwndItem == (*userdata).body_control {
        util::set_font((*lpds).hDC, "Segeo UI", 18, 400);
        Gdi::SetTextColor((*lpds).hDC, SUBTITLE_COLOR.to_int());
        let mut rc = w32f::RECT::default();
        w32wm::GetClientRect((*lpds).hwndItem, &mut rc);
        Gdi::DrawTextW(
          (*lpds).hDC,
          (*userdata).notification.body.clone(),
          (*userdata).notification.body.len() as _,
          &mut rc,
          Gdi::DT_LEFT | Gdi::DT_EXTERNALLEADING | Gdi::DT_WORDBREAK,
        );
      }

      true.into()
    }

    w32wm::WM_COMMAND => {
      if util::get_loword(wparam as _) == CLOSE_BTN_ID as u16 {
        let userdata = userdata as *mut WindowData;
        close_notification((*userdata).window)
      }
      w32wm::DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_PAINT => {
      let userdata = userdata as *mut WindowData;

      // draw notification icon
      if let Some(hicon) = util::get_hicon_from_buffer(&(*userdata).notification.icon, 16, 16) {
        let mut ps = Gdi::PAINTSTRUCT::default();
        let hdc = Gdi::BeginPaint(hwnd, &mut ps);
        w32wm::DrawIconEx(
          hdc,
          N_MARGIN,
          N_MARGIN,
          hicon,
          N_ICON_SIZE,
          N_ICON_SIZE,
          0,
          Gdi::HBRUSH::default(),
          w32wm::DI_NORMAL,
        );
        Gdi::EndPaint(hwnd, &ps);
      }

      w32wm::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
    _ => w32wm::DefWindowProcW(hwnd, msg, wparam, lparam),
  }
}

unsafe fn close_notification(hwnd: w32f::HWND) {
  w32wm::ShowWindow(hwnd, w32wm::SW_HIDE);
  w32wm::CloseWindow(hwnd);

  // remove notification from our cache
  if let Ok(mut active_noti) = ACTIVE_NOTIFICATIONS.lock() {
    if let Some(index) = active_noti.iter().position(|e| *e == hwnd) {
      active_noti.remove(index);
    }

    // re-order notifications
    if let Ok(pm) = PRIMARY_MONITOR.lock() {
      let w32f::RECT { right, bottom, .. } = pm.monitorInfo.rcWork;
      for (i, h) in active_noti.iter().rev().enumerate() {
        w32wm::SetWindowPos(
          h,
          w32f::HWND::default(),
          right - N_WIDTH - 15,
          bottom - (N_HEIGHT * (i + 1) as i32) - 15,
          0,
          0,
          w32wm::SWP_NOOWNERZORDER | w32wm::SWP_NOSIZE | w32wm::SWP_NOZORDER,
        );
      }
    }
  }
}
