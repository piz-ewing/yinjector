$env:RUSTFLAGS="-C link-arg=/DEBUG:NONE"
$env:RUSTFLAGS=$env:RUSTFLAGS+" "+"--remap-path-prefix $HOME=~"

cargo b --target=i686-pc-windows-msvc --release
cargo b --target=x86_64-pc-windows-msvc --release

cp target\i686-pc-windows-msvc\release\yinjector.exe release\yinjector32.exe
cp target\x86_64-pc-windows-msvc\release\yinjector.exe release\yinjector64.exe
cp config.toml release\config.toml
cp -Force -Recurse dlls release\