use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("target/rvv")?;

    let file_list = vec![
        "zz_add_indexed",
        "zz_add",
        "zz_mul_indexed",
        "zz_mul_scalar",
        "zz_mul",
        "zz_neg",
        "zz_normalize",
        "zz_preload",
        "zz_sqr",
        "zz_sub",
    ];
    for name in file_list {
        let output = std::process::Command::new("rvv-as")
            .args([format!("src/rvv_crypto/{}.S", name)])
            .output()?;
        let mut f = std::fs::File::create(format!("target/rvv/{}.S", name))?;
        f.write(&output.stdout)?;
        cc::Build::new()
            .compiler(format!(
                "{}/bin/riscv64-unknown-elf-gcc",
                std::env::var("RISCV").unwrap()
            ))
            .target("elf64-littleriscv")
            .file(format!("target/rvv/{}.S", name))
            .compile(name);
    }

    Ok(())
}
