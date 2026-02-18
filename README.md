# xPlatform

This is a research project for creating a universal boilerplate project using Rust + Slint.

## 0. Initial setup

_Installing host dependencies (minimum for native app compilation)_

#### A. Universal command (if `just` is already installed):

```sh
just desktop_set_deps
```

#### B. Platform-specific commands:

Linux:

```sh
./setup_linux.sh
```

MacOS:

```sh
sh ./setup_macos.sh
```

Windows:

```sh
pwsh ./setup_windows.ps1 || powershell ./setup_windows.ps1
```

# 1. Debugging the app natively

_desktop package:_

```sh
just desktop run
```

_mobile application (on connected mobile device):_

```sh
just android run
```

# 2. Producing desktop installers / bundles / binaries

_desktop package:_

```sh
just desktop build
```

_mobile application:_

```sh
just android build
```
