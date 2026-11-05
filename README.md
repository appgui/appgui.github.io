# appGUI

[![License: AGPL-3.0-or-later](https://img.shields.io/badge/License-AGPL--3.0--or--later-green.svg?style=flat)](https://www.gnu.org/licenses/agpl-3.0.en.html)

Proof-of-concept project for creating a universal boilerplate project using Rust + Slint.

## 0. Initial setup (runtime dependencies)

_Installing host dependencies (minimum for native app compilation)_

#### A. Universal command:

```sh
# if `nix` package manager is available
nix develop

# if only `just` is available
just desktop_set_deps
```

#### B. Platform-specific commands:

```sh
# for Linux:
./setup_linux.sh

# for MacOS:
sh ./setup_macos.sh

# for Windows:
pwsh ./setup_windows.ps1 || powershell ./setup_windows.ps1
```

# 1. Debugging the app natively

```sh
# (optional) dev dependencies and IDE support
just desktop_set_tools_dev

# for desktop package
just desktop run

# for mobile application (on connected mobile device)
just android run
```

# 2. Producing desktop installers / bundles / binaries

_desktop package:_

```sh
just desktop build
#   or
# just linux_build native
```

_mobile application:_

```sh
just android build
```
