//! macOS : `CGEventTap` en écoute seule. Les lettres sont lues directement dans
//! l'événement (`CGEventKeyboardGetUnicodeString`), sans API qui exige le fil principal.

use std::ffi::c_ulong;
use std::sync::mpsc::{self, Sender};
use std::thread;

use core_foundation::runloop::CFRunLoop;
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CallbackResult, EventField,
};
use foreign_types::ForeignType;

use crate::saisie::Touche;

/// `kCGEventSourceStateID` : 1 = clavier physique ; les frappes simulées (les nôtres
/// comprises) viennent d'une autre source et sont ignorées.
const CHAMP_SOURCE: u32 = 45;
const SOURCE_MATERIELLE: i64 = 1;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventKeyboardGetUnicodeString(
        event: core_graphics::sys::CGEventRef,
        max: c_ulong,
        longueur: *mut c_ulong,
        chaine: *mut u16,
    );
    fn CGPreflightListenEventAccess() -> bool;
    fn CGRequestListenEventAccess() -> bool;
}

fn caracteres(event: &CGEvent) -> Vec<char> {
    let mut tampon = [0u16; 8];
    let mut longueur: c_ulong = 0;
    unsafe {
        CGEventKeyboardGetUnicodeString(
            event.as_ptr(),
            tampon.len() as c_ulong,
            &mut longueur,
            tampon.as_mut_ptr(),
        );
    }
    char::decode_utf16(tampon[..longueur as usize].iter().copied())
        .filter_map(Result::ok)
        .collect()
}

fn convertir(type_: CGEventType, event: &CGEvent) -> Vec<Touche> {
    match type_ {
        CGEventType::LeftMouseDown | CGEventType::RightMouseDown => {
            return vec![Touche::Separateur]
        }
        CGEventType::KeyDown => {}
        _ => return vec![],
    }
    if event.get_integer_value_field(CHAMP_SOURCE) != SOURCE_MATERIELLE {
        return vec![Touche::Ignorer];
    }
    let code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
    let drapeaux = event.get_flags();
    if drapeaux.intersects(CGEventFlags::CGEventFlagCommand | CGEventFlags::CGEventFlagControl) {
        // Ctrl+1/2/3 sert à choisir une suggestion : il ne doit pas vider le mot.
        return match code {
            18..=20 => vec![Touche::Ignorer],
            _ => vec![Touche::Separateur],
        };
    }
    match code {
        51 => vec![Touche::Effacer],
        53 => vec![Touche::Echap],
        // Entrée, Tab, flèches, début/fin, page préc./suiv., suppr.
        36 | 48 | 76 | 115..=126 => vec![Touche::Separateur],
        _ => {
            let c = caracteres(event);
            if c.is_empty() {
                vec![Touche::Ignorer] // touche morte (ex. ^ avant une voyelle)
            } else {
                c.into_iter().map(Touche::Caractere).collect()
            }
        }
    }
}

pub fn ecouter(tx: Sender<Touche>) -> Result<(), String> {
    let refus = "Autorise OpenLex dans Réglages Système › Confidentialité et sécurité › \
                 Surveillance de l'entrée, puis relance OpenLex.";
    if !unsafe { CGPreflightListenEventAccess() } {
        unsafe { CGRequestListenEventAccess() };
        return Err(refus.into());
    }
    let (pret_tx, pret_rx) = mpsc::channel();
    thread::spawn(move || {
        let resultat = CGEventTap::with_enabled(
            CGEventTapLocation::Session,
            CGEventTapPlacement::TailAppendEventTap,
            CGEventTapOptions::ListenOnly,
            vec![
                CGEventType::KeyDown,
                CGEventType::LeftMouseDown,
                CGEventType::RightMouseDown,
            ],
            |_proxy, type_, event| {
                for touche in convertir(type_, event) {
                    let _ = tx.send(touche);
                }
                CallbackResult::Keep
            },
            || {
                let _ = pret_tx.send(Ok(()));
                CFRunLoop::run_current();
            },
        );
        if resultat.is_err() {
            let _ = pret_tx.send(Err(()));
        }
    });
    match pret_rx.recv() {
        Ok(Ok(())) => Ok(()),
        _ => Err(refus.into()),
    }
}
