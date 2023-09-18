fn main() {
    // only build the resource for release builds
    // as calling rc.exe might be slow
    // if std::env::var("PROFILE").unwrap() == "release" {
    let mut res = winres::WindowsResource::new();
    res
        // .set_icon("resources\\ico\\fiscalidade_server.ico")
        .set_manifest(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#,
        );

    if let Err(error) = res.compile() {
        eprint!("{}", error);
        std::process::exit(1);
    }
    // }
}
