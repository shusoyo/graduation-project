fn main() {
    println!("cargo:rustc-check-cfg=cfg(verus_keep_ghost)");
    println!("cargo:rustc-check-cfg=cfg(verus_keep_ghost_body)");
    println!("cargo:rustc-check-cfg=cfg(test)");
}
