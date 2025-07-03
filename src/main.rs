use codex::{Binding, Def, ModifierSet, Symbol};

#[allow(clippy::type_complexity)]
struct FlatModuleIter {
    stack: Vec<(String, Box<dyn Iterator<Item = (&'static str, Binding)>>)>,
}

impl Iterator for FlatModuleIter {
    type Item = (String, Symbol);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (prefix, iter) = self.stack.last_mut()?;
            let Some((name, binding)) = iter.next() else {
                self.stack.pop();
                continue;
            };
            let name = prefix.clone() + if !prefix.is_empty() { "." } else { "" } + name;
            match binding.def {
                Def::Symbol(s) => return Some((name, s)),
                Def::Module(m) => {
                    self.stack.push((name, Box::new(Box::leak(Box::new(m)).iter())))
                }
            }
        }
    }
}

fn main() {
    use std::io::Write;
    let iter = FlatModuleIter {
        stack: vec![("".into(), Box::new(codex::ROOT.iter()))],
    };
    for (name, sym) in iter {
        let max_modifs = sym.variants().map(|(m, ..)| m.iter().count()).max().unwrap();
        let modifs = sym.modifiers().collect::<Vec<_>>();
        let n = modifs.len();
        eprint!("{:30}\r{name} {n} {max_modifs}\r", "");
        std::io::stderr().flush().unwrap();
        for k in 0..=max_modifs {
            let mut indices = (0..k).collect::<Vec<_>>();
            loop {
                let mset = indices.iter().map(|i| modifs[*i]).fold(
                    ModifierSet::<String>::default(),
                    |mut res, m| {
                        res.insert_raw(m);
                        res
                    },
                );

                let old_val = sym.old_get(mset.as_deref());
                let new_val = sym.get(mset.as_deref());

                if old_val != new_val {
                    let dot = if mset.is_empty() { "" } else { "." };
                    println!("{name}{dot}{} : {old_val:?} => {new_val:?}", mset.as_str());
                }

                if next_subseq(&mut indices, n.saturating_sub(1)) {
                    break;
                }
            }
        }
    }
}

// e.g.: [0,1,2], [0,1,3], [0,1,4], [0,2,3], [0,2,4], [0,3,4], [1,2,3], [1,2,4], [1,3,4], [2,3,4]
fn next_subseq(indices: &mut [usize], max_index: usize) -> bool {
    // invariant: indices.len() <= max_index + 1
    match indices {
        [] => true,
        [single] => {
            if *single < max_index {
                *single += 1;
                false
            } else {
                true
            }
        }
        [left @ .., last] => {
            assert_ne!(max_index, 0);
            if *last < max_index {
                *last += 1;
                false
            } else {
                let res = next_subseq(left, max_index - 1);
                *last = left.last().copied().map_or(*last, |x| x + 1);
                res
            }
        }
    }
}
