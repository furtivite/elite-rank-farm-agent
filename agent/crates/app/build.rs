fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=../../assets/app.ico");

    #[cfg(windows)]
    {
        let result = embed_resource::compile("app.rc", embed_resource::NONE);
        if let Err(error) = result.manifest_optional() {
            println!("cargo:warning=ресурсы не встроены в .exe (значок и манифест): {error}");
        }
    }
}
