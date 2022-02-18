use anyhow::Context;
use serde::{Deserialize, Serialize};
use swc_plugin::{ast::*, plugin_transform, syntax_pos::DUMMY_SP};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    ignore: Vec<JsWord>,

    #[serde(default = "default_prefix_pattern")]
    prefix_pattern: String,

    #[serde(default)]
    pub filename: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginContext {
    #[serde(default)]
    pub filename: Option<String>,
}

fn default_prefix_pattern() -> String {
    "[filename]".to_owned()
}

impl Default for Config {
    fn default() -> Self {
        serde_json::from_str("{}").unwrap()
    }
}

struct TransformVisitor {
    config: Config,
}

impl TransformVisitor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    fn ignore_console_method(&self, id: &Ident) -> bool {
        &*id.sym == "table" || self.config.ignore.contains(&id.sym)
    }

    fn generate_prefix(&self) -> JsWord {
        let mut prefix = self.config.prefix_pattern.to_owned();

        if prefix.contains("[filename]") {
            prefix = prefix.replace(
                "[filename]",
                self.config.filename.as_deref().unwrap_or_default(),
            );
        }

        JsWord::from(prefix)
    }

    fn add_prefix_to_log(&self, call_expr: &mut CallExpr) {
        let prefix = self.generate_prefix();

        if !prefix.is_empty() {
            call_expr.args.insert(
                0,
                ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        has_escape: false,
                        kind: StrKind::Synthesized,
                        value: prefix,
                    }))),
                },
            );
        }
    }
}

impl VisitMut for TransformVisitor {
    noop_visit_mut_type!();

    fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
        call_expr.visit_mut_children_with(self);
        if let Callee::Expr(expr) = &call_expr.callee {
            if let Expr::Member(member_expr) = &**expr {
                if let (Expr::Ident(obj_id), MemberProp::Ident(prop_id)) =
                    (&*member_expr.obj, &member_expr.prop)
                {
                    if &*obj_id.sym == "console" && !self.ignore_console_method(&prop_id) {
                        self.add_prefix_to_log(call_expr);
                    }
                }
            }
        }
    }
}

/// An entrypoint to the SWC's transform plugin.
/// `plugin_transform` macro handles necessary interop to communicate with the host,
/// and entrypoint function name (`process_transform`) can be anything else.
///
/// If plugin need to handle low-level ptr directly,
/// it is possible to opt out from macro by writing transform fn manually via raw interface
///
/// `__plugin_process_impl(
///     ast_ptr: *const u8,
///     ast_ptr_len: i32,
///     config_str_ptr: *const u8,
///     config_str_ptr_len: i32) ->
///     i32 /*  0 for success, fail otherwise.
///             Note this is only for internal pointer interop result,
///             not actual transform result */
///
/// However, this means plugin author need to handle all of serialization/deserialization
/// steps with communicating with host. Refer `swc_plugin_macro` for more details.
#[plugin_transform]
pub fn process_transform(program: Program, plugin_config: String, context: String) -> Program {
    let mut config: Config = serde_json::from_str(&plugin_config)
        .context("failed to parse plugin config")
        .unwrap();

    let context: PluginContext = serde_json::from_str(&context)
        .context("failed to parse plugin context")
        .unwrap();

    if config.filename.is_none() {
        config.filename = context.filename;
    }

    program.fold_with(&mut as_folder(TransformVisitor::new(config)))
}

#[cfg(test)]
mod transform_visitor_tests {
    use swc_ecma_transforms_testing::test;

    use super::*;

    fn transform_visitor(config: Config) -> impl 'static + Fold + VisitMut {
        as_folder(TransformVisitor::new(config))
    }

    test!(
        ::swc_ecma_parser::Syntax::default(),
        |_| transform_visitor(Config {
            filename: Some("test.js".to_owned()),
            ..Default::default()
        }),
        adds_default_prefix_when_filename_is_some,
        r#"console.log("hello world");"#,
        r#"console.log("test.js", "hello world");"#
    );

    test!(
        ::swc_ecma_parser::Syntax::default(),
        |_| transform_visitor(Default::default()),
        doesnt_add_default_prefix_when_filename_is_none,
        r#"console.log("hello world");"#,
        r#"console.log("hello world");"#
    );

    test!(
        ::swc_ecma_parser::Syntax::default(),
        |_| transform_visitor(Config {
            prefix_pattern: "custom-prefix:".to_owned(),
            ..Default::default()
        }),
        adds_custom_prefix_to_console_logs,
        r#"console.log("hello world");"#,
        r#"console.log("custom-prefix:", "hello world");"#
    );

    test!(
        ::swc_ecma_parser::Syntax::default(),
        |_| transform_visitor(Default::default()),
        does_not_alter_console_table,
        r#"console.table(["apples", "oranges", "bananas"]);"#,
        r#"console.table(["apples", "oranges", "bananas"]);"#
    );

    test!(
        ::swc_ecma_parser::Syntax::default(),
        |_| transform_visitor(Config {
            ignore: vec![JsWord::from("log")],
            ..Default::default()
        }),
        ignores_console_members,
        r#"console.log("hello world");"#,
        r#"console.log("hello world");"#
    );

    test!(
        ::swc_ecma_parser::Syntax::default(),
        |_| transform_visitor(Config {
            prefix_pattern: "file: [filename]".to_owned(),
            filename: Some("test.js".to_owned()),
            ..Default::default()
        }),
        adds_filename,
        r#"console.log("hello world");"#,
        r#"console.log("file: test.js", "hello world");"#
    );
}
