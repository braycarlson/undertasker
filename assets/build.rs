#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut resource = winres::WindowsResource::new();
    resource.set_icon("assets/undertasker.ico");
    resource.compile().expect("Resource could not be compiled.");
}
