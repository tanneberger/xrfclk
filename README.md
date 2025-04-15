xilinx.rs
-----------

**This library is not from xilinx**

This rust crate is a rust rewrite of the xrfclk library from xilinx, which 
is used to control the clocks of different boards like the ZCU212.

## Cross compilinx for PYNQ Boards

The easiest way is to use the cross-tool which uses docker containers with preinstalled toolchains.

```bash
    $ cross build --target armv7-unknown-linux-gnueabihf --all
```

otherwise use cargos cross compilation capability. 

```bash
    $ rustup target add armv7-unknown-linux-gnueabihf
    $ cargo build --target armv7-unknown-linux-gnueabihf --all
```

## Future Work

Currently this implementation uses the xilinx-xrfclk driver, which handles the communication with the IC. 
In the future it would be worth implementing a "bare-metal" version which handles the I2C communication, that
this library becomes more platform independent. A good start would be xilinx's [driver implementation](https://github.com/Xilinx/embeddedsw/blob/master/XilinxProcessorIPLib/drivers/board_common/src/rfclk/src/xrfclk.c).

## Useful Links
- Board Used: https://www.xilinx.com/products/boards-and-kits/zcu216.html
- https://xilinx-wiki.atlassian.net/wiki/spaces/A/pages/769229238/XM650+Example+Design+-+RF+DC+Evaluation+Tool
- https://www.ti.com/product/LMX2594
- https://www.ti.com/product/LMK04208
- https://www.ti.com/product/LMK04832

