use std::{
    collections::HashMap, error::Error, fs::File, io::Read, path::{Path, PathBuf}, sync::{Arc, RwLock}
};

use glam::Vec3;
use rhai::{module_resolvers::FileModuleResolver, ASTFlags, Dynamic, Engine, EvalAltResult, EvalContext, Expr, Expression, FnPtr, Ident, ImmutableString, Module, NativeCallContext, RhaiNativeFunc, Stmt, Variant, AST};

use crate::pattern::{Part, Pattern, PatternError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    contents: String,
    file_path: Option<PathBuf>,
}

impl Script {
    pub fn new(contents: impl Into<String>) -> Self {
        Self {
            contents: contents.into(),
            file_path: None,
        }
    }

    pub fn new_with_path(contents: impl Into<String>, file_path: impl Into<PathBuf>) -> Self {
        Self {
            contents: contents.into(),
            file_path: Some(file_path.into()),
        }
    }

    pub fn load_file(path: &Path) -> std::io::Result<Self> {
        println!("{path:?}");
        let mut f = File::open(path)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;

        Ok(Self::new_with_path(s, path))
    }

    pub fn path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    pub fn set_path(&mut self, file_path: impl Into<PathBuf>) {
        self.file_path = Some(file_path.into());
    }

    pub fn source(&self) -> &str {
        &self.contents
    }

    pub fn source_mut(&mut self) -> &mut String {
        &mut self.contents
    }
}

impl<T> From<T> for Script
where T: Into<String> {
    fn from(value: T) -> Self {
        Script {
            contents: value.into(),
            file_path: None,
        }
    }
}

pub struct PatternScript;

impl PatternScript {
    pub fn create_compile_engine() -> rhai::Engine {
        let mut engine = Engine::new();

        engine
            .set_max_expr_depths(64, 64)
            .set_module_resolver(FileModuleResolver::new_with_extension("ph"))
            .register_type_with_name::<petgraph::graph::NodeIndex>("StitchMark")
            .register_custom_syntax(vec!["rep", "$expr$", "$expr$"], true, |context: &mut EvalContext, inputs: &[Expression]| {
                let count = context.eval_expression_tree(&inputs[0])?.as_int().map_err(|err| format!("Invalid count type: {err}"))?;
                
                for _ in 0..count {
                    let _ = context.eval_expression_tree(&inputs[1])?;
                }

                Ok(Dynamic::UNIT)
            })
            .unwrap();

        engine
    }

    pub fn create_engine(pattern: Arc<Pattern>, part: Arc<RwLock<Part>>) -> rhai::Engine {
        let mut engine = PatternScript::create_compile_engine();

        fn callback<F>(
            part: Arc<RwLock<Part>>,
            func: F,
        ) -> impl RhaiNativeFunc<(), 0, false, (), false>
        where
            F: Fn(&mut Part) + 'static + Send + Sync,
        {
            move || func(&mut part.write().unwrap())
        }

        fn callback_fallible<F, R>(
            part: Arc<RwLock<Part>>,
            func: F,
        ) -> impl RhaiNativeFunc<(), 0, false, R, true>
        where
            F: Fn(&mut Part) -> Result<R, PatternError> + 'static + Send + Sync,
            R: Clone + Send + Sync + 'static,
        {
            move || func(&mut part.write().unwrap()).map_err(|err| format!("{err}").into())
        }

        #[allow(deprecated)]
        engine
            .register_fn("new_part", {
                let part = part.clone();
                let pattern = pattern.clone();
                move || {
                    (*part.write().unwrap()) = pattern.add_part();
                }
            })
            .register_fn("turn", callback_fallible(part.clone(),    Part::turn))
            .register_fn("turn_", callback_fallible(part.clone(),   Part::turn_noskip))
            .register_fn("new_row", callback_fallible(part.clone(), Part::new_row))
            .register_fn("chain", callback_fallible(part.clone(),   Part::chain))
            .register_fn("dc", callback_fallible(part.clone(),      Part::dc))
            .register_fn("dc_", callback_fallible(part.clone(),     Part::dc_noskip))
            .register_fn("dec", callback_fallible(part.clone(),     Part::dec))
            .register_fn("skip", callback_fallible(part.clone(),    Part::skip))
            .register_fn("magic_ring", callback(part.clone(),       Part::magic_ring))
            .register_fn("mark", {
                let part = part.clone();
                move || part.read().unwrap().prev()
            })
            .register_fn("curr", {
                let part = part.clone();
                move || -> Result<_, Box<EvalAltResult>> { part.read().unwrap().insert().ok_or("No current insertion point".into()) }
            })
            .register_fn("row", {
                let part = part.clone();
                move || -> Result<Dynamic, Box<EvalAltResult>> {
                    part.read().unwrap().current_row()
                        .map(|v| v.clone().into())
                        .map_err(|err| format!("{err}").into())
                }
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
                    part.write().unwrap().start_ch_sp().map_err(|err| -> Box<EvalAltResult> { format!("{err}").into() })?;
                    func.call_within_context::<()>(&ctx, ())?;
                    let ch_sp = part.write().unwrap().end_ch_sp().map_err(|err| -> Box<EvalAltResult> { format!("{err}").into() })?;
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
                        .map_err(|err| format!("{err}").into())
                }
            })
            .register_fn("change_color", {
                let part = part.clone();
                move |color: rhai::Array| -> Result<(), Box<EvalAltResult>>{
                    let color = color.into_iter()
                        .map(|comp| comp.cast::<f64>() as f32)
                        .collect::<Vec<_>>();
                    if color.len() != 3 {
                        Err::<(), Box<EvalAltResult>>("Color should be in RGB format".into())?;
                    }
                    let color = Vec3::from_slice(&color);
                    part.write().unwrap().change_color(color);

                    Ok(())
                }
            });
            // .on_var(|name, _index, ctx| {
            //     let var = ctx.scope().get_value::<Dynamic>(name);
            //     if var.is_some() {
            //         Ok(None)
            //     } else {
            //         println!("couldn't find name: {name}");
            //         let func = FnPtr::new(name)?;
            //         Ok(Some(func.into()))
            //     }
            // });

        engine
    }

    pub fn get_script_exports(script: &Script) -> Result<Vec<(ImmutableString, Dynamic)>, Box<dyn Error + Send + Sync>> {
        let engine = PatternScript::create_compile_engine();
        let mut ast = engine.compile(&script.contents)?;

        let exports = ast.statements().iter()
            .filter_map(|stmt| -> Option<Result<(ImmutableString, Dynamic), Box<dyn Error + Send + Sync>>> {
                let mut stmt = stmt.clone();

                if let rhai::Stmt::Var(ref mut body, flags, position) = stmt {
                    if flags.contains(ASTFlags::EXPORTED) && !flags.contains(ASTFlags::CONSTANT) {
                        let value = body.1.get_literal_value().ok_or("Exported parameter must be a literal.".to_string().into());
                        Some(value.map(|v| (body.0.name.clone(), v)))
                    } else { None }
                } else { None }
            })
            .try_collect::<Vec<_>>()?;

        println!("{exports:?}");

        Ok(exports)
    }

    pub fn preprocess_script(script: &Script, exports: &HashMap<ImmutableString, Dynamic>) -> Result<AST, Box<dyn Error + Send + Sync>> {
        let engine = PatternScript::create_compile_engine();
        let mut ast = engine.compile(&script.contents)?;
        if let Some(path) = &script.file_path { ast.set_source(path.to_str().unwrap()); }

        let new_stmts = ast.statements().iter()
            .map(|stmt| -> Result<Stmt, Box<dyn Error + Send + Sync>> {
                let mut stmt = stmt.clone();

                if let rhai::Stmt::Var(ref mut body, flags, position) = stmt {
                    if flags.contains(ASTFlags::EXPORTED) && !flags.contains(ASTFlags::CONSTANT) {
                        if let Some(v) = exports.get(&body.0.name) {
                            body.1 = Expr::from_dynamic(v.clone(), body.1.position());
                        }
                    }
                }

                Ok(stmt)
            })
            .try_collect::<Vec<_>>()?;

        let new_ast = AST::new(new_stmts, Module::default())
            .merge(&ast.clone_functions_only());

        Ok(new_ast)
    }

    pub fn eval_script(script: &Script) -> Result<Pattern, Box<dyn Error + Send + Sync>> {
        let pattern = Pattern::new();
        let part = Arc::new(RwLock::new(pattern.add_part()));

        {
            let engine = PatternScript::create_engine(pattern.clone(), part.clone());
            let mut ast = engine.compile(&script.contents)?;
            if let Some(path) = &script.file_path { ast.set_source(path.to_str().unwrap()); }
            engine.run_ast(&ast)?
        }

        drop(part);
        Ok(pattern.into_inner())
    }

    pub fn eval_script_with_exports(script: &Script, exports: &HashMap<ImmutableString, Dynamic>) -> Result<Pattern, Box<dyn Error + Send + Sync>> {
        let pattern = Pattern::new();
        let part = Arc::new(RwLock::new(pattern.add_part()));

        {
            let engine = PatternScript::create_engine(pattern.clone(), part.clone());
            let ast = PatternScript::preprocess_script(script, exports)?;
            engine.run_ast(&ast)?
        }

        drop(part);
        Ok(pattern.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use crate::examples;

    use super::*;

    #[test]
    fn test_script() {
        let pattern = PatternScript::eval_script(
            &r#"
rep 15 chain();
rep 15 {
    turn();
    rep 15 dc();
}
        "#.into(),
        )
        .expect("Error in evaluating script");

        assert_eq!(pattern, crate::pattern::test_pattern_flat(15).unwrap());
    }

    #[test]
    fn test_all_examples() {
        examples::EXAMPLES.iter()
            .for_each(|&(name, path)| {
                let path = Path::new("../").join(Path::new(path));
                let script = Script::load_file(&path).unwrap();
                PatternScript::eval_script(&script)
                    .map_err(|err| format!("Error in evaluating example {name}: {err}"))
                    .unwrap();
            });
    }
}
