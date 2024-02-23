use std::{
    fmt::Display,
    fs::{self, File},
    io::{self, Write as _},
    path::{Path, PathBuf},
};

use crate::ir::{to_display, REG_MATCH_ENABLED};

pub fn bin_search<F>(
    functions_dir: &Path,
    arms: &[i32],
    namespace: &str,
    pointer_reg: &str,
    is_simple: bool,
    file_content: F,
) -> io::Result<()>
where
    F: Fn(Option<i32>, &mut File) -> io::Result<()>,
{
    BinSearch::new(
        functions_dir,
        arms,
        namespace,
        pointer_reg,
        file_content,
        is_simple,
    )
    .entry_file()
}

struct BinSearch<'a, F1> {
    path_prefix: PathBuf,
    file_content: F1,
    arms: &'a [i32],
    namespace: &'a str,
    pointer_reg: &'a str,
    is_simple: bool,
}

impl<'a, F1> BinSearch<'a, F1>
where
    F1: Fn(Option<i32>, &mut File) -> io::Result<()>,
{
    fn new(
        functions_dir: &Path,
        arms: &'a [i32],
        namespace: &'a str,
        pointer_reg: &'a str,
        file_content: F1,
        is_simple: bool,
    ) -> Self {
        let mut path_prefix = functions_dir.to_path_buf();
        path_prefix.push("MCSH");
        path_prefix.push(namespace);

        Self {
            path_prefix,
            file_content,
            arms,
            pointer_reg,
            namespace,
            is_simple,
        }
    }

    fn entry_file(&self) -> io::Result<()> {
        let Self {
            namespace,
            pointer_reg,
            arms,
            path_prefix,
            is_simple,
            ..
        } = self;

        let mut file_path = path_prefix.clone();
        file_path.set_extension("mcfunction");
        let mut entry_file = File::create(file_path)?;
        fs::create_dir(path_prefix)?;
        let default_file = self.default_file()?;

        // 确定取值范围
        let (Some(first), Some(last)) = (arms.first(), arms.last()) else {
            writeln!(entry_file, "function MCSH/{namespace}/{default_file}")?;
            return Ok(());
        };

        let start_search_func = self.branch_file(arms)?;

        if !is_simple {
            writeln!(
                entry_file,
                "scoreboard players set MCSH {REG_MATCH_ENABLED} 1"
            )?;
        }

        writeln!(
            entry_file,
            "execute if score MCSH {pointer_reg} matches {first}..{last} run function MCSH/{namespace}/{start_search_func}"
        )?;

        if *is_simple {
            writeln!(
                entry_file,
                "execute unless score MCSH {pointer_reg} matches {first}..{last} run \
                function MCSH/{namespace}/{default_file}"
            )?;
        } else {
            let check_match_enabled = self.check_match_enabled();
            writeln!(
                entry_file,
                "execute {check_match_enabled}run \
                    function MCSH/{namespace}/{default_file}"
            )?;
        }

        Ok(())
    }

    fn check_match_enabled(&self) -> impl Display {
        let is_simple = self.is_simple;
        to_display(move |f| {
            if is_simple {
                Ok(())
            } else {
                write!(f, "if score MCSH {REG_MATCH_ENABLED} matches 1 ")
            }
        })
    }

    fn stop_match(&self, file: &mut File) -> io::Result<()> {
        if !self.is_simple {
            writeln!(file, "scoreboard players set MCSH {REG_MATCH_ENABLED} 0")
        } else {
            Ok(())
        }
    }

    fn default_file(&self) -> io::Result<&'static str> {
        let mcfn = "Default";
        let mut file = File::create(self.path_prefix.join("Default.mcfunction"))?;
        self.stop_match(&mut file)?;
        (self.file_content)(None, &mut file)?;
        Ok(mcfn)
    }

    fn branch_file(&self, arms: &[i32]) -> io::Result<String> {
        let Self {
            namespace,
            pointer_reg,
            ..
        } = self;

        let create_file = |mcfn| {
            File::create({
                let mut file_path = self.path_prefix.join(&mcfn);
                file_path.set_extension("mcfunction");
                file_path
            })
        };

        match arms {
            [] => unreachable!(),
            [one] => {
                let mcfn = format!("Leaf{}", one);
                let mut file = create_file(&mcfn)?;
                self.stop_match(&mut file)?;
                (self.file_content)(Some(*one), &mut file)?;
                Ok(mcfn)
            }
            [first_el, .., last_el] => {
                let mcfn = format!("Branch{first_el}_{last_el}");
                let mut file = create_file(&mcfn)?;
                let (arms1, arms2) = arms.split_at(arms.len() / 2);

                let file_name1 = self.branch_file(arms1)?;
                let file_name2 = self.branch_file(arms2)?;
                let check_match_enabled = self.check_match_enabled();

                writeln!(
                    file,
                    "execute {check_match_enabled}if score MCSH {pointer_reg} matches {first_el}..{0} run \
                        function MCSH/{namespace}/{file_name1}\n\
                    execute {check_match_enabled}if score MCSH {pointer_reg} matches {1}..{last_el} run \
                        function MCSH/{namespace}/{file_name2}",
                    arms1.last().unwrap(), arms2.first().unwrap()
                )?;

                Ok(mcfn)
            }
        }
    }
}
