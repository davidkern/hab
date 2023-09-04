# Hab BCM (Body Control Module)

The BCM uses an STM32F4 Discovery board.

## Setting up development environment

* Install rust: https://rustup.rs/
* Install udev rules and update stm32 board probe firmware: https://www.st.com/en/development-tools/stsw-link007.html
    - For linux, everything is in the stsw-link007/AllPlatforms directory
    - Install the rpm or deb to setup stlink udev rules from the StlinkRulesFilesForLinux directory
    - To update the probe firmware, remove the two jumpers on CN3, connect to usb and run `java -jar STLinkUpgrade.jar`

## Usage

* Run `cargo run` - to start debugging via probe-rs
* Run `probe-rs` for additional commands
