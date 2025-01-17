use crate::mmio::Mmio;
use std::collections::HashMap;
use std::ops::Index;
use std::rc::Rc;

trait IpBlock {
    fn write(&mut self, addr: usize, value: &[u32]);

    fn read(&self, addr: usize) -> u32;
}

struct DefaultIpBlock {
    mmio: Mmio,
}

impl IpBlock for DefaultIpBlock {
    fn write(&mut self, offset: usize, buf: &[u32]) {
        for i in 0..buf.len() {
            self.mmio[offset + i] = buf[i];
        }
    }

    fn read(&self, offset: usize) -> u32 {
        self.mmio[offset]
    }
}

impl DefaultIpBlock {
    pub fn new(phys_addr: u32, length: u32) -> Self {
        Self {
            mmio: Mmio::map(phys_addr, length),
        }
    }
}

struct Overlay {
    ip_block_map: HashMap<String, Rc<dyn IpBlock>>,
}

impl Index<&str> for Overlay {
    type Output = Rc<dyn IpBlock>;
    fn index(&self, ii: &str) -> &Self::Output {
        &self.ip_block_map.get(ii).unwrap()
    }
}

impl Index<&String> for Overlay {
    type Output = Rc<dyn IpBlock>;
    fn index(&self, ii: &String) -> &Self::Output {
        &self.ip_block_map.get(ii).unwrap()
    }
}

impl Overlay {}
