# SlintXPlatform

This is a research project for creating a universal boilerplate project using Rust + Slint.

# Producing Linux binary

Option 1: appimage

```sh
just setup_host_tools
just setup_appimagetool
just build_desktop
```

Option 2: binary (fallback on no `appimagetool`)
```sh
just build_desktop --release
```
