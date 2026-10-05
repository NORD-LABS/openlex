//! Windows : hooks bas niveau clavier et souris sur un fil dédié avec sa propre boucle de
//! messages. Les frappes injectées (les nôtres comprises) sont ignorées.

use std::sync::mpsc::{self, Sender};
use std::sync::{Mutex, OnceLock};
use std::thread;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, GetKeyboardLayout, ToUnicodeEx, VIRTUAL_KEY, VK_BACK,
    VK_CAPITAL, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_LEFT, VK_LWIN,
    VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_RWIN, VK_SHIFT, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId,
    SetWindowsHookExW, TranslateMessage, HHOOK, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG,
    WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_LBUTTONDOWN, WM_RBUTTONDOWN, WM_SYSKEYDOWN,
};

use crate::saisie::Touche;

static ENVOI: OnceLock<Mutex<Sender<Touche>>> = OnceLock::new();

fn envoyer(touches: &[Touche]) {
    if let Some(tx) = ENVOI.get() {
        if let Ok(tx) = tx.lock() {
            for &t in touches {
                let _ = tx.send(t);
            }
        }
    }
}

fn appuyee(vk: VIRTUAL_KEY) -> bool {
    (unsafe { GetAsyncKeyState(vk.0 as i32) }) < 0
}

fn caracteres(vk: u32, scan: u32) -> Vec<char> {
    let mut etat = [0u8; 256];
    if appuyee(VK_SHIFT) {
        etat[VK_SHIFT.0 as usize] = 0x80;
    }
    if (unsafe { GetKeyState(VK_CAPITAL.0 as i32) }) & 1 != 0 {
        etat[VK_CAPITAL.0 as usize] = 0x01;
    }
    // AltGr = Ctrl + Alt (caractères comme « @ » sur un clavier canadien français)
    if appuyee(VK_CONTROL) && appuyee(VK_MENU) {
        etat[VK_CONTROL.0 as usize] = 0x80;
        etat[VK_MENU.0 as usize] = 0x80;
    }
    let mut tampon = [0u16; 8];
    let disposition = unsafe {
        let fil = GetWindowThreadProcessId(GetForegroundWindow(), None);
        GetKeyboardLayout(fil)
    };
    // Drapeau 0x4 : ne pas modifier l'état du clavier (préserve les touches mortes).
    let n = unsafe { ToUnicodeEx(vk, scan, &etat, &mut tampon, 0x4, Some(disposition)) };
    if n <= 0 {
        return Vec::new();
    }
    char::decode_utf16(tampon[..n as usize].iter().copied())
        .filter_map(Result::ok)
        .collect()
}

fn convertir(info: &KBDLLHOOKSTRUCT) -> Vec<Touche> {
    if info.flags.0 & LLKHF_INJECTED.0 != 0 {
        return vec![Touche::Ignorer];
    }
    let vk = VIRTUAL_KEY(info.vkCode as u16);
    let altgr = appuyee(VK_CONTROL) && appuyee(VK_MENU);
    if (appuyee(VK_CONTROL) || appuyee(VK_LWIN) || appuyee(VK_RWIN)) && !altgr {
        // Ctrl+1/2/3 sert à choisir une suggestion : il ne doit pas vider le mot.
        return match info.vkCode {
            0x31..=0x33 => vec![Touche::Ignorer],
            _ if is_modificateur(vk) => vec![Touche::Ignorer],
            _ => vec![Touche::Separateur],
        };
    }
    match vk {
        VK_BACK => vec![Touche::Effacer],
        VK_ESCAPE => vec![Touche::Echap],
        VK_RETURN | VK_TAB | VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_HOME | VK_END | VK_PRIOR
        | VK_NEXT | VK_DELETE => vec![Touche::Separateur],
        _ if is_modificateur(vk) => vec![Touche::Ignorer],
        _ => {
            let c = caracteres(info.vkCode, info.scanCode);
            if c.is_empty() {
                vec![Touche::Ignorer]
            } else {
                c.into_iter().map(Touche::Caractere).collect()
            }
        }
    }
}

fn is_modificateur(vk: VIRTUAL_KEY) -> bool {
    // Maj, Ctrl, Alt (gauche/droite incluses), Windows, Verr. maj.
    matches!(vk.0, 0x10..=0x12 | 0x14 | 0x5B | 0x5C | 0xA0..=0xA5)
}

unsafe extern "system" fn hook_clavier(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && (wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN) {
        let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        envoyer(&convertir(info));
    }
    CallNextHookEx(None, code, wparam, lparam)
}

unsafe extern "system" fn hook_souris(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && (wparam.0 as u32 == WM_LBUTTONDOWN || wparam.0 as u32 == WM_RBUTTONDOWN) {
        envoyer(&[Touche::Separateur]);
    }
    CallNextHookEx(None, code, wparam, lparam)
}

pub fn ecouter(tx: Sender<Touche>) -> Result<(), String> {
    if ENVOI.set(Mutex::new(tx)).is_err() {
        return Ok(()); // déjà à l'écoute
    }
    let (pret_tx, pret_rx) = mpsc::channel();
    thread::spawn(move || unsafe {
        let clavier: Result<HHOOK, _> =
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_clavier), None, 0);
        let souris: Result<HHOOK, _> = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_souris), None, 0);
        let _ = pret_tx.send(clavier.is_ok() && souris.is_ok());
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
    match pret_rx.recv() {
        Ok(true) => Ok(()),
        _ => Err("Impossible d'écouter le clavier sur cet ordinateur.".into()),
    }
}
