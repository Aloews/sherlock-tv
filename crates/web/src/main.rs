#[cfg(target_arch = "wasm32")]
fn main() {
    sherlock_web::mount_app();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
