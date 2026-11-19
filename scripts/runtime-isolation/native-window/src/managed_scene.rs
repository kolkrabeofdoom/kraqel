//! One A/read fixture exchange; no paths, mode or scene identifiers from the UI.
use crate::{bootstrap::Role, check, policy, require, wide, window_receipt};
use kraqel_native_scene_model::channel::{Binding, HANDSHAKE_BYTES, Handshake};
use std::{
    fs::File,
    io::{self, Read, Write},
    os::windows::io::FromRawHandle,
    ptr::null_mut,
};
use zeroize::Zeroizing;
impl window_receipt::SceneOperation {
    pub fn from_argument(arg: Option<&std::ffi::OsStr>) -> Option<Self> {
        [Self::Once, Self::CheckDeniedSave, Self::CheckEditConflict]
            .into_iter()
            .find(|mode| arg == Some(std::ffi::OsStr::new(mode.argument())))
    }
}

fn acknowledge(
    input: &mut impl std::io::Read,
    output: &mut impl std::io::Write,
    open: bool,
) -> std::io::Result<()> {
    let mut ticket = [0; 32];
    input.read_exact(&mut ticket)?;
    output.write_all(&window_receipt::receipt(&ticket))?;
    if open {
        let mut grant = [0; 32];
        input.read_exact(&mut grant)?;
        require(window_receipt::commitment(&grant) == ticket, "UI grant")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acknowledged_ticket_rejects_another_opening_grant() {
        let ticket = window_receipt::commitment(&[1; 32]);
        let input = [ticket, [2; 32]].concat();
        let mut output = Vec::new();
        assert!(acknowledge(&mut input.as_slice(), &mut output, true).is_err());
        assert_eq!(output, window_receipt::receipt(&ticket));
    }
}

pub fn confirm_bootstrap(profile: Option<&str>, role: Role) -> std::io::Result<(File, File)> {
    use windows_sys::Win32::Storage::FileSystem::{FILE_TYPE_PIPE, GetFileType};
    use windows_sys::Win32::System::Threading::{GetStartupInfoW, STARTUPINFOW};

    let mut startup = STARTUPINFOW::default();
    unsafe { GetStartupInfoW(&mut startup) };
    if startup.hStdInput == startup.hStdOutput
        || [startup.hStdInput, startup.hStdOutput]
            .iter()
            .any(|h| unsafe { GetFileType(*h) } != FILE_TYPE_PIPE)
    {
        return Err(std::io::Error::other("invalid bootstrap endpoints"));
    }
    // Borrow the standard handles; process teardown owns their lifetime.
    use std::os::windows::io::BorrowedHandle;
    let input = unsafe { BorrowedHandle::borrow_raw(startup.hStdInput) };
    let output = unsafe { BorrowedHandle::borrow_raw(startup.hStdOutput) };
    let mut input = std::fs::File::from(input.try_clone_to_owned()?);
    let mut output = std::fs::File::from(output.try_clone_to_owned()?);
    if let Some(profile) = profile {
        let suffix = profile
            .strip_prefix(match role {
                Role::Window => "org.kraqel.probe.ui.",
                Role::Data => "org.kraqel.probe.data.",
            })
            .unwrap_or("");
        require(
            suffix.len() == 32 && suffix.bytes().all(|b| b.is_ascii_hexdigit()),
            "UI profile",
        )?;
        unsafe {
            use windows_sys::Win32::Security::{
                FreeSid, Isolation::DeriveAppContainerSidFromAppContainerName,
            };
            use windows_sys::Win32::System::{
                JobObjects::IsProcessInJob, Threading::GetCurrentProcess,
            };
            let mut sid = null_mut();
            require(
                DeriveAppContainerSidFromAppContainerName(wide(profile).as_ptr(), &mut sid) >= 0,
                "UI profile SID",
            )?;
            let process = GetCurrentProcess();
            let result = policy(process, sid);
            FreeSid(sid);
            result?;
            let mut member = 0;
            check(IsProcessInJob(process, null_mut(), &mut member))?;
            require(member != 0, "UI job")?;
        }
    }
    acknowledge(&mut input, &mut output, profile.is_some())?;
    Ok((input, output))
}

struct Channel {
    binding: Binding,
    reader: File,
    writer: File,
}
impl Channel {
    fn open(mut control: File) -> io::Result<Self> {
        let mut keys = Zeroizing::new([0u8; 96]);
        control.read_exact(keys.as_mut())?;
        let binding = Binding::from_keys(
            keys[..32].try_into().unwrap(),
            keys[32..64].try_into().unwrap(),
            keys[64..].try_into().unwrap(),
        )
        .map_err(|_| io::Error::other("scene binding"))?;
        let mut ends = [0u8; 16];
        control.read_exact(&mut ends)?;
        let handle = |bytes: &[u8]| -> io::Result<_> {
            let raw = usize::try_from(u64::from_le_bytes(bytes.try_into().unwrap()))
                .map_err(|_| io::Error::other("scene handle"))?
                as windows_sys::Win32::Foundation::HANDLE;
            use windows_sys::Win32::Storage::FileSystem::{FILE_TYPE_PIPE, GetFileType};
            require(unsafe { GetFileType(raw) } == FILE_TYPE_PIPE, "scene pipe")?;
            Ok(raw)
        };
        let read = handle(&ends[..8])?;
        let write = handle(&ends[8..])?;
        require(read != write, "scene ends")?;
        Ok(Self {
            binding,
            reader: unsafe { File::from_raw_handle(read) },
            writer: unsafe { File::from_raw_handle(write) },
        })
    }
    fn send(&mut self, kind: Handshake) -> io::Result<()> {
        self.writer.write_all(
            &self
                .binding
                .handshake_frame(kind)
                .map_err(|_| io::Error::other("scene handshake"))?,
        )
    }
    fn receive(&mut self, kind: Handshake) -> io::Result<()> {
        let mut frame = [0; HANDSHAKE_BYTES];
        self.reader.read_exact(&mut frame)?;
        self.binding
            .verify_handshake(kind, &frame)
            .map_err(|_| io::Error::other("scene peer"))
    }
}
pub fn exchange(control: File, mode: window_receipt::SceneOperation) -> io::Result<String> {
    let mut ch = Channel::open(control)?;
    ch.send(Handshake::Hello)?;
    ch.receive(Handshake::Ready)?;
    match mode {
        window_receipt::SceneOperation::Once => {
            ch.binding.read_scene((&mut ch.reader, &mut ch.writer))
        }
        window_receipt::SceneOperation::CheckEditConflict => ch
            .binding
            .check_edit_conflict(&mut ch.reader, &mut ch.writer),
        window_receipt::SceneOperation::CheckDeniedSave => {
            ch.binding.check_denied_save(&mut ch.reader, &mut ch.writer)
        }
    }
    .map_err(|_| io::Error::other("scene read"))
}
pub fn serve(profile: &str, mode: window_receipt::SceneOperation) -> io::Result<()> {
    let (input, _output) = confirm_bootstrap(Some(profile), Role::Data)?;
    let mut ch = Channel::open(input)?;
    ch.receive(Handshake::Hello)?;
    ch.send(Handshake::Ready)?;
    match mode {
        window_receipt::SceneOperation::Once => {
            ch.binding.serve_scene_read(&mut ch.reader, &mut ch.writer)
        }
        window_receipt::SceneOperation::CheckEditConflict => ch
            .binding
            .serve_edit_conflict(&mut ch.reader, &mut ch.writer),
        window_receipt::SceneOperation::CheckDeniedSave => {
            ch.binding.serve_denied_save(&mut ch.reader, &mut ch.writer)
        }
    }
    .map_err(|_| io::Error::other("scene service"))
}
