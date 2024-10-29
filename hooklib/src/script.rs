use rhai::{Dynamic, Engine, FnPtr, Locked, NativeCallContext, RhaiNativeFunc, Shared};

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
                .register_custom_operator("#", 160)
                .unwrap()
                .register_fn("#", |ctx: NativeCallContext, times: i64, func: FnPtr| {
                    for _ in 1..=times {
                        func.call_within_context::<()>(&ctx, ()).unwrap();
                    }
                })
                .register_fn("turn", callback(pattern.clone(), Pattern::turn))
                .register_fn("chain", callback(pattern.clone(), Pattern::chain))
                .register_fn("dc", callback(pattern.clone(), Pattern::dc))
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
