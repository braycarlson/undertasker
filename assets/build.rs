#![cfg(windows)]
#![windows_subsystem = "windows"]

extern crate winres;

fn main() {
    let config = slint_build::CompilerConfiguration::new().with_style("fluent-dark".into());
    slint_build::compile_with_config("ui/ui.slint", config).unwrap();

    let mut resource = winres::WindowsResource::new();
    resource
        .set_icon_with_id("assets/logo.ico", "1000")
        .set("InternalName", "undertasker");
    resource.compile().expect("Resource could not be compiled.");
}
