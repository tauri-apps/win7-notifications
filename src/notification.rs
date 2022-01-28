// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use once_cell::sync::Lazy;
use std::{
  sync::Mutex,
  thread::{sleep, spawn},
  time::Duration,
};
use windows::{
  core::*,
  Win32::{
    Foundation::{BOOL, HINSTANCE, HWND, LPARAM, LRESULT, PWSTR, RECT, WPARAM},
    Graphics::{
      Dwm::{DwmExtendFrameIntoClientArea, DwmIsCompositionEnabled},
      Gdi::{
        BeginPaint, CreateSolidBrush, DrawTextW, EndPaint, InvalidateRect, SetBkColor,
        SetTextColor, TextOutW, DT_EXTERNALLEADING, DT_LEFT, DT_WORDBREAK, HBRUSH, MONITORINFOEXW,
        PAINTSTRUCT,
      },
    },
    System::{Diagnostics::Debug::MessageBeep, LibraryLoader::GetModuleHandleW},
    UI::{
      Controls::MARGINS,
      WindowsAndMessaging::{
        self as w32wm, CloseWindow, CreateWindowExW, DefWindowProcW, DrawIconEx, LoadCursorW,
        RegisterClassW, SetCursor, SetWindowPos, ShowWindow, CREATESTRUCTW, DI_NORMAL,
        GWL_USERDATA, HMENU, IDC_ARROW, IDC_HAND, MB_OK, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
        SW_HIDE, SW_SHOW, WNDCLASSW, WS_CAPTION, WS_EX_TOPMOST, WS_SYSMENU, WS_VISIBLE,
      },
    },
  },
};

use crate::{
  timeout::Timeout,
  util::{self, Color},
};

/// notification width
const NW: i32 = 360;
/// notification height
const NH: i32 = 170;
/// notification margin
const NM: i32 = 16;
/// notification icon size (width/height)
const NIS: i32 = 16;
/// notification window bg color
const WC: Color = Color(50, 57, 69);
/// used for notification summary (title)
const TC: Color = Color(255, 255, 255);
/// used for notification body
const SC: Color = Color(200, 200, 200);

static ACTIVE_NOTIFICATIONS: Lazy<Mutex<Vec<HWND>>> = Lazy::new(|| Mutex::new(Vec::new()));
static PRIMARY_MONITOR: Lazy<Mutex<MONITORINFOEXW>> =
  Lazy::new(|| unsafe { Mutex::new(util::get_monitor_info(util::primary_monitor())) });

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
  /// Requires a win32 event_loop to be running on the thread, otherwise the notification will close immediately.
  pub fn show(&self) -> Result<()> {
    unsafe {
      let hinstance = GetModuleHandleW(PWSTR::default());

      let mut class_name = String::from("win7-notify-rust");
      let wnd_class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        lpszClassName: PWSTR(class_name.as_mut_ptr() as _),
        hInstance: hinstance,
        hbrBackground: CreateSolidBrush(WC.to_int()),
        ..Default::default()
      };
      RegisterClassW(&wnd_class);

      if let Ok(pm) = PRIMARY_MONITOR.lock() {
        let RECT { right, bottom, .. } = pm.monitorInfo.rcWork;

        let data = WindowData {
          window: HWND::default(),
          mouse_hovering_close_btn: false,
          notification: self.clone(),
        };

        let hwnd = CreateWindowExW(
          WS_EX_TOPMOST,
          PWSTR(class_name.as_mut_ptr() as _),
          "win7-notifications-window",
          WS_SYSMENU | WS_CAPTION | WS_VISIBLE,
          right - NW - 15,
          bottom - NH - 15,
          NW,
          NH,
          HWND::default(),
          HMENU::default(),
          hinstance,
          Box::into_raw(Box::new(data)) as _,
        );

        if hwnd.is_invalid() {
          return Err(Error::from_win32());
        }

        // re-order active notifications and make room for new one
        if let Ok(mut active_notifications) = ACTIVE_NOTIFICATIONS.lock() {
          active_notifications.push(hwnd);
          let mut i = active_notifications.len() as i32;
          for hwnd in active_notifications.iter() {
            SetWindowPos(
              hwnd,
              HWND::default(),
              right - NW - 15,
              bottom - 15 - (NH * i) - 10 * (i - 1),
              0,
              0,
              SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOZORDER,
            );
            i -= 1;
          }
        }

        // shadows
        if DwmIsCompositionEnabled()?.as_bool() {
          let margins = MARGINS {
            cxLeftWidth: 1,
            ..Default::default()
          };
          DwmExtendFrameIntoClientArea(hwnd, &margins)?;
        }

        util::skip_taskbar(hwnd)?;
        ShowWindow(hwnd, SW_SHOW);
        MessageBeep(MB_OK);

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

unsafe fn close_notification(hwnd: HWND) {
  ShowWindow(hwnd, SW_HIDE);
  CloseWindow(hwnd);

  if let Ok(mut active_noti) = ACTIVE_NOTIFICATIONS.lock() {
    if let Some(index) = active_noti.iter().position(|e| *e == hwnd) {
      active_noti.remove(index);
    }

    // re-order notifications
    if let Ok(pm) = PRIMARY_MONITOR.lock() {
      let RECT { right, bottom, .. } = pm.monitorInfo.rcWork;
      for (i, h) in active_noti.iter().rev().enumerate() {
        SetWindowPos(
          h,
          HWND::default(),
          right - NW - 15,
          bottom - (NH * (i + 1) as i32) - 15,
          0,
          0,
          SWP_NOSIZE | SWP_NOZORDER,
        );
      }
    }
  }
}

struct WindowData {
  window: HWND,
  notification: Notification,
  mouse_hovering_close_btn: bool,
}

pub unsafe extern "system" fn window_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  let mut userdata = util::GetWindowLongPtrW(hwnd, GWL_USERDATA);

  match msg {
    w32wm::WM_NCCREATE => {
      if userdata == 0 {
        let createstruct = &*(lparam.0 as *const CREATESTRUCTW);
        userdata = createstruct.lpCreateParams as isize;
        util::SetWindowLongPtrW(hwnd, GWL_USERDATA, userdata);
      }
      DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    // make the window borderless
    w32wm::WM_NCCALCSIZE => LRESULT(0),

    w32wm::WM_CREATE => {
      let userdata = userdata as *mut WindowData;
      (*userdata).window = hwnd;
      DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_PAINT => {
      let userdata = userdata as *mut WindowData;
      let mut ps = PAINTSTRUCT::default();
      let hdc = BeginPaint(hwnd, &mut ps);

      SetBkColor(hdc, WC.to_int());
      SetTextColor(hdc, TC.to_int());

      // draw notification icon
      if let Some(hicon) = util::get_hicon_from_buffer(&(*userdata).notification.icon, 16, 16) {
        DrawIconEx(
          hdc,
          NM,
          NM,
          hicon,
          NIS,
          NIS,
          0,
          HBRUSH::default(),
          DI_NORMAL,
        );
      }

      // draw notification close button
      SetTextColor(
        hdc,
        if (*userdata).mouse_hovering_close_btn {
          TC.to_int()
        } else {
          SC.to_int()
        },
      );
      TextOutW(hdc, NW - NM - NM / 2, NM, "X", 1);

      // draw notification app name
      SetTextColor(hdc, TC.to_int());
      util::set_font(hdc, "Segeo UI", 15, 400);
      TextOutW(
        hdc,
        NM + NIS + (NM / 2),
        NM,
        (*userdata).notification.appname.clone(),
        (*userdata).notification.appname.len() as _,
      );

      // draw notification summary (title)
      util::set_font(hdc, "Segeo UI", 17, 700);
      TextOutW(
        hdc,
        NM,
        NM + NIS + (NM / 2),
        (*userdata).notification.summary.clone(),
        (*userdata).notification.summary.len() as _,
      );

      // draw notification body
      SetTextColor(hdc, SC.to_int());
      util::set_font(hdc, "Segeo UI", 17, 400);
      let mut rc = RECT {
        left: NM,
        top: NM + NIS + (NM / 2) + 17 + (NM / 2),
        right: NW - NM,
        bottom: NH - NM,
      };
      DrawTextW(
        hdc,
        (*userdata).notification.body.clone(),
        (*userdata).notification.body.len() as _,
        &mut rc,
        DT_LEFT | DT_EXTERNALLEADING | DT_WORDBREAK,
      );

      EndPaint(HWND(hdc.0), &ps);
      DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_MOUSEMOVE => {
      let userdata = userdata as *mut WindowData;

      let (x, y) = (util::GET_X_LPARAM(lparam), util::GET_Y_LPARAM(lparam));
      let hit = close_button_hit_test(x, y);
      SetCursor(LoadCursorW(
        HINSTANCE::default(),
        if hit { IDC_HAND } else { IDC_ARROW },
      ));
      if hit != (*userdata).mouse_hovering_close_btn {
        // only trigger redraw if the previous state is different than the new state
        InvalidateRect(hwnd, std::ptr::null(), BOOL(0));
      }
      (*userdata).mouse_hovering_close_btn = hit;

      DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_LBUTTONDOWN => {
      let (x, y) = (util::GET_X_LPARAM(lparam), util::GET_Y_LPARAM(lparam));
      if close_button_hit_test(x, y) {
        close_notification(hwnd)
      }

      DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_DESTROY => {
      let userdata = userdata as *mut WindowData;
      Box::from_raw(userdata);

      DefWindowProcW(hwnd, msg, wparam, lparam)
    }
    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
  }
}

fn close_button_hit_test(x: i16, y: i16) -> bool {
  (x > (NW - NM - NM) as i16)
    && (x < (NW - NM / 2) as i16)
    && (y > NM as i16)
    && (y < (NM * 2) as i16)
}
