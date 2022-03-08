import ctypes
from pathlib import Path
from contextlib import contextmanager

__all__ = [
    "FFIVec",
    "mod_widgets",
    "display_widgets",
    "free_ffi_vec",
]


class FFIVec(ctypes.Structure):
    _fields_ = [
        ("len", ctypes.c_ssize_t),
        ("data", ctypes.c_void_p),
        ("_storage", ctypes.c_void_p),
    ]

    @classmethod
    def from_bytes(cls, buf: bytes) -> "FFIVec":
        length = len(buf)
        buf = bytearray(buf)
        ptr = (ctypes.c_uint8 * length).from_buffer(buf)
        data = ctypes.cast(ptr, ctypes.c_void_p)
        return FFIVec(len=length, data=data, _storage=None)

    def to_bytes(self) -> bytes:
        ptr = (ctypes.c_uint8 * self.len).from_address(self.data)
        return bytes(ptr)

    @contextmanager
    def guard(self):
        try:
            yield self
        finally:
            free_ffi_vec(self)


PFFIVec = ctypes.POINTER(FFIVec)

widget_pybind = ctypes.cdll.LoadLibrary(str(Path(__file__).parent / "libwidget_pybind.so"))

mod_widgets = widget_pybind.mod_widgets
mod_widgets.argtypes = (PFFIVec, PFFIVec)
mod_widgets.restype = PFFIVec

free_ffi_vec = widget_pybind.free_ffi_vec
free_ffi_vec.argtypes = (PFFIVec,)

display_widgets = widget_pybind.display_widgets
display_widgets.argtypes = (PFFIVec,)
