# Linux Binary Import Guide

Complete guide for importing and using `@omron-fins/native` binaries on Linux with Node.js or Bun.

## Table of Contents

1. [Overview](#overview)
2. [Installation Methods](#installation-methods)
3. [Using with npm Package](#using-with-npm-package)
4. [Direct Binary Download](#direct-binary-download)
5. [FFI Loading](#ffi-loading)
6. [Bun Specifics](#bun-specifics)
7. [Platform Requirements](#platform-requirements)
8. [Error Handling](#error-handling)
9. [Distribution](#distribution)
10. [Examples](#examples)

---

## Overview

| Property | Value |
|----------|-------|
| Package | `@omron-fins/native` |
| Binary Name | `omron-fins-v{version}-{arch}-linux-gnu` |
| Supported Architectures | `x86_64`, `aarch64` |
| C Runtime | glibc 2.17+ (CentOS 7+, Ubuntu 18.04+, etc.) |
| Node.js | >= 18 |
| Bun | >= 1.0 |

---

## Installation Methods

### Method 1: npm Package (Recommended)

```bash
npm install @omron-fins/native
```

```bash
bun add @omron-fins/native
```

### Method 2: Direct Binary Download

Download the appropriate binary from the [releases page](https://github.com/deviagomendes/omron-fins-rs/releases):

| Architecture | Filename | File Size |
|--------------|----------|-----------|
| x86_64 | `omron-fins-v0.6.0-x86_64-unknown-linux-gnu.node` | ~2.3 MB |
| aarch64 | `omron-fins-v0.6.0-aarch64-unknown-linux-gnu.node` | ~1.8 MB |

---

## Using with npm Package

### JavaScript/TypeScript

```javascript
const { FinsClient, FinsMemoryArea, FinsDataType } = require('@omron-fins/native');

async function main() {
  const client = new FinsClient('192.168.1.250', 1, 0);

  const data = await client.read('DM', 100, 10);
  console.log('DM100-109:', data);

  const bit = await client.readBit('CIO', 0, 5);
  console.log('CIO 0.05:', bit);

  await client.write('DM', 200, [0x1234, 0x5678]);

  await client.close?.(); // if available
}

main().catch(console.error);
```

### TypeScript

```typescript
import { FinsClient, FinsMemoryArea, FinsDataType } from '@omron-fins/native';

async function main(): Promise<void> {
  const client = new FinsClient('192.168.1.250', 1, 0, {
    timeoutMs: 5000,
    port: 9600
  });

  // Read 10 words from DM100
  const data: number[] = await client.read(FinsMemoryArea.DM, 100, 10);
  console.log('DM100-109:', data);

  // Read a specific bit
  const bit: boolean = await client.readBit(FinsMemoryArea.CIO, 0, 5);
  console.log('CIO 0.05:', bit);

  // Write values
  await client.write(FinsMemoryArea.DM, 200, [0x1234, 0x5678]);

  // Read float (f32)
  const temperature: number = await client.readF32(FinsMemoryArea.DM, 300);
  console.log('Temperature:', temperature.toFixed(2));

  // Read struct
  const struct = await client.readStruct(FinsMemoryArea.DM, 400, [
    FinsDataType.LINT,
    FinsDataType.INT,
    FinsDataType.REAL
  ]);
  console.log('Struct:', struct);
}

main().catch(console.error);
```

---

## Direct Binary Download

### Step 1: Download the Binary

```bash
# x86_64
curl -L -o omron-fins.node \
  https://github.com/deviagomendes/omron-fins-rs/releases/download/v0.6.0/omron-fins-v0.6.0-x86_64-unknown-linux-gnu.node

# aarch64
curl -L -o omron-fins.node \
  https://github.com/deviagomendes/omron-fins-rs/releases/download/v0.6.0/omron-fins-v0.6.0-aarch64-unknown-linux-gnu.node
```

### Step 2: Verify the Binary

```bash
# Check file type
file omron-fins.node
# Output: omron-fins.node: ELF 64-bit LSB shared object, x86-64, version 1 (SYSV), dynamically linked

# Check dependencies
ldd omron-fins.node
# Should show: libm.so.6, libpthread.so.0, libc.so.6, libdl.so.2, librt.so.1, libgcc_s.so.1

# Verify version (if built with version info)
readelf -s omron-fins.node | head -20
```

### Step 3: Set Permissions

```bash
chmod +x omron-fins.node
```

---

## FFI Loading

### Using `node:ffi-napi`

```javascript
const ffi = require('ffi-napi');
const path = require('path');

// Load the native binary
const omronFins = ffi.Library(
  path.join(__dirname, 'omron-fins.node'),
  {
    // FinsClient constructor
    'FinsClient': ['pointer', ['string', 'uint8', 'uint8', 'pointer']],
    // Instance methods
    'read': ['pointer', ['pointer', 'uint32', 'uint16', 'uint16']],
    'readBit': ['bool', ['pointer', 'uint32', 'uint16', 'uint8']],
    'write': ['void', ['pointer', 'uint32', 'uint16', 'pointer', 'uint32']],
    // Utility functions
    'getBit': ['bool', ['uint32', 'uint8']],
    'setBit': ['uint32', ['uint32', 'uint8', 'bool']],
    'formatHex': ['string', ['uint32']],
  }
);

// Helper to create JavaScript error from Rust Result
function handleResult(result) {
  if (result.err) {
    throw new Error(`FINS Error: ${result.err}`);
  }
  return result.ok;
}

module.exports = { omronFins };
```

### Using `@putout/ffi`

```javascript
const { load } = require('@putout/ffi');
const { CString, uint32, uint16, uint8, bool, pointer } = require('@putout/ffi').types;
const path = require('path');

const lib = load({
  name: 'omron-fins',
  path: path.join(__dirname, 'omron-fins.node'),
  structs: {
    FinsClient: {
      host: CString,
      sourceNode: uint8,
      destNode: uint8,
    }
  },
  functions: {
    read: [pointer, [pointer, uint32, uint16, uint16]],
    readBit: [bool, [pointer, uint32, uint16, uint8]],
    write: [void, [pointer, uint32, uint16, pointer, uint32]],
  }
});

module.exports = lib;
```

### Manual Dynamic Loading

```javascript
const { load } = require('node:module');
const path = require('path');
const { execSync } = require('child_process');

// Detect architecture
function getArch() {
  const arch = process.arch;
  const platform = process.platform;
  if (platform !== 'linux') {
    throw new Error(`Unsupported platform: ${platform}`);
  }
  if (arch === 'x64') return 'x86_64';
  if (arch === 'arm64') return 'aarch64';
  throw new Error(`Unsupported architecture: ${arch}`);
}

// Get libc flavor (glibc vs musl)
function getLibcFlavor() {
  try {
    execSync('ldd --version 2>&1 | grep -q musl && echo musl || echo glibc');
    return 'musl';
  } catch {
    return 'glibc';
  }
}

// Load the binary
const arch = getArch();
const libc = getLibcFlavor();
const binaryName = `omron-fins-v0.6.0-${arch}-unknown-linux-${libc}.node`;
const binaryPath = path.join(__dirname, 'binaries', arch, binaryName);

const nativeBinding = load(binaryPath);

module.exports = { nativeBinding, binaryPath };
```

---

## Bun Specifics

### Using native imports

```typescript
import { FinsClient, FinsMemoryArea } from '@omron-fins/native';

const client = new FinsClient('192.168.1.250', 1, 0);

const data = await client.read(FinsMemoryArea.DM, 100, 10);
console.log('DM100-109:', data);
```

### Using FFI with Bun

```typescript
import { load } from 'bun:ffi';

const lib = load({
  name: 'omron-fins',
  path: './omron-fins.node',
  symbols: {
    read: {
      args: ['pointer', 'uint32', 'uint16', 'uint16'],
      returns: 'pointer',
    },
  },
});

const result = lib.symbols.read(/* ... */);
```

### Bun FFI with Bun.$ (experimental)

```typescript
const lib = Bun.$({
  path: './omron-fins.node',
  symbols: {
    read: ['pointer', ['pointer', 'uint32', 'uint16', 'uint16']],
    readBit: ['bool', ['pointer', 'uint32', 'uint16', 'uint8']],
  },
});
```

---

## Platform Requirements

### glibc vs musl

The pre-built binaries use **glibc**. For Alpine Linux (musl), build from source:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build for musl target
rustup target add x86_64-unknown-linux-musl
cargo build --release --features napi --target x86_64-unknown-linux-musl

# Output: target/x86_64-unknown-linux-musl/release/libomron_fins.so
```

### Distribution Compatibility

| Distribution | glibc Version | Compatible |
|--------------|---------------|------------|
| Ubuntu 18.04+ | glibc 2.27 | ✅ |
| Debian 10+ | glibc 2.28 | ✅ |
| CentOS 7+ | glibc 2.17 | ✅ |
| RHEL 7+ | glibc 2.17 | ✅ |
| Fedora 28+ | glibc 2.27 | ✅ |
| Alpine Linux | musl | ❌ (build from source) |
| openSUSE Leap 15+ | glibc 2.26 | ✅ |

### Check glibc Version

```bash
# Method 1: ldd version
ldd --version

# Method 2: Check installed glibc
ldconfig -p | grep libc.so

# Method 3: Direct check
objdump -T /lib/x86_64-linux-gnu/libc.so.6 | grep GLIBC_2 | tail -1
```

---

## Error Handling

### JavaScripttry/catch

```javascript
const { FinsClient } = require('@omron-fins/native');

async function main() {
  const client = new FinsClient('192.168.1.250', 1, 0);

  try {
    const data = await client.read('DM', 100, 10);
    console.log('Success:', data);
  } catch (error) {
    if (error.message.includes('Timeout')) {
      console.error('PLC did not respond - check connection');
    } else if (error.message.includes('PLC error')) {
      console.error('PLC returned an error - check PLC status');
    } else if (error.message.includes('Invalid')) {
      console.error('Invalid parameter - check address and area');
    } else {
      console.error('Unexpected error:', error.message);
    }
  }
}

main();
```

### Error Types

| Error Pattern | Likely Cause | Solution |
|---------------|--------------|----------|
| `Timeout` | PLC unreachable, wrong IP | Verify network connection, check PLC IP |
| `PLC error: main=X sub=Y` | PLC rejected command | Check PLC error codes in manual |
| `Invalid addressing` | Bit access on DM | DM doesn't support bit access |
| `Invalid parameter` | Count exceeds limit | Use count <= 999 |
| `ENOENT` | Binary not found | Reinstall npm package |
| `GLIBC_2.X not found` | Incompatible libc | Use correct binary or build from source |

### TypeScript with Result Pattern

```typescript
type Result<T> = { ok: T } | { err: Error };

async function safeRead(
  client: FinsClient,
  area: string,
  address: number,
  count: number
): Promise<Result<number[]>> {
  try {
    const data = await client.read(area, address, count);
    return { ok: data };
  } catch (error) {
    return { err: error as Error };
  }
}

// Usage
const result = await safeRead(client, 'DM', 100, 10);
if ('ok' in result) {
  console.log('Data:', result.ok);
} else {
  console.error('Error:', result.err.message);
}
```

---

## Distribution

### Method 1: npm Package (Recommended)

The `@omron-fins/native` package handles everything automatically:

```json
{
  "name": "my-plc-app",
  "dependencies": {
    "@omron-fins/native": "^0.6.0"
  }
}
```

### Method 2: Direct Binary with PATH

Download binary to a location in PATH:

```bash
# /usr/local/bin/omron-fins.node
sudo cp omron-fins.node /usr/local/bin/

# Or ~/.local/bin/
mkdir -p ~/.local/bin
cp omron-fins.node ~/.local/bin/

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.local/bin:$PATH"
```

### Method 3: Application Bundle

Include binary in your application distribution:

```
my-plc-app/
├── bin/
│   └── omron-fins.node    # Platform-specific binary
├── lib/
│   └── index.js           # Your application
├── package.json
└── README.md
```

### Method 4: npm bin Field

Add to your `package.json`:

```json
{
  "name": "my-plc-app",
  "bin": {
    "omron-fins": "./bin/omron-fins.node"
  },
  "dependencies": {
    "@omron-fins/native": "^0.6.0"
  }
}
```

Install and run:

```bash
npm install
npx omron-fins --version
```

### Cross-Platform Distribution

For packages that need to work across platforms:

```json
{
  "name": "my-cross-platform-app",
  "scripts": {
    "postinstall": "npx @omron-fins/native/scripts/select-binary.js"
  }
}
```

---

## Examples

### Basic Read/Write

```javascript
const { FinsClient } = require('@omron-fins/native');

async function plcDemo() {
  console.log('Connecting to PLC at 192.168.1.250...');

  const client = new FinsClient('192.168.1.250', 1, 0, {
    timeoutMs: 3000,
    port: 9600
  });

  console.log('Reading DM100-DM109...');
  const words = await client.read('DM', 100, 10);
  console.log('Words:', words);

  console.log('Writing to DM200...');
  await client.write('DM', 200, [0x0001, 0x0002, 0x0003]);

  console.log('Reading CIO 0.00-0.15...');
  for (let bit = 0; bit < 16; bit++) {
    const value = await client.readBit('CIO', 0, bit);
    console.log(`  CIO 0.${bit} = ${value}`);
  }

  console.log('PLC operations completed successfully!');
}

plcDemo().catch(err => {
  console.error('PLC Error:', err.message);
  process.exit(1);
});
```

### Reading Typed Values

```javascript
const { FinsClient, FinsMemoryArea, FinsDataType } = require('@omron-fins/native');

async function typedReadDemo() {
  const client = new FinsClient('192.168.1.250', 1, 0);

  // Read temperature as f32 (2 words)
  const temperature = await client.readF32(FinsMemoryArea.DM, 100);
  console.log(`Temperature: ${temperature.toFixed(2)}°C`);

  // Read counter as i32 (2 words)
  const counter = await client.readI32(FinsMemoryArea.DM, 102);
  console.log(`Counter: ${counter}`);

  // Read pressure as f64 (4 words)
  const pressure = await client.readF64(FinsMemoryArea.DM, 200);
  console.log(`Pressure: ${pressure.toFixed(3)} bar`);

  // Read struct
  const recipe = await client.readStruct(FinsMemoryArea.DM, 300, [
    FinsDataType.LINT,   // 8 bytes
    FinsDataType.INT,     // 2 bytes
    FinsDataType.REAL     // 4 bytes
  ]);
  console.log('Recipe struct:', recipe);

  // Read string (10 words = 20 characters)
  const productCode = await client.readString(FinsMemoryArea.DM, 400, 10);
  console.log(`Product Code: ${productCode}`);
}

typedReadDemo().catch(console.error);
```

### Error Recovery

```javascript
const { FinsClient } = require('@omron-fins/native');

class PlcConnection {
  constructor(host, maxRetries = 3) {
    this.host = host;
    this.maxRetries = maxRetries;
    this.client = null;
  }

  async connect() {
    for (let attempt = 1; attempt <= this.maxRetries; attempt++) {
      try {
        this.client = new FinsClient(this.host, 1, 0);
        // Test connection
        await this.client.read('DM', 0, 1);
        console.log(`Connected to PLC at ${this.host}`);
        return true;
      } catch (error) {
        console.warn(`Connection attempt ${attempt}/${this.maxRetries} failed: ${error.message}`);
        if (attempt < this.maxRetries) {
          await this.sleep(1000 * attempt); // Exponential backoff
        }
      }
    }
    throw new Error(`Failed to connect to PLC after ${this.maxRetries} attempts`);
  }

  async readWithRetry(area, address, count, retries = 2) {
    for (let attempt = 1; attempt <= retries; attempt++) {
      try {
        return await this.client.read(area, address, count);
      } catch (error) {
        if (attempt === retries) throw error;
        console.warn(`Read attempt ${attempt} failed, retrying...`);
        await this.sleep(500);
      }
    }
  }

  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

async function main() {
  const plc = new PlcConnection('192.168.1.250', 3);
  await plc.connect();

  const data = await plc.readWithRetry('DM', 100, 10);
  console.log('Data:', data);
}

main().catch(console.error);
```

---

## Build from Source

### Prerequisites

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y build-essential curl git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify Rust installation
rustc --version
cargo --version
```

### Build Commands

```bash
# Clone repository
git clone https://github.com/deviagomendes/omron-fins-rs.git
cd omron-fins-rs

# Build for current platform (glibc)
npm run build

# Build for specific target
cargo build --release --features napi --target x86_64-unknown-linux-gnu
cargo build --release --features napi --target aarch64-unknown-linux-gnu

# Build for musl (Alpine)
rustup target add x86_64-unknown-linux-musl
cargo build --release --features napi --target x86_64-unknown-linux-musl

# Output locations
# npm build: dist/
# cargo build: target/{target}/release/libomron_fins.so
```

### Verify Build

```bash
# Check built binary
file dist/*.node
# Output: dist/omron-fins-v0.6.0-x86_64-unknown-linux-gnu.node: ELF 64-bit LSB shared object

# Test loading
node -e "const m = require('./dist'); console.log('Loaded:', Object.keys(m))"

# Run version check (if available)
node -e "const { VERSION } = require('./dist'); console.log('Version:', VERSION)"
```

---

## Troubleshooting

### "Binary not found"

```bash
# Reinstall npm package
rm -rf node_modules package-lock.json
npm install

# Or manually link
npm link @omron-fins/native
```

### "GLIBC_2.X not found"

```bash
# Check glibc version
ldd --version

# If on Alpine, build from source
apk add --no-cache gcc musl-dev rust cargo
cargo build --release --features napi --target x86_64-unknown-linux-musl
```

### "Architecture mismatch"

```bash
# Check system architecture
uname -m
# x86_64 = Intel/AMD 64-bit
# aarch64 = ARM 64-bit

# Check Node.js architecture
node -p "process.arch + '-' + process.platform"
```

### Performance Issues

```javascript
// Use connection pooling for multiple operations
class PlcPool {
  constructor(host, size = 3) {
    this.pool = [];
    for (let i = 0; i < size; i++) {
      this.pool.push(new FinsClient(host, i + 1, 0));
    }
  }

  async acquire() {
    return this.pool.pop();
  }

  release(client) {
    this.pool.push(client);
  }

  async withClient(fn) {
    const client = this.acquire();
    try {
      return await fn(client);
    } finally {
      this.release(client);
    }
  }
}
```
