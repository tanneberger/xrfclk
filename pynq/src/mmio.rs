/// Taken from: https://github.com/hegza/pynq-z1-experiments/blob/master/common/src/mmio.rs
/// Adapted [from](https://github.com/ekiwi/pynq).
use std::ffi::CString;
use std::ops::{Drop, Index, IndexMut};

pub struct Mmio {
    mem: *mut u32,
    words: usize,
}
impl Mmio {
    pub fn map(phys_addr: u32, length: u32) -> Self {
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u32;
        assert!(
            phys_addr % page_size == 0,
            "Only page boundary aligned IO is supported!"
        );
        let phys_mem = CString::new("/dev/mem").unwrap();
        let words = ((length + 3) / 4) as usize;
        let mem = unsafe {
            let fd = libc::open(phys_mem.as_ptr(), libc::O_RDWR | libc::O_SYNC);
            assert!(fd > -1, "Failed to open /dev/mem. Are we root?");
            let mm = libc::mmap(
                std::ptr::null_mut(),
                words * 4,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                phys_addr.into(), //libc::c_long,
            );
            assert!(mm != libc::MAP_FAILED, "Failed to mmap physical memory.");
            assert!(libc::close(fd) == 0, "Failed to close /dev/mem.");
            mm as *mut u32
        };
        Mmio { mem, words }
    }

    pub fn copy_from_slice(&mut self, offset_u32: usize, buf: &[u8], length_u32: usize) {
        unsafe {
            let dst_ptr = (self.mem as *mut u32).add(offset_u32);
            let src_ptr = (&buf[0] as *const u8) as *const u32;
            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, length_u32);
        }
    }

    pub fn read_into_slice(&self, offset_u32: usize, buf: &mut [u8], length_u32: usize) {
        unsafe {
            let dst_ptr = (&mut buf[0] as *mut u8) as *mut u32;
            let src_ptr = (self.mem as *const u32).add(offset_u32);
            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, length_u32);
        }
    }

    pub fn get_slice(&mut self) -> &mut [u32] {
        unsafe { std::slice::from_raw_parts_mut(self.mem, self.words) }
    }
}

impl Drop for Mmio {
    fn drop(&mut self) {
        unsafe {
            assert!(
                libc::munmap(self.mem as *mut libc::c_void, self.words * 4) == 0,
                "Failed to unmap IO."
            );
        }
    }
}

impl Index<usize> for Mmio {
    type Output = u32;
    fn index(&self, ii: usize) -> &u32 {
        unsafe { &std::slice::from_raw_parts(self.mem, self.words)[ii] }
    }
}
impl IndexMut<usize> for Mmio {
    fn index_mut(&mut self, ii: usize) -> &mut u32 {
        unsafe { &mut std::slice::from_raw_parts_mut(self.mem, self.words)[ii] }
    }
}
