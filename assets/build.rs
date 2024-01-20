#![cfg(windows)]
#![windows_subsystem = "windows"]

extern crate winres;

fn main() {
    let mut resource = winres::WindowsResource::new();
    resource
        .set_icon_with_id("assets/logo.ico", "1000")
        .set("InternalName", "undertasker");
    resource.compile().expect("Resource could not be compiled.");
}
