//! Windows: exe simgesini (assets/icon.ico) windres ile gömer.
//! windres yoksa sessizce geçilir (simgesiz derlenir, hata vermez).

fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=assets/app.rc");
        println!("cargo:rerun-if-changed=assets/icon.ico");
        let out = std::env::var("OUT_DIR").unwrap_or_else(|_| ".".into());
        let obje = format!("{out}/uygulama_simgesi.o");
        let derleme = std::process::Command::new("windres")
            .args(["--input", "assets/app.rc", "--output", &obje, "--output-format", "coff"])
            .status();
        match derleme {
            Ok(st) if st.success() => {
                println!("cargo:rustc-link-arg={obje}");
            }
            _ => {
                println!("cargo:warning=windres calismadi; exe simgesiz olacak");
            }
        }
    }
}
