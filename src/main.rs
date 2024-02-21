use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use ir::{simulate::SimulateResult, LabelMap};
use manifest::McManifest;
use parse::parse_file;

use crate::atoi::Atoi;

mod atoi;
mod format;
mod ir;
mod manifest;
mod parse;

#[derive(Parser, Debug)]
#[command(
    name = "MCSH",
    author = "FancyFlame<fancyflame@163.com>",
    about = "MCSH编译器：将mcsh源代码文件编译为mcfunction文件"
)]
struct Cli {
    #[arg(help = "输入源文件")]
    input: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "在指定函数上运行指令仿真")]
    Simulate { function: String },

    // #[command(about = "编译文件并快速安装到游戏开发目录，用于调试使用")]
    // Dev { clear: bool },
    #[command(alias = "b", about = "编译文件")]
    Build(BuildArgs),
}

#[derive(Args, Debug)]
struct BuildArgs {
    #[arg(long, short, help = "编译文件保存路径")]
    out: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "在编译结果附加manifest.json文件。\
            详细信息将启动命令行进行交互式信息输入。"
    )]
    manifest: bool,

    #[arg(long, help = "编译结果打包为mcpack", requires = "manifest")]
    mcpack: bool,

    #[arg(long, help = "在编译结果附加图标文件")]
    pack_icon: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let file = fs::read_to_string(&cli.input)?;
    let defs = parse_file(&file)?;
    let mut atoi = Atoi::new();
    atoi.parse(&defs)?;
    let label_map = atoi.finish();

    match cli.command {
        Command::Simulate { function } => start_simulation(&label_map, &function),
        Command::Build(args) => build(&label_map, &cli.input, args),
    }
}

fn build(
    lm: &LabelMap,
    file_path: &Path,
    BuildArgs {
        out,
        manifest,
        mcpack,
        pack_icon,
    }: BuildArgs,
) -> Result<()> {
    let out_dir = match out {
        Some(o) => o,
        None => {
            let Some(parent_dir) = file_path.parent() else {
                return Err(anyhow!("output directory must be specified with this file"));
            };
            parent_dir.join("mcsh_out")
        }
    };

    if !out_dir.exists() {
        fs::create_dir_all(&out_dir)?;
    }

    let work_dir = if mcpack {
        dirs::cache_dir()
            .as_ref()
            .unwrap_or(&out_dir)
            .join("mcsh_compile_cache")
    } else {
        out_dir.clone()
    };

    if !work_dir.exists() {
        fs::create_dir_all(&work_dir)?;
    }

    if manifest {
        let manifest_file = work_dir.join("manifest.json");
        if manifest_file.exists() && !mcpack {
            return Err(anyhow!(
                "已存在manifest.json文件，请妥善处理后重试或\
                    关闭生成manifest.json选项"
            ));
        }
        fs::write(manifest_file, McManifest::interact()?.to_json())?;
    }

    if let Some(pack_icon) = pack_icon {
        let ext = pack_icon.extension();
        let mut pack_icon_dst = work_dir.join("pack_icon");
        if let Some(ext) = ext {
            pack_icon_dst.set_extension(ext);
        }
        fs::copy(pack_icon, pack_icon_dst)?;
    }

    let functions_dir = work_dir.join("functions");
    if functions_dir.exists() {
        fs::remove_dir_all(&functions_dir)?;
    }
    fs::create_dir_all(&work_dir)?;
    lm.compile(&functions_dir)?;

    if mcpack {
        let mut out_file = out_dir.join(file_path.file_name().unwrap());
        out_file.set_extension("mcpack");
        zip_extensions::zip_create_from_directory(&out_file, &work_dir)?;
        fs::remove_dir_all(&work_dir)?;
    }

    Ok(())
}

fn start_simulation(lm: &LabelMap, fn_name: &str) -> Result<()> {
    let SimulateResult { result, log } = lm.simulate_pub(fn_name);
    println!("日志：\n{log}");
    println!("运行结果：{result:?}");
    Ok(())
}
