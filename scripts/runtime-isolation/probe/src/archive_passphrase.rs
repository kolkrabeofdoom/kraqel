//! Owned Win32 passphrase prompts for an explicitly requested portable supplement.
use std::{io, ptr};
use windows_sys::Win32::{
    Foundation::{ERROR_CANCELLED, ERROR_SUCCESS, HWND},
    Security::Credentials::{
        CREDUI_FLAGS_ALWAYS_SHOW_UI, CREDUI_FLAGS_DO_NOT_PERSIST, CREDUI_FLAGS_GENERIC_CREDENTIALS,
        CREDUI_FLAGS_KEEP_USERNAME, CREDUI_INFOW, CredUIPromptForCredentialsW,
    },
    UI::WindowsAndMessaging::GetWindowThreadProcessId,
};
use zeroize::Zeroizing;

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

fn prompt(owner: HWND, message: &str) -> io::Result<Option<Zeroizing<String>>> {
    let mut pid = 0;
    // The HWND comes from the native desktop window, never from renderer IPC.
    if owner.is_null()
        || unsafe { GetWindowThreadProcessId(owner, &mut pid) } == 0
        || pid != std::process::id()
    {
        return Err(io::Error::other("Das zugehörige KRAQEL-Fenster fehlt."));
    }
    let caption = wide("KRAQEL · Portables Zusatzarchiv");
    let message = wide(message);
    let target = wide(&format!("KRAQEL-Archiv-{}", uuid::Uuid::new_v4()));
    let info = CREDUI_INFOW {
        cbSize: size_of::<CREDUI_INFOW>() as u32,
        hwndParent: owner,
        pszCaptionText: caption.as_ptr(),
        pszMessageText: message.as_ptr(),
        hbmBanner: ptr::null_mut(),
    };
    let mut user = [0u16; 514];
    let label = wide("KRAQEL-Archiv");
    user[..label.len()].copy_from_slice(&label);
    let mut secret = Zeroizing::new([0u16; 257]);
    let mut save = 0;
    // All strings and writable buffers live until this modal call returns. A unique
    // target and DO_NOT_PERSIST prevent reuse or registration as a system credential.
    let status = unsafe {
        CredUIPromptForCredentialsW(
            &info,
            target.as_ptr(),
            ptr::null(),
            0,
            user.as_mut_ptr(),
            user.len() as u32,
            secret.as_mut_ptr(),
            secret.len() as u32,
            &mut save,
            CREDUI_FLAGS_GENERIC_CREDENTIALS
                | CREDUI_FLAGS_ALWAYS_SHOW_UI
                | CREDUI_FLAGS_KEEP_USERNAME
                | CREDUI_FLAGS_DO_NOT_PERSIST,
        )
    };
    match status {
        ERROR_CANCELLED => Ok(None),
        ERROR_SUCCESS => {
            let length = secret
                .iter()
                .position(|&s| s == 0)
                .filter(|&len| len > 0 && len < secret.len() - 1)
                .ok_or_else(|| {
                    io::Error::other("Eine Passphrase mit bis zu 255 Zeichen eingeben.")
                })?;
            String::from_utf16(&secret[..length])
                .map(Zeroizing::new)
                .map(Some)
                .map_err(|_| io::Error::other("Ungültige Passphrase."))
        }
        _ => Err(io::Error::from_raw_os_error(status as i32)),
    }
}

/// Reads the archive secret twice from native password fields. Cancellation at either
/// prompt drops and wipes existing application-owned input buffers.
pub fn prompt_archive_passphrase(owner: isize) -> io::Result<Option<Zeroizing<String>>> {
    let Some(first) = prompt(
        owner as HWND,
        "Legen Sie eine eigene Passphrase für das Zusatzarchiv fest. Sie benötigen sie zur Wiederherstellung auf jedem Rechner.",
    )?
    else {
        return Ok(None);
    };
    let Some(second) = prompt(
        owner as HWND,
        "Geben Sie die Archiv-Passphrase zur Bestätigung erneut ein.",
    )?
    else {
        return Ok(None);
    };
    if *first != *second {
        return Err(io::Error::other(
            "Die Archiv-Passphrasen stimmen nicht überein.",
        ));
    }
    Ok(Some(first))
}
