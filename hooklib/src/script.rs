use rhai::{Dynamic, Engine, EvalAltResult, FnPtr, Locked, NativeCallContext, RhaiNativeFunc, Shared};

use crate::pattern::Pattern;

pub struct PatternScript;

pub enum PatternInstructions {
    Chain,
    Dc,
    Seq(Box<PatternInstructions>, Box<PatternInstructions>),
    Repeat(u32, Box<PatternInstructions>),
}

impl PatternScript {
    pub fn eval_script(script: &str) -> Result<Pattern, Box<dyn std::error::Error>> {
        let pattern = Shared::new(Locked::new(Pattern::new()));

        {
            let mut engine = Engine::new();

            fn callback<F>(
                pattern: Shared<Locked<Pattern>>,
                func: F,
            ) -> impl RhaiNativeFunc<(), 0, false, (), false>
            where
                F: Fn(&mut Pattern) -> () + 'static,
            {
                move || func(&mut *pattern.borrow_mut())
            }

            #[allow(deprecated)]
            engine
                .register_custom_operator("#", 160).unwrap()
                .register_type_with_name::<petgraph::graph::NodeIndex>("StitchMark")
                .register_fn("#", |ctx: NativeCallContext, times: i64, func: FnPtr| -> Result<(), Box<EvalAltResult>> {
                    for _ in 1..=times {
                        func.call_within_context::<()>(&ctx, ())?;
                    }
                    Ok(())
                })
                .register_fn("turn", callback(pattern.clone(), Pattern::turn))
                .register_fn("chain", callback(pattern.clone(), Pattern::chain))
                .register_fn("dc", callback(pattern.clone(), Pattern::dc))
                .register_fn("dc_", callback(pattern.clone(), Pattern::dc_noskip))
                .register_fn("mark", {
                    let pattern = pattern.clone();
                    move || pattern.borrow().prev()
                })
                .register_fn("ss", {
                    let pattern = pattern.clone();
                    move |into: petgraph::graph::NodeIndex| pattern.borrow_mut().slip_stitch(into)
                })
                .register_fn("into", {
                    let pattern = pattern.clone();
                    move |into: petgraph::graph::NodeIndex| pattern.borrow_mut().set_insert(into)
                })
                .register_fn("chain_space", {
                    let pattern = pattern.clone();
                    move |ctx: NativeCallContext, func: FnPtr| -> Result<petgraph::graph::NodeIndex, Box<EvalAltResult>> {
                        { pattern.borrow_mut().start_ch_sp(); }
                        func.call_within_context::<()>(&ctx, ())?;
                        let ch_sp = { pattern.borrow_mut().end_ch_sp() };
                        Ok(ch_sp)
                    }
                })
                .on_var(|name, _index, ctx| {
                    let var = ctx.scope().get_value::<Dynamic>(name);
                    if let Some(var) = var {
                        Ok(Some(var))
                    } else {
                        let func = FnPtr::new(name)?;
                        Ok(Some(func.into()))
                    }
                });

            let _ = engine.run(script)?;
        }

        Ok(Shared::try_unwrap(pattern)
            .expect("pattern variable still in use?")
            .into_inner())
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

        assert_eq!(pattern, crate::pattern::test_pattern_flat());
    }
}
