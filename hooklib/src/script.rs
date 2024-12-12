use std::{
    error::Error,
    sync::{Arc, RwLock},
};

use rhai::{Dynamic, Engine, EvalAltResult, FnPtr, NativeCallContext, RhaiNativeFunc};

use crate::pattern::{Part, Pattern};

pub struct PatternScript;

impl PatternScript {
    pub fn eval_script(script: &str) -> Result<Pattern, Box<dyn Error + Send + Sync>> {
        let pattern = Pattern::new();
        let part = Arc::new(RwLock::new(pattern.add_part()));

        {
            let mut engine = Engine::new();

            fn callback<F>(
                part: Arc<RwLock<Part>>,
                func: F,
            ) -> impl RhaiNativeFunc<(), 0, false, (), false>
            where
                F: Fn(&mut Part) + 'static + Send + Sync,
            {
                move || func(&mut part.write().unwrap())
            }

            #[allow(deprecated)]
            engine
                .register_custom_operator("#", 160).unwrap()
                .register_custom_operator("@", 160).unwrap()
                .register_type_with_name::<petgraph::graph::NodeIndex>("StitchMark")
                .register_fn("#", |ctx: NativeCallContext, times: i64, func: FnPtr| -> Result<(), Box<EvalAltResult>> {
                    for _ in 1..=times {
                        func.call_within_context::<()>(&ctx, ())?;
                    }
                    Ok(())
                })
                .register_fn("@", |ctx: NativeCallContext, times: i64, func: FnPtr| -> Result<(), Box<EvalAltResult>> {
                    for i in 1..=times {
                        func.call_within_context::<()>(&ctx, (i,))?;
                    }
                    Ok(())
                })
                .register_fn("new_part", {
                    let part = part.clone();
                    let pattern = pattern.clone();
                    move || {
                        (*part.write().unwrap()) = pattern.add_part();
                    }
                })
                .register_fn("turn", callback(part.clone(),    Part::turn))
                .register_fn("turn_", callback(part.clone(),   Part::turn_noskip))
                .register_fn("new_row", callback(part.clone(), Part::new_row))
                .register_fn("chain", callback(part.clone(),   Part::chain))
                .register_fn("dc", callback(part.clone(),      Part::dc))
                .register_fn("dc_", callback(part.clone(),     Part::dc_noskip))
                .register_fn("dec", callback(part.clone(),     Part::dec))
                .register_fn("skip", callback(part.clone(),    Part::skip))
                .register_fn("mark", {
                    let part = part.clone();
                    move || part.read().unwrap().prev()
                })
                .register_fn("curr", {
                    let part = part.clone();
                    move || -> Result<_, Box<EvalAltResult>> { part.read().unwrap().insert().ok_or("No current insertion point".into()) }
                })
                .register_fn("ss", {
                    let part = part.clone();
                    move |into: petgraph::graph::NodeIndex| part.write().unwrap().slip_stitch(into)
                })
                .register_fn("into", {
                    let part = part.clone();
                    move |into: petgraph::graph::NodeIndex| part.write().unwrap().set_insert(into)
                })
                .register_fn("chain_space", {
                    let part = part.clone();
                    move |ctx: NativeCallContext, func: FnPtr| -> Result<petgraph::graph::NodeIndex, Box<EvalAltResult>> {
                        { part.write().unwrap().start_ch_sp(); }
                        func.call_within_context::<()>(&ctx, ())?;
                        let ch_sp = { part.write().unwrap().end_ch_sp() };
                        Ok(ch_sp)
                    }
                })
                .register_fn("ignore", {
                    let part = part.clone();
                    move |ctx: NativeCallContext, func: FnPtr| -> Result<(), Box<EvalAltResult>> {
                        { part.write().unwrap().set_ignore(true); }
                        func.call_within_context::<()>(&ctx, ())?;
                        { part.write().unwrap().set_ignore(false); }
                        Ok(())
                    }
                })
                .register_fn("sew", {
                    let pattern = pattern.clone();
                    move |row_1: rhai::Array, row_2: rhai::Array| -> Result<(), Box<EvalAltResult>> {
                        let row_1: Vec<petgraph::graph::NodeIndex> = row_1.into_iter()
                            .map(|d| d.try_cast().ok_or("Sew argument not a node index"))
                            .collect::<Result<Vec<_>, _>>()?;
                        let row_2: Vec<petgraph::graph::NodeIndex> = row_2.into_iter()
                            .map(|d| d.try_cast().ok_or("Sew argument not a node index"))
                            .collect::<Result<Vec<_>, _>>()?;
                        pattern.sew(row_1, row_2)
                            .map_err(|err| err.into())
                    }
                })
                .on_var(|name, _index, ctx| {
                    let var = ctx.scope().get_value::<Dynamic>(name);
                    if var.is_some() {
                        Ok(None)
                    } else {
                        let func = FnPtr::new(name)?;
                        Ok(Some(func.into()))
                    }
                });

            engine.run(script)?
        }

        drop(part);

        Ok(pattern.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script() {
        let pattern = PatternScript::eval_script(
            r#"
15 # chain;
15 # || {
    turn();
    15 # dc;
}
        "#,
        )
        .expect("Error in evaluating script");

        assert_eq!(pattern, crate::pattern::test_pattern_flat(15));
    }
}
