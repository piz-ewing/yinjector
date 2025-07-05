$env:RUSTFLAGS="-C link-arg=/DEBUG:NONE"
$env:RUSTFLAGS=$env:RUSTFLAGS+" "+"--remap-path-prefix $HOME=~"

# Ensure the release directory exists
New-Item -ItemType Directory -Path release -Force
New-Item -ItemType Directory -Path release\dlls -Force


# Build 32-bit version (injector and test DLL)
cargo build --target i686-pc-windows-msvc --release -p yinjector
cargo build --target i686-pc-windows-msvc --release -p test_dll

# Build 64-bit version (injector and test DLL)
cargo build --target x86_64-pc-windows-msvc --release -p yinjector
cargo build --target x86_64-pc-windows-msvc --release -p test_dll

# Copy 32-bit and 64-bit executables
Copy-Item -Path target\i686-pc-windows-msvc\release\yinjector.exe -Destination release\yinjector32.exe -Force
Copy-Item -Path target\x86_64-pc-windows-msvc\release\yinjector.exe -Destination release\yinjector64.exe -Force

# Copy 32-bit and 64-bit test DLLs
Copy-Item -Path target\i686-pc-windows-msvc\release\test_dll.dll -Destination release\dlls\x86.dll -Force
Copy-Item -Path target\x86_64-pc-windows-msvc\release\test_dll.dll -Destination release\dlls\x64.dll -Force

# Copy the configuration file
Copy-Item -Path config.toml -Destination release\config.toml -Force
