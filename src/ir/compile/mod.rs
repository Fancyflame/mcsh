use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use super::{CacheTag, Ir, Label, LabelMap, PREFIX};
use anyhow::{anyhow, Result};
use memory::*;
use miscellaneous::*;

macro_rules! display_write {
    ($($tt:tt)*) => {
        $crate::ir::to_display(|formatter| write!(formatter, $($tt)*))
    };
}

mod binary_search;
mod memory;
mod miscellaneous;

impl LabelMap<'_> {
    pub fn compile(&self, functions_dir: &Path) -> Result<()> {
        let mcsh_dir = functions_dir.join("MCSH");
        if !mcsh_dir.exists() {
            fs::create_dir_all(mcsh_dir)?;
        }

        if self.mem_size % self.word_width != 0 {
            return Err(anyhow!(
                "the memory size ({}) is not a multiple of the word width ({})",
                self.mem_size,
                self.word_width
            ));
        }

        let mut cache_set = HashSet::new();
        let mut cache_size = 0;

        //let optimized_label_map = dead_label_delete(&self.label_map)?;

        for (label, info) in &self.label_map {
            compile_one_label(
                functions_dir,
                &mut cache_size,
                &mut cache_set,
                *label,
                &info.insts,
            )?;
        }

        let mem_maker = MemoryMaker {
            functions_dir,
            used_cache_regs: &cache_set,
            mem_size: self.mem_size,
            cache_size,
            static_caches: &self.static_map,
            word_width: self.word_width,
        };
        mem_maker.mem_swap_func()?;
        mem_maker.mem_bootstrap()?;

        Ok(())
    }
}

/*fn dead_label_delete<'a>(
    label_map: &HashMap<Label<'a>, LabelInfo<'a>>,
) -> Result<HashMap<Label<'a>, LabelInfo<'a>>> {
    fn get_dep_tree<'a>(out: &mut HashMap<Label<'a>, LabelInfo<'a>>, insts: &Vec<Ir>) {
        for inst in insts {
            let Ir::Call { label } = inst else {
                continue;
            };

            let Some(value) = label_map.get(label) else {
                return Err(anyhow!(
                    "label `{}` was referenced but not defined",
                    compile_label(label, false)
                ));
            };

            map.entry(*label).or_insert_with(|| value.clone());
        }
    }
    let mut map = HashMap::new();

    for LabelInfo { insts, label } in label_map.values() {
        if !matches!(label, Label::Named { export: true, .. }) {
            continue;
        }
    }

    Ok(map)
}*/

fn compile_one_label(
    functions_dir: &Path,
    cache_size: &mut u32,
    cache_set: &mut HashSet<u32>,
    label: Label,
    insts: &Vec<Ir>,
) -> io::Result<()> {
    let mut file_path = functions_dir.to_path_buf();
    if let Label::Named { export: false, .. } | Label::Anonymous(_) = label {
        file_path.push("MCSH");
    }
    file_path.push(compile_label(&label, false).to_string());
    file_path.set_extension("mcfunction");
    let mut file = File::create(file_path)?;

    for inst in insts {
        match inst {
            Ir::Load { size, .. } | Ir::Store { size, .. } => {
                *cache_size = (*cache_size).max(*size)
            }
            Ir::Assign { dst, .. } | Ir::Operation { dst, .. } | Ir::BoolOperation { dst, .. } => {
                if let CacheTag::Regular(id) = dst {
                    cache_set.insert(*id);
                }
            }
            _ => {}
        }

        write!(file, "{}", compile_ir(inst))?;
    }

    Ok(())
}
