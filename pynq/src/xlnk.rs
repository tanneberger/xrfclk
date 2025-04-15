use std::ffi::CString;

const XLNK_IOCALLOCBUF: libc::c_ulong = 0xc0045802;
const XLNK_IOCFREEBUF: libc::c_ulong = 0xc0045803;

// from `drivers/staging/apf/xlnk.h`
#[repr(C)]
struct AllocBufIoctlArg {
    len: u32,
    id: i32,
    phyaddr: u32,
    cacheable: u8,
}
#[repr(C)]
struct FreeBufIoctlArg {
    id: u32,
    buf: u32,
}

/// thin wrapper around the /dev/xlnk pseudo file
pub struct Xlnk {
    fd: libc::c_int,
}
impl Xlnk {
    pub fn open() -> Self {
        let pseudo_file = CString::new("/dev/xlnk").unwrap();
        let fd = unsafe { libc::open(pseudo_file.as_ptr(), libc::O_RDWR) };
        assert!(fd > -1, "Failed to open /dev/xlnk.");
        Xlnk { fd }
    }
    pub fn alloc_buf(&mut self, length: usize, is_cacheable: bool) -> (u32, u32) {
        let mut args = AllocBufIoctlArg {
            len: length as u32,
            id: -1,
            phyaddr: 0,
            cacheable: if is_cacheable { 1 } else { 0 },
        };
        let ret = unsafe { libc::ioctl(self.fd, XLNK_IOCALLOCBUF, &mut args) };
        assert!(ret >= 0, "xlnk.alloc_buf failed!");
        (args.id as u32, args.phyaddr)
    }
    pub fn free_buf(&mut self, id: u32) {
        let mut args = FreeBufIoctlArg { id: id, buf: 0 };
        let ret = unsafe { libc::ioctl(self.fd, XLNK_IOCFREEBUF, &mut args) };
        assert!(ret >= 0, "xlnk.free_buf failed!");
    }
    pub unsafe fn mmap_buffer(&mut self, id: u32, length: usize) -> *mut libc::c_void {
        let offset = id << 24;
        let mm = libc::mmap(
            std::ptr::null_mut(),
            length,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED | libc::MAP_LOCKED,
            self.fd,
            offset as libc::c_long,
        );
        assert!(mm != libc::MAP_FAILED, "Failed to mmap DMA buffer.");
        mm
    }
}
impl Drop for Xlnk {
    fn drop(&mut self) {
        assert!(
            unsafe { libc::close(self.fd) } == 0,
            "Failed to close /dev/xlnk."
        );
    }
}
