# StarDust OS

Immutable, image-based gaming OS. x86 first, ARM later.

## Layout
- `image/` – Containerfile (Bazzite base) + `system_files/` overlaid onto the OS
- `core/` – Rust: tenant/runtime policy to sandboxed launch command (bubblewrap)
- `.github/workflows/build.yml` – builds and publishes the OCI image

## Try it
    cd core && cargo test
    podman build -t stardust-os -f image/Containerfile image
    sudo bootc switch ghcr.io/<you>/stardust-os:latest   # on a Bazzite/Fedora Atomic machine

## Next milestones
1. stardust-broker service (D-Bus): grants Wayland/PipeWire/input per launch
2. Library scanner + SQLite metadata store
3. Shell UI (Rust + wgpu/Slint) running under Gamescope
4. Runtime registry CLI: install from Flathub / signed repo / untrusted AppImage
5. Plugin SDK (manifest-declared capabilities)
