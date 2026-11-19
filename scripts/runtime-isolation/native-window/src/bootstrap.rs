//! Guest-only one-shot handshake. The protected receipt contains no key material.
use crate::*;
use kraqel_native_scene_model::channel::{Binding, HANDSHAKE_BYTES, Handshake};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    os::windows::io::FromRawHandle,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{FreeSid, Isolation::*},
    System::{Memory::*, Threading::*},
};
use zeroize::Zeroizing;

#[derive(Clone, Copy, PartialEq)]
pub enum Role {
    Window,
    Data,
}
impl Role {
    pub fn receipt(self) -> std::path::PathBuf {
        Path::new(DIR).join(match self {
            Self::Window => "window.start",
            Self::Data => "data.start",
        })
    }
    fn name(self) -> &'static str {
        match self {
            Self::Window => "window",
            Self::Data => "data",
        }
    }
}

pub fn receipt(role: Role, ui: &str, data: &str, section: usize) -> String {
    format!("BP4b-v1/A/edit/{}/{ui}/{data}\n{section}", role.name())
}
// Section bytes: context digest, channel ID, two keys, receiver-local read/write handles.
const SIZE: usize = 32 * 4 + 8 * 2;
fn context(role: Role, ui: &str, data: &str) -> io::Result<[u8; 32]> {
    let mut hash = Sha256::new();
    hash.update(receipt(role, ui, data, 0).as_bytes());
    hash.update(std::fs::read(Path::new(DIR).join("anchor.json"))?);
    Ok(hash.finalize().into())
}
pub fn section(
    role: Role,
    ui: &str,
    data: &str,
    ends: [usize; 2],
    material: &[u8; 96],
) -> io::Result<Handle> {
    unsafe {
        let mapping = Handle(CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            null(),
            PAGE_READWRITE,
            0,
            SIZE as u32,
            null(),
        ));
        require(!mapping.0.is_null(), "bootstrap section")?;
        let digest = context(role, ui, data)?;
        let view = MapViewOfFile(mapping.0, FILE_MAP_WRITE, 0, 0, SIZE);
        require(!view.Value.is_null(), "bootstrap staging view")?;
        let bytes = std::slice::from_raw_parts_mut(view.Value.cast::<u8>(), SIZE);
        bytes[..32].copy_from_slice(&digest);
        bytes[32..128].copy_from_slice(material);
        bytes[128..136].copy_from_slice(&(ends[0] as u64).to_le_bytes());
        bytes[136..144].copy_from_slice(&(ends[1] as u64).to_le_bytes());
        check(UnmapViewOfFile(view))?;
        Ok(mapping)
    }
}

pub struct Endpoint {
    binding: Binding,
    reader: File,
    writer: File,
}

impl Endpoint {
    pub fn open(role: Role) -> io::Result<Self> {
        let (image, profile) = match role {
            Role::Window => (CHILD, PROFILE),
            Role::Data => ("pipe-service.exe", "org.kraqel.bp4.data"),
        };
        guest_image(image)?;
        let own = unsafe {
            let mut sid = null_mut();
            require(
                DeriveAppContainerSidFromAppContainerName(wide(profile).as_ptr(), &mut sid) >= 0,
                "role SID",
            )?;
            let result = policy(GetCurrentProcess(), sid);
            FreeSid(sid);
            result?;
            ticket(GetCurrentProcess())?
        };
        let receipt = std::fs::read_to_string(role.receipt())?;
        let (meta, handle) = receipt
            .split_once('\n')
            .ok_or(io::Error::other("receipt"))?;
        let prefix = format!("BP4b-v1/A/edit/{}/", role.name());
        let pair = meta.strip_prefix(&prefix).ok_or(io::Error::other("role"))?;
        let boundary = pair
            .find("/KRAQEL-NATIVE-EDIT-v1/")
            .ok_or(io::Error::other("pair"))?;
        let (ui, data) = (&pair[..boundary], &pair[boundary + 1..]);
        require(
            ui != data && own == if role == Role::Window { ui } else { data },
            "generation",
        )?;
        let id = handle
            .parse::<usize>()
            .map_err(|_| io::Error::other("section"))?;
        let digest = context(role, ui, data)?;
        unsafe {
            let section = Handle(id as HANDLE);
            let view = MapViewOfFile(section.0, FILE_MAP_READ, 0, 0, SIZE);
            require(!view.Value.is_null(), "bootstrap read")?;
            let mut bytes = Zeroizing::new([0u8; SIZE]);
            bytes.copy_from_slice(std::slice::from_raw_parts(view.Value.cast::<u8>(), SIZE));
            check(UnmapViewOfFile(view))?;
            require(bytes[..32] == digest, "context")?;
            let binding = Binding::from_keys(
                bytes[32..64].try_into().unwrap(),
                bytes[64..96].try_into().unwrap(),
                bytes[96..128].try_into().unwrap(),
            )
            .map_err(|_| io::Error::other("keys"))?;
            let read = u64::from_le_bytes(bytes[128..136].try_into().unwrap()) as usize;
            let write = u64::from_le_bytes(bytes[136..144].try_into().unwrap()) as usize;
            require(
                read != write && read != id && write != id && read != 0 && write != 0,
                "endpoints",
            )?;
            Ok(Self {
                binding,
                reader: File::from_raw_handle(read as HANDLE),
                writer: File::from_raw_handle(write as HANDLE),
            })
        }
    }
    fn send(&mut self, kind: Handshake) -> io::Result<()> {
        self.writer.write_all(
            &self
                .binding
                .handshake_frame(kind)
                .map_err(|_| io::Error::other("encode"))?,
        )
    }
    fn receive(&mut self, kind: Handshake) -> io::Result<bool> {
        let mut frame = [0; HANDSHAKE_BYTES];
        self.reader.read_exact(&mut frame)?;
        Ok(self.binding.verify_handshake(kind, &frame).is_ok())
    }
    pub fn window(mut self) -> io::Result<i32> {
        self.send(Handshake::Hello)?;
        Ok(if self.receive(Handshake::Ready)? {
            62
        } else {
            63
        })
    }
    pub fn service(mut self) -> io::Result<i32> {
        if !self.receive(Handshake::Hello)? {
            return Ok(61);
        }
        self.send(Handshake::Ready)?;
        Ok(60)
    }
}
