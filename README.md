# autotiling-rs
When used on sway (and possibly i3), this automatically alternates the container layout between horizontal and vertical for successive new containers.

## Usage
Simply run the program `autotiling-rs`. To start it automatically, put it in your sway config like this: `exec autotiling-rs`.

For more info run `autotiling-rs --help`.

## Installation

Arch Linux: [autotiling-rs](https://archlinux.org/packages/?q=autotiling-rs)

Arch Linux (AUR): [autotiling-rs-git](https://aur.archlinux.org/packages/autotiling-rs-git).

Nix: [autotiling-rs](https://search.nixos.org/packages?channel=unstable&show=autotiling-rs&from=0&size=50&sort=relevance&type=packages&query=autotiling-rs)

## Compiling
```
cargo build --release
```
