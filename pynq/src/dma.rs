use crate::mmio::Mmio;
use crate::xlnk::Xlnk;

pub struct DmaBuffer {
    id: u32,
    physical_addr: u32,
    data: *mut u8,
    size: usize,
}

impl DmaBuffer {
    pub fn allocate(size: usize) -> Self {
        let mut xlnk = Xlnk::open();
        let (id, physical_addr) = xlnk.alloc_buf(size, false);
        let data = unsafe { xlnk.mmap_buffer(id, size) } as *mut u8;
        DmaBuffer {
            id,
            physical_addr,
            data,
            size,
        }
    }
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.data, self.size) }
    }
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data, self.size) }
    }
}
impl Drop for DmaBuffer {
    fn drop(&mut self) {
        let mm = self.data as *mut libc::c_void;
        assert!(
            unsafe { libc::munmap(mm, self.size) } == 0,
            "Failed to unmap DMA buffer."
        );
        Xlnk::open().free_buf(self.id);
    }
}

pub struct Dma {
    mem: Mmio,
    tx_buffer: Option<DmaBuffer>,
    rx_buffer: Option<DmaBuffer>,
}

impl Dma {
    // TODO: support interrupts
    pub fn get() -> Self {
        let mem = Mmio::map(0x40000000, 2 * 0x30);
        let mut dma = Dma {
            mem,
            tx_buffer: None,
            rx_buffer: None,
        };
        dma.start();
        dma
    }
    fn start(&mut self) {
        // TODO: add timeout
        self.mem[0] = 0x00000001;
        self.mem[12] = 0x00000001;
        while !((self.mem[1] & 1 == 0) && (self.mem[12 + 1] & 1 == 0)) {}
    }
    fn is_tx_idle(&mut self) -> bool {
        self.mem[1] & 2 == 2
    }
    fn is_rx_idle(&mut self) -> bool {
        self.mem[12 + 1] & 2 == 2
    }
    pub fn start_send(&mut self, buf: DmaBuffer) {
        assert!(
            self.tx_buffer.is_none(),
            "Cannot send when transmission is in progress!"
        );
        self.mem[6] = buf.physical_addr;
        self.mem[10] = buf.size as u32;
        self.tx_buffer = Some(buf);
    }
    pub fn is_send_done(&mut self) -> bool {
        self.is_tx_idle()
    }
    pub fn finish_send(&mut self) -> DmaBuffer {
        assert!(
            self.is_send_done(),
            "Cannot finish send when transmission hasn't finished!"
        );
        assert!(
            self.tx_buffer.is_some(),
            "Cannot finish send when no transmission was started!"
        );
        let mut buf = None;
        std::mem::swap(&mut buf, &mut self.tx_buffer);
        buf.unwrap()
    }
    pub fn start_receive(&mut self, buf: DmaBuffer) {
        assert!(
            self.rx_buffer.is_none(),
            "Cannot receive when transmission is in progress!"
        );
        self.mem[12 + 6] = buf.physical_addr;
        self.mem[12 + 10] = buf.size as u32;
        self.rx_buffer = Some(buf);
    }
    pub fn is_receive_done(&mut self) -> bool {
        self.is_rx_idle()
    }
    pub fn finish_receive(&mut self) -> DmaBuffer {
        assert!(
            self.is_receive_done(),
            "Cannot finish receive when transmission hasn't finished!"
        );
        assert!(
            self.rx_buffer.is_some(),
            "Cannot finish receive when no transmission was started!"
        );
        let mut buf = None;
        std::mem::swap(&mut buf, &mut self.rx_buffer);
        buf.unwrap()
    }
}
