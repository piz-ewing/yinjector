## windows-injector

## build
```bash
# windows-x86
cargo b --target=i686-pc-windows-msvc

# windows-x64
cargo b --target=x86_64-pc-windows-msvc
```

## config
```toml
[global]
monitor_interval = 50

[injector]
"a.exe" = 'b.dll'
"b.exe" = '../c.dll'
"c.exe" = 'c:/1.dll'
"x86.exe"='x86.dll'
"x64.exe"='x64.dll'

```
## run

```
./injector.exe [config_path]
```

## todo
[x] merge x86 and x64 injector

## ref

***Maybe I'll modify it, so I don't import using subprojects***

[YAPI -- Yet Another Process Injector](https://github.com/ez8-co/yapi.git) @ez8-co

[pretty-env-logger](https://github.com/seanmonstar/pretty-env-logger.git) @seanmonstar
