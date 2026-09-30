//! A Windows-owned text surface inside the game window, not a speech recognizer.
//! All HWND operations run on the main thread through a NonSend resource.
use super::*;
use bevy::window::RawHandleWrapper;
use raw_window_handle::RawWindowHandle;
use std::ptr::null_mut;
use windows_sys::Win32::{
    Foundation::{FreeLibrary, HMODULE, HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::{GetStockObject, DEFAULT_GUI_FONT},
    System::LibraryLoader::{LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32},
    UI::{
        Controls::{EM_GETMODIFY, EM_SETMODIFY, EM_SETSEL},
        Input::KeyboardAndMouse::{GetFocus, SetFocus, VK_ESCAPE, VK_F10, VK_F8},
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::*,
    },
};

// Rich Edit messages from richedit.h. windows-sys does not expose these constants.
const EM_EXLIMITTEXT: u32 = WM_USER + 53;
const EM_SETTEXTMODE: u32 = WM_USER + 89;
const EM_SETEDITSTYLE: u32 = WM_USER + 204;
const SES_USECTF: usize = 0x00010000;
const TM_PLAINTEXT: usize = 1;
const SUBCLASS: usize = 0x4642;

#[derive(Default)]
struct Editor {
    window: HWND,
    parent: HWND,
    library: HMODULE,
    visible: bool,
    failed: bool,
    mirrored: String,
}
impl Drop for Editor {
    fn drop(&mut self) {
        // SAFETY: NonSend resource is owned by the window/main thread. The child is
        // destroyed before unloading its class DLL; parent destruction may precede us.
        unsafe {
            if !self.window.is_null() && IsWindow(self.window) != 0 {
                RemoveWindowSubclass(self.window, Some(editor_proc), SUBCLASS);
                DestroyWindow(self.window);
            }
            if !self.library.is_null() {
                FreeLibrary(self.library);
            }
        }
    }
}
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

unsafe extern "system" fn editor_proc(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
    _: usize,
    _: usize,
) -> LRESULT {
    // SAFETY: Installed only on our live same-thread child. No Rust references or
    // allocations cross the callback, and all unhandled messages keep native editing.
    if (msg == WM_KEYDOWN || msg == WM_KEYUP)
        && [VK_ESCAPE as usize, VK_F8 as usize, VK_F10 as usize].contains(&w)
    {
        PostMessageW(GetParent(hwnd), msg, w, l);
        return 0;
    }
    if msg == WM_CHAR && w == VK_ESCAPE as usize {
        return 0;
    }
    if msg == WM_NCDESTROY {
        RemoveWindowSubclass(hwnd, Some(editor_proc), SUBCLASS);
    }
    DefSubclassProc(hwnd, msg, w, l)
}

pub(super) fn install(app: &mut App) {
    app.init_non_send_resource::<Editor>()
        .add_systems(Update, read_text.before(typing))
        .add_systems(
            PostUpdate,
            place
                .after(bevy::ui::UiSystem::PostLayout)
                .after(bevy::transform::TransformSystem::TransformPropagate),
        );
}
fn read_text(mut editor: NonSendMut<Editor>, mut f: ResMut<Frontend>) {
    if f.page != Page::Feedback || !editor.visible || editor.window.is_null() {
        return;
    }
    // SAFETY: This system is main-thread-only. Buffer includes the terminator and
    // the API receives its actual capacity. The control remains owned by Editor.
    unsafe {
        if IsWindow(editor.window) == 0 || SendMessageW(editor.window, EM_GETMODIFY, 0, 0) == 0 {
            return;
        }
        let len = GetWindowTextLengthW(editor.window).max(0) as usize;
        let mut text = vec![0u16; len + 1];
        let copied =
            GetWindowTextW(editor.window, text.as_mut_ptr(), text.len() as i32).max(0) as usize;
        let text = String::from_utf16_lossy(&text[..copied])
            .replace("\r\n", "\n")
            .replace('\r', "\n");
        f.feedback_text = text.clone();
        editor.mirrored = text;
        SendMessageW(editor.window, EM_SETMODIFY, 0, 0);
    }
    if let Err(error) = feedback::persist(&f) {
        f.message = format!("Draft retained in memory; save failed: {error}");
    }
    f.dirty = true;
}
fn place(
    mut editor: NonSendMut<Editor>,
    mut f: ResMut<Frontend>,
    windows: Query<(&Window, &RawHandleWrapper), With<PrimaryWindow>>,
    slots: Query<(&ComputedNode, &GlobalTransform), With<feedback::EditorSlot>>,
) {
    // SAFETY: Both systems accessing Editor are NonSend, hence same-thread. All
    // pointers below are live HWNDs or bounded NUL-terminated local UTF-16 buffers.
    unsafe {
        if f.page != Page::Feedback {
            if editor.visible && !editor.window.is_null() {
                let focused = GetFocus() == editor.window;
                ShowWindow(editor.window, SW_HIDE);
                if focused {
                    SetFocus(editor.parent);
                }
            }
            editor.visible = false;
            return;
        }
        if editor.failed {
            return;
        }
        let Ok((window, raw)) = windows.single() else {
            return;
        };
        let Ok((node, transform)) = slots.single() else {
            return;
        };
        let RawWindowHandle::Win32(handle) = raw.get_window_handle() else {
            return;
        };
        let parent = handle.hwnd.get() as HWND;
        if editor.window.is_null() {
            editor.library = LoadLibraryExW(
                wide("msftedit.dll").as_ptr(),
                null_mut(),
                LOAD_LIBRARY_SEARCH_SYSTEM32,
            );
            if !editor.library.is_null() {
                editor.window = CreateWindowExW(
                    WS_EX_CLIENTEDGE,
                    wide("RICHEDIT50W").as_ptr(),
                    wide("").as_ptr(),
                    WS_CHILD
                        | WS_VSCROLL
                        | WS_TABSTOP
                        | ES_MULTILINE as u32
                        | ES_AUTOVSCROLL as u32
                        | ES_WANTRETURN as u32,
                    0,
                    0,
                    1,
                    1,
                    parent,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                );
            }
            if editor.window.is_null()
                || SetWindowSubclass(editor.window, Some(editor_proc), SUBCLASS, 0) == 0
            {
                if !editor.window.is_null() {
                    DestroyWindow(editor.window);
                    editor.window = null_mut();
                }
                editor.failed = true;
                f.message = "Windows text control unavailable. Basic typing remains available; Windows dictation is not enabled in this fallback.".into();
                f.dirty = true;
                return;
            }
            editor.parent = parent;
            SendMessageW(editor.window, EM_SETTEXTMODE, TM_PLAINTEXT, 0);
            // Rich Edit requires explicit TSF opt-in for Windows text services.
            // Windows owns dictation and any listening UI, not the game.
            SendMessageW(
                editor.window,
                EM_SETEDITSTYLE,
                SES_USECTF,
                SES_USECTF as isize,
            );
            SendMessageW(
                editor.window,
                WM_SETFONT,
                GetStockObject(DEFAULT_GUI_FONT) as usize,
                1,
            );
            f.feedback_native = true;
            f.dirty = true;
        }
        if !editor.visible || f.feedback_text != editor.mirrored {
            // Preserve over-limit migrated drafts rather than truncating them.
            let limit = crate::compiled_frontend_config()
                .feedback_limit
                .max(f.feedback_text.encode_utf16().count());
            SendMessageW(editor.window, EM_EXLIMITTEXT, 0, limit as isize);
            SetWindowTextW(
                editor.window,
                wide(&f.feedback_text.replace('\n', "\r\n")).as_ptr(),
            );
            SendMessageW(editor.window, EM_SETSEL, usize::MAX, -1);
            SendMessageW(editor.window, EM_SETMODIFY, 0, 0);
            editor.mirrored = f.feedback_text.clone();
        }
        let size = node.size();
        let origin = transform.translation().truncate() - size * 0.5;
        let x = origin.x.max(0.).round() as i32;
        let y = origin.y.max(0.).round() as i32;
        let width = size
            .x
            .min(window.physical_width() as f32 - x as f32)
            .max(1.)
            .round() as i32;
        let height = size
            .y
            .min(window.physical_height() as f32 - y as f32)
            .max(1.)
            .round() as i32;
        SetWindowPos(editor.window, HWND_TOP, x, y, width, height, SWP_NOACTIVATE);
        ShowWindow(editor.window, SW_SHOWNA);
        if (!editor.visible || f.feedback_focus) && GetForegroundWindow() == parent {
            SetFocus(editor.window);
            f.feedback_focus = false;
        }
        editor.visible = true;
    }
}
