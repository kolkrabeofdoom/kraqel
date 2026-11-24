"""Pending Windows read oplock used by the bounded synthetic NTFS capture."""
import ctypes
from ctypes import wintypes
from contextlib import contextmanager


class CaptureChanged(RuntimeError):
    """The pending read lease broke before this acquisition completed."""


class Overlapped(ctypes.Structure):
    _fields_ = [("internal", ctypes.c_size_t), ("internal_high", ctypes.c_size_t),
                ("offset", wintypes.DWORD), ("offset_high", wintypes.DWORD), ("event", wintypes.HANDLE)]


class Request(ctypes.Structure):
    _fields_ = [("version", wintypes.WORD), ("size", wintypes.WORD),
                ("level", wintypes.DWORD), ("flags", wintypes.DWORD)]


class Reply(ctypes.Structure):
    _fields_ = [("version", wintypes.WORD), ("size", wintypes.WORD),
                ("original", wintypes.DWORD), ("new", wintypes.DWORD), ("flags", wintypes.DWORD),
                ("access", wintypes.DWORD), ("share", wintypes.WORD)]


kernel = ctypes.WinDLL("kernel32", use_last_error=True)
kernel.CreateEventW.restype = wintypes.HANDLE


@contextmanager
def read_oplock(raw):
    handle = wintypes.HANDLE(raw)
    event = wintypes.HANDLE(kernel.CreateEventW(None, True, False, None))
    if not event.value:
        raise ctypes.WinError(ctypes.get_last_error())
    overlap = Overlapped(event=event.value)
    request = Request(1, ctypes.sizeof(Request), 1, 1)  # CACHE_READ, REQUEST
    reply = Reply()
    transferred = wintypes.DWORD()
    pending = False
    try:
        done = kernel.DeviceIoControl(handle, 590400, ctypes.byref(request), ctypes.sizeof(request),
                                      ctypes.byref(reply), ctypes.sizeof(reply),
                                      ctypes.byref(transferred), ctypes.byref(overlap))
        if done or ctypes.get_last_error() != 997:  # Only ERROR_IO_PENDING denotes a held lease.
            raise RuntimeError("NTFS read oplock unavailable")
        pending = True

        def verify():
            if kernel.WaitForSingleObject(event, 0) != 258:  # WAIT_TIMEOUT
                raise CaptureChanged("project changed during capture")

        verify()
        yield verify
    finally:
        if pending:
            kernel.CancelIoEx(handle, ctypes.byref(overlap))
            # Keep buffers and event alive until completion, including a break racing cancellation.
            kernel.GetOverlappedResult(handle, ctypes.byref(overlap), ctypes.byref(transferred), True)
        kernel.CloseHandle(event)
