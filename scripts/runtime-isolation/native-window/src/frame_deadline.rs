//! Receive deadline for an in-flight frame on the synchronous, single-reader guest pipe.
use crate::{
    require,
    scene_store::{Store, StorePhase},
};
use kraqel_native_scene_model::channel::{Binding, Closed, PARTIAL_SAVE_BYTES};
use std::{
    cell::Cell,
    fs::File,
    io::{self, Read},
    os::windows::io::AsRawHandle,
    ptr::null_mut,
    time::{Duration, Instant},
};
use windows_sys::Win32::System::Pipes::PeekNamedPipe;

const FRAME_TIMEOUT: Duration = Duration::from_secs(30);

struct FrameReader<'a> {
    pipe: &'a File,
    completed: &'a Cell<usize>,
    generation: usize,
    started: Option<Instant>,
    received: usize,
    ending: Option<io::ErrorKind>,
}

impl FrameReader<'_> {
    fn pending_read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let Some(start) = self.started else {
            // Waiting for the first byte is idle time.
            return self.pipe.read(buffer);
        };
        loop {
            if start.elapsed() >= FRAME_TIMEOUT {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let mut available = 0;
            let ok = unsafe {
                PeekNamedPipe(
                    self.pipe.as_raw_handle(),
                    null_mut(),
                    0,
                    null_mut(),
                    &mut available,
                    null_mut(),
                )
            };
            if ok == 0 {
                return Err(io::Error::last_os_error());
            }
            if available > 0 {
                // This endpoint has exactly one reader. Only read already buffered bytes.
                let n = buffer.len().min(available as usize);
                return self.pipe.read(&mut buffer[..n]);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Read for FrameReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if self.generation != self.completed.get() {
            self.generation = self.completed.get();
            self.started = None;
            self.received = 0;
        }
        let result = self.pending_read(buf);
        match &result {
            Ok(0) => self.ending = Some(io::ErrorKind::UnexpectedEof),
            Ok(n) => {
                self.started.get_or_insert_with(Instant::now);
                self.received += n;
            }
            Err(e) => self.ending = Some(e.kind()),
        }
        result
    }
}

pub(crate) fn serve(
    binding: Binding,
    pipe: File,
    mut writer: File,
    store: &mut Store,
    phase: Option<StorePhase>,
    reopening: bool,
) -> io::Result<()> {
    let completed = Cell::new(0);
    let mut reader = FrameReader {
        pipe: &pipe,
        generation: 0,
        completed: &completed,
        started: None,
        received: 0,
        ending: None,
    };
    let mut rolled_back = false;
    let result = binding.durable_service(reopening, &mut reader, &mut writer, |body| {
        let reply = store
            .dispatch(body, || {
                let mut input = &pipe;
                match input.read(&mut [0; 1]) {
                    Ok(0) => Ok(()),
                    Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
                    _ => Err(io::Error::other("expected window EOF")),
                }
            })
            .map_err(|error| {
                rolled_back = error.kind() == io::ErrorKind::ConnectionAborted;
                Closed
            })?;
        completed.set(completed.get() + 1);
        Ok(reply)
    });
    if phase == Some(StorePhase::WindowClose) {
        return require(
            result.is_err() && rolled_back && completed.get() == 1,
            "window-end rollback",
        );
    }
    let expected = match phase {
        Some(StorePhase::Partial) => matches!(
            reader.ending,
            Some(io::ErrorKind::UnexpectedEof | io::ErrorKind::BrokenPipe)
        ),
        Some(StorePhase::Timeout) => reader.ending == Some(io::ErrorKind::TimedOut),
        _ => return result.map_err(|_| io::Error::other("durable request")),
    };
    require(
        result.is_err()
            && completed.get() == 1
            && reader.received == PARTIAL_SAVE_BYTES
            && expected,
        "partial request outcome",
    )
}
