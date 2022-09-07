use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("target/rvv")?;

    let output = std::process::Command::new("rvv-as")
        .args(["src/rvv_crypto/zz_preload.S"])
        .output()?;
    let mut f = std::fs::File::create("target/rvv/zz_preload.S")?;
    f.write(&output.stdout)?;
    cc::Build::new()
        .compiler(format!(
            "{}/bin/riscv64-unknown-elf-gcc",
            std::env::var("RISCV").unwrap()
        ))
        .target("elf64-littleriscv")
        .file("target/rvv/zz_preload.S")
        .compile("zz_preload");

    Ok(())
}
