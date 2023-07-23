# Hab Magtag user-interface device

## Setting up development environment

Follow instructions in Rust ESP Book at: https://esp-rs.github.io/book/installation/index.html

For flashing, add your user to dialout or similar group to be able to access /dev/ttyACM0

## Building/Flashing

Access the bootloader on the Magtag by holding the Boot0 button, clicking Reset and then releasing
Boot0. A serial port should then show up in `dmesg` output.

* cargo build
* cargo espflash flash

Hit the Reset button to start executing the code.

