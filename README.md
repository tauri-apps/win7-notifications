# win7-notifications

[![](https://img.shields.io/crates/v/win7-notifications)](https://crates.io/crates/win7-notifications) [![](https://img.shields.io/docsrs/win7-notifications)](https://docs.rs/win7-notifications/) ![](https://img.shields.io/crates/l/win7-notifications)
[![Chat Server](https://img.shields.io/badge/chat-on%20discord-7289da.svg)](https://discord.gg/SpmNs4S)

Send Windows 10 styled notifications on Windows 7.

#### Note:
This crate requires a win32 event loop is running, otherwise the notification will close immediately, check [examples/simple.rs](examples/simple.rs) which uses [tao](https://github.com/tauri-apps/tao) or you can use [winit](https://github.com/rust-windowing/winit) or just roll your own win32 event loop.