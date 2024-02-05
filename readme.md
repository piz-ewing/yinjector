## windows-injector
[![license](https://img.shields.io/badge/license-MIT-yellow.svg?style=flat)](https://github.com/piz-ewing/injector/blob/main/LICENSE)
![Language](https://img.shields.io/badge/language-rust-brightgreen)

- ✨ Fusion injector
- 👍 Easy to configure
- 🚅 Automatically monitor processes

## build
```bash
# windows-x86
cargo b --target=i686-pc-windows-msvc

# windows-x64
cargo b --target=x86_64-pc-windows-msvc

# for release
$env:RUSTFLAGS="--remap-path-prefix $HOME=~"
```

## config
```toml
[global]
# monitor interval 50ms
monitor_interval = 50

# use native method to inject
native = false

# exit after injection
exit_on_injected = false

[base]
"a.exe" = 'a.dll'
"b.exe" = '../b.dll'
"c.exe" = 'c:\c.dll'

"x86.exe" = '.\dlls\x86\msg.dll'
"x64.exe" = '.\dlls\x64\msg.dll'

# execute when module exists
[module]
"x86.exe" = "user32.dll"

# execute when window title exists
[window]
"x86.exe" = "window title"

# deferred x seconds execution, 5000 ms
[delay]
"x86.exe" = 5000

```
## run

```
./injector.exe [config_path]
```

![demo](./demo.png)

## todo
- ✅ ~~Merge x86 and x64 injector~~

- 📝 Better way for merge x86 and x64 injector

- ⌨️ [More ways to inject](https://github.com/HackerajOfficial/injectAllTheThings)

- ~~⌨️ organize 'window' injection code~~

- ~~⌨️ organize 'module' injection code~~

- ~~⌨️ organize 'delay' injection code~~

- ❌ [bug] setting multiple targets

- need to fix priority between modes

## ref

***thx***

[YAPI -- Yet Another Process Injector](https://github.com/ez8-co/yapi.git) @ez8-co

[pretty-env-logger](https://github.com/seanmonstar/pretty-env-logger.git) @seanmonstar

[remove absolute paths in release binary](https://users.rust-lang.org/t/how-to-remove-absolute-paths-in-release-binary/75969)

[windows-win-rs](https://github.com/DoumanAsh/windows-win-rs.git)
