use std::rc::Rc;

use rhai::{Engine, Locked};

use crate::pattern::Pattern;

pub struct PatternScript;

impl PatternScript {
    pub fn eval_script(script: &str) -> Result<Pattern, Box<dyn std::error::Error>> {
        let pattern = Rc::new(Locked::new(Pattern::new()));

        {
            let mut engine = Engine::new();

            engine
                .register_fn("new_row", {
                    let pattern = pattern.clone();
                    move || pattern.borrow_mut().new_row()
                })
                .register_fn("chain", {
                    let pattern = pattern.clone();
                    move || pattern.borrow_mut().chain()
                })
                .register_fn("dc", {
                    let pattern = pattern.clone();
                    move || pattern.borrow_mut().dc()
                });

            let _ = engine.run(script)?;
        }

        Ok(Rc::try_unwrap(pattern).expect("pattern variable still in use?").into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script() {
        let pattern = PatternScript::eval_script(
            r#"
            new_row();
            for _c in 1..=15 {
                chain();
            }
            for _r in 1..=15 {
                new_row();
                for _s in 1..=15 {
                    dc();
                }
            }
        "#,
        )
        .expect("Error in evaluating script");

        assert_eq!(pattern, crate::pattern::test_pattern_flat());
    }
}
