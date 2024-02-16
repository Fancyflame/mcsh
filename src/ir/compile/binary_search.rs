use std::{
    fs::{self, File},
    io::{self, Write as _},
    ops::Range,
    path::{Path, PathBuf},
};

pub fn bin_search<F>(
    functions_dir: &Path,
    size: u32,
    namespace: &str,
    pointer_reg: &str,
    file_content: F,
) -> io::Result<()>
where
    F: Fn(u32, &mut File) -> io::Result<()>,
{
    BinSearch::new(functions_dir, size, namespace, pointer_reg, file_content).entry_file()
}

struct BinSearch<'a, F1> {
    path_prefix: PathBuf,
    file_content: F1,
    size: u32,
    namespace: &'a str,
    pointer_reg: &'a str,
}

impl<'a, F1> BinSearch<'a, F1>
where
    F1: Fn(u32, &mut File) -> io::Result<()>,
{
    fn new(
        functions_dir: &Path,
        size: u32,
        namespace: &'a str,
        pointer_reg: &'a str,
        file_content: F1,
    ) -> Self {
        let mut path_prefix = functions_dir.to_path_buf();
        path_prefix.push("MCSH");
        path_prefix.push(namespace);

        Self {
            path_prefix,
            file_content,
            size,
            pointer_reg,
            namespace,
        }
    }

    fn entry_file(&self) -> io::Result<()> {
        let mut file_path = self.path_prefix.clone();
        file_path.set_extension("mcfunction");
        let mut file = File::create(file_path)?;

        if self.size == 0 {
            return writeln!(file, "say MCSH: pointer out of range");
        }

        if !self.path_prefix.exists() {
            fs::create_dir(&self.path_prefix)?;
        }

        let start_search_func = self.branch_file(0..self.size)?;
        let Self {
            namespace,
            pointer_reg,
            size,
            ..
        } = self;
        let lb = *size - 1;

        writeln!(
            file,
            "execute unless score MCSH {pointer_reg} matches 0..{lb} run say MCSH: pointer out of range\n\
            execute if score MCSH {pointer_reg} matches 0..{lb} run function MCSH/{namespace}/{start_search_func}"
        )
    }

    fn branch_file(&self, range: Range<u32>) -> io::Result<String> {
        match range.len() {
            0 => unreachable!(),
            1 => {
                let mcfn = format!("Leaf{}.mcfunction", range.start);
                let mut file = File::create(self.path_prefix.join(&mcfn))?;

                (self.file_content)(range.start, &mut file)?;
                Ok(mcfn)
            }
            _ => {
                let center = (range.end + range.start) / 2;
                let lb = center - 1;

                let mcfn = format!("Branch{center}.mcfunction");
                let mut file = File::create(self.path_prefix.join(&mcfn))?;

                let file_name1 = self.branch_file(range.start..center)?;
                let file_name2 = self.branch_file(center..range.end)?;
                let Self {
                    namespace,
                    pointer_reg,
                    ..
                } = self;
                writeln!(
                    file,
                    "execute if score MCSH {pointer_reg} matches ..{lb} run function MCSH/{namespace}/{file_name1}\n\
                    execute if score MCSH {pointer_reg} matches {center}.. run function MCSH/{namespace}/{file_name2}"
                )?;
                Ok(mcfn)
            }
        }
    }
}
