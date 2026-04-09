# lx-monitor

Linux Hardware Monitor - A modern system resource monitor for Linux (CPU, GPU, temperatures).

## Features
- Real-time CPU usage monitoring.
- GPU monitoring (AMD support via sysfs).
- Temperature tracking for all system components.
- Process manager with memory reporting.

## Installation

### Arch Linux (AUR)
You can build and install using the provided `PKGBUILD`:
```bash
makepkg -si
```

### Manual Build
Ensure you have Rust installed and the following system dependencies:
`libx11`, `libxi`, `libxcursor`, `libxrandr`, `libxinerama`, `libxkbcommon`, `pciutils`.

```bash
cargo build --release
```

## License
MIT
