use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::File,
    io::{self, Write as _},
    path::Path,
};

use const_format::formatcp;

use super::{compile_cache_tag, PREFIX};
use crate::ir::{to_display, CacheTag};

use super::binary_search::bin_search;

pub(super) const REG_MEM_PTR: &str = formatcp!("{PREFIX}_MemoryPointer");

pub struct MemoryMaker<'a> {
    pub functions_dir: &'a Path,
    pub mem_size: u32,
    pub cache_size: u32,
    pub used_cache_regs: &'a HashSet<u32>,
    pub static_caches: &'a HashMap<CacheTag<'a>, i32>,
    pub word_width: u32,
}

pub fn compile_store_func(chunks: u32) -> impl Display {
    to_display(move |f| write!(f, "{PREFIX}_MemoryStore_Chunks{chunks}",))
}

pub fn compile_load_func(chunks: u32) -> impl Display {
    to_display(move |f| write!(f, "{PREFIX}_MemoryLoad_Chunks{}", chunks))
}

impl MemoryMaker<'_> {
    pub fn mem_swap_func(&self) -> io::Result<()> {
        let mem_chunk_count = self.mem_size.div_ceil(self.word_width);
        let cache_chunk_count = self.cache_size.div_ceil(self.word_width);

        for chunk_count in 1..=cache_chunk_count {
            let init = |is_store| {
                let namespace = if is_store {
                    compile_store_func(chunk_count).to_string()
                } else {
                    compile_load_func(chunk_count).to_string()
                };

                bin_search(
                    self.functions_dir,
                    mem_chunk_count,
                    &namespace,
                    REG_MEM_PTR,
                    |index, file| {
                        for (cache_unit, mem_unit) in (index * self.word_width
                            ..(index + chunk_count) * self.word_width)
                            .enumerate()
                        {
                            let mem_unit = compile_mem_unit(mem_unit);
                            let cache_unit = compile_cache_tag(CacheTag::Regular(cache_unit as _));
                            let (dst, src): (&dyn Display, &dyn Display) = if is_store {
                                (&mem_unit, &cache_unit)
                            } else {
                                (&cache_unit, &mem_unit)
                            };

                            writeln!(file, "scoreboard players set MCSH {dst} = MCSH {src}",)?;
                        }
                        Ok(())
                    },
                )
            };

            init(false)?;
            init(true)?;
        }

        Ok(())
    }

    pub fn mem_bootstrap(&self) -> io::Result<()> {
        let mut file = File::create(self.functions_dir.join("mcsh_bootstrap.mcfunction"))?;

        writeln!(file, "scoreboard players reset MCSH")?;

        for x in 0..self.mem_size {
            writeln!(file, "{}", register_object(compile_mem_unit(x)))?;
        }

        for x in (0..self.cache_size).chain(
            self.used_cache_regs
                .iter()
                .copied()
                .filter(|x| *x >= self.cache_size),
        ) {
            writeln!(
                file,
                "{}",
                register_object(compile_cache_tag(CacheTag::Regular(x)))
            )?;
        }

        for (key, value) in self.static_caches.iter() {
            let key = compile_cache_tag(*key);
            let reg = register_object(&key);

            writeln!(
                file,
                "{reg}\n\
                scoreboard players set MCSH {key} {value}",
            )?;
        }

        writeln!(file, "{}", register_object(REG_MEM_PTR))
    }
}

fn compile_mem_unit(position: u32) -> impl Display {
    to_display(move |f| write!(f, "{PREFIX}_MemoryUnit_{position}"))
}

fn register_object(item: impl Display) -> impl Display {
    to_display(move |f| write!(f, "scoreboard objectives add {item} dummy"))
}
