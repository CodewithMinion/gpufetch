# gpufetch

**gpufetch** is a simple yet fancy GPU architecture fetching tool written in **Rust**. It displays detailed GPU information in a clean, beautiful, and colorful way.

<img width="4545" height="1875" alt="image" src="https://github.com/user-attachments/assets/08fe470a-5dad-49f5-a488-65393de1a0e8" />

---

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

MIT License - see [LICENSE](https://github.com/CodewithMinion/gpufetch/edit/main/LICENSE) for details.

---

## Support

gpufetch supports the following GPUs:

- **NVIDIA GPUs** (Compute Capability >= 2.0)
- **AMD GPUs** (Experimental) (RDNA 3.0, CDNA 3.0)
- **Intel iGPUs** (Generation >= Gen6)

> **Note:** Only compilation under **Linux** is supported.

---

## Backends

gpufetch is made up of three backends:

- **CUDA backend** (NVIDIA)
- **HSA backend** (AMD)
- **Intel backend**

Backends are enabled and disabled at compile time using Cargo features. When building gpufetch, check the build output to see which backends are enabled.

```bash
cargo build --release --all-features
```

gpufetch will only detect your GPU if the appropriate backend was enabled during compilation (e.g., it will not detect your NVIDIA GPU if the CUDA backend is disabled!).

By default, all backends are enabled. However, they can be manually disabled. See the build commands below for instructions.

### 2.1 CUDA backend is not enabled. Why?

The CUDA toolkit is required to build gpufetch with the CUDA backend enabled. However, when building gpufetch, Cargo may be unable to find the CUDA installation. If CUDA is installed but Cargo does not find it, you need to set the `CUDA_PATH` environment variable:

```bash
export CUDA_PATH=/usr/local/cuda
cargo build --release --features cuda
```

### 2.2 The backend is enabled, but gpufetch is unable to detect my GPU

First, make sure that your GPU is visible in the system. You can print enabled GPUs with `lspci`:

```bash
$ lspci -nn | grep -i vga
$ lspci -nn | grep -i 3d
```

If there is a NVIDIA, AMD, or Intel GPU in the system and the appropriate backend is enabled but gpufetch does not detect the GPU, please create a new issue with the provided error message on the [issues page](https://github.com/CodewithMinion/gpufetch/issues).

---

## Installation (building from source)

**You will need (mandatory):**

- Rust toolchain (1.70+ recommended)
- `make`
- `pkg-config`
- `libpciaccess-dev` (for Intel backend)
- `zlib1g-dev`

**Optionally:**

- **CUDA toolkit** (needed for CUDA backend)

To build gpufetch, just clone the repo and run:

```bash
git clone https://github.com/your-repo/gpufetch.git
cd gpufetch
cargo build --release
./target/release/gpufetch
```

**Build with specific backends:**

```bash
# All backends
cargo build --release --all-features

# Only AMD
cargo build --release --features amd

# Only Intel
cargo build --release --features intel

# Only NVIDIA (CUDA)
cargo build --release --features cuda
```

---

## Colors

By default, gpufetch will print the GPU logo with the vendor-specific color scheme. However, you can set a custom color scheme in two different ways:

### 4.1 Specifying a name

By specifying a name, gpufetch will use the specific colors of each manufacturer. Valid values are:

- `intel`
- `amd`
- `nvidia`

```bash
./target/release/gpufetch --color intel  # default color for Intel
./target/release/gpufetch --color amd    # default color for AMD
./target/release/gpufetch --color nvidia # default color for NVIDIA
```

### 4.2 Specifying the colors in RGB format

5 colors must be given in RGB with the format: `[R,G,B:R,G,B:R,G,B:R,G,B:R,G,B]`. These colors correspond to:

- **First 3 colors:** GPU logo
- **Last 2 colors:** Text colors

```bash
./target/release/gpufetch --color 239,90,45:210,200,200:0,0,0:100,200,45:0,200,200
```

---

## Features

- ✅ **Accurate hardware detection** - RDNA vs CDNA architecture distinction
- ✅ **Panic-safe error handling** - No runtime crashes with typed errors
- ✅ **Beautiful ASCII art logos** - Vendor-specific colors
- ✅ **Aligned output** - Clean, professional formatting
- ✅ **Multi-platform backends** - NVIDIA, AMD, Intel support
- ✅ **Production-ready** - 10/10 code quality with proper error handling

---

---

**gpufetch** - Show off your GPU in style! 🚀
