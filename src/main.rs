use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use ir::{simulate::SimulateResult, LabelMap};
use parse::parse_file;

use crate::atoi::Atoi;

mod atoi;
mod ir;
mod parse;

#[derive(Parser, Debug)]
#[command(name = "MCSH")]
#[command(about = "MCSH编译器：将mcsh源代码文件编译为mcfunction文件")]
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

    #[command(about = "编译文件并快速安装到游戏开发目录，用于调试使用")]
    Dev { clear: bool },

    #[command(alias = "b", about = "编译文件")]
    Build {
        #[arg(long, short, help = "编译文件保存路径")]
        out: Option<PathBuf>,

        #[arg(
            short,
            long,
            help = "在编译结果附加manifest.json文件。\
                详细信息将启动命令行进行交互式信息输入。"
        )]
        manifest: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    dbg!(&cli);
    let file = fs::read_to_string(&cli.input)?;
    let defs = parse_file(&file)?;
    let mut atoi = Atoi::new();
    atoi.parse(&defs)?;
    let label_map = atoi.finish();

    match cli.command {
        Command::Simulate { function } => start_simulation(&label_map, &function),
        Command::Build { out, manifest } => build(&label_map, out.as_deref(), &cli.input, manifest),
        Command::Dev { clear } => todo!(),
    }
}

fn build(lm: &LabelMap, out: Option<&Path>, file_path: &Path, with_manifest: bool) -> Result<()> {
    let out_dir: Cow<Path> = match out {
        Some(o) => o.into(),
        None => {
            let Some(parent_dir) = file_path.parent() else {
                return Err(anyhow!("output directory must be specified with this file"));
            };

            parent_dir.join("mcsh_out").into()
        }
    };

    let functions_dir = out_dir.join("functions");
    if functions_dir.exists() {
        fs::remove_dir_all(&functions_dir)?;
    }
    fs::create_dir_all(&out_dir)?;

    lm.compile(&functions_dir)?;
    Ok(())
}

fn start_simulation(lm: &LabelMap, fn_name: &str) -> Result<()> {
    let SimulateResult { result, log } = lm.simulate_pub(fn_name);
    println!("日志：\n{log}");
    println!("运行结果：{result:?}");
    Ok(())
}
