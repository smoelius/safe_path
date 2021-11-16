use clippy_utils::{
    consts::{constant_context, Constant},
    diagnostics::span_lint_and_help,
    get_trait_def_id, match_def_path, match_path, path_to_res, paths,
    source::snippet_opt,
    ty::implements_trait,
};
use if_chain::if_chain;
use rustc_hir::{
    def::Res,
    def_id::{DefId, LocalDefId},
    Expr, ExprKind, HirId, ItemKind,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::{
    subst::{GenericArg, GenericArgKind},
    Ty, TyKind,
};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::{Span, Symbol};
use safe_path::PathOps;
use std::path::Path;

declare_lint! {
    /// **What it does:** Checks for calls to `Path::join` or `Utf8Path::join` with a non-constant
    /// path argument.
    ///
    /// **Why is this bad?** An attacker controlled path argument could lead to a directory
    /// traversal attack.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # use std::{env::current_dir, io::{Read, stdin}};
    /// # fn main() -> Result<()> {
    /// let mut filename = String::new();
    /// let _ = stdin().read_to_string(&mut filename)?;
    /// let path = current_dir().unwrap().join("lib").join(filename);
    /// let lib = unsafe { libloading::Library::new(path) }?;
    /// # Ok(())
    /// # }
    /// ```
    /// Use instead:
    /// ```no_run
    /// # use anyhow::Result;
    /// # use safe_path::SafePath;
    /// # use std::{env::current_dir, io::{Read, stdin}};
    /// # fn main() -> Result<()> {
    /// let mut filename = String::new();
    /// let _ = stdin().read_to_string(&mut filename)?;
    /// let path = current_dir().unwrap().join("lib").safe_join(filename)?;
    /// let lib = unsafe { libloading::Library::new(path) }?;
    /// # Ok(())
    /// # }
    /// ```
    pub SAFE_JOIN_OPPORTUNITY,
    Warn,
    "calls where `safe_join` or `relaxed_safe_join` could be used"
}

declare_lint! {
    /// **What it does:** Checks for calls to `Path::parent` or `Utf8Path::parent` with a
    /// non-constant `self` argument.
    ///
    /// **Why is this bad?** An attacker controlled `self` argument could lead to a directory
    /// traversal attack.
    ///
    /// **Known problems:** None.
    pub SAFE_PARENT_OPPORTUNITY,
    Warn,
    "calls where `safe_parent` or `relaxed_safe_parent` could be used"
}

declare_lint! {
    /// **What it does:** Checks for calls to `SafePath::safe_join` or `SafePath::relaxed_safe_join`
    /// that return an error when the receiver is not `/`.
    ///
    /// **Why is this bad?** Such behavior is likely not what the programmer intended. There are
    /// simpler ways to check whether the receiver is `/`, if this is what the programmer intended.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```
    /// # use safe_path::SafePath;
    /// # let dir = std::path::PathBuf::new();
    /// let path = dir.safe_join("..");
    /// ```
    /// Use instead:
    /// ```
    /// # let dir = std::path::PathBuf::new();
    /// let path = dir.join("..");
    /// ```
    pub SAFE_JOIN_MISAPPLICATION,
    Warn,
    "calls to `safe_join` or `relaxed_safe_join` that are likely erroneous"
}

declare_lint_pass!(SafePathLint => [SAFE_JOIN_OPPORTUNITY, SAFE_PARENT_OPPORTUNITY, SAFE_JOIN_MISAPPLICATION]);

const UTF8PATH_JOIN: [&str; 3] = ["camino", "Utf8Path", "join"];
const UTF8PATH_PARENT: [&str; 3] = ["camino", "Utf8Path", "parent"];
const SAFE_JOIN_TRAIT: [&str; 2] = ["safe_path", "SafePath"];
const RELAXED_SAFE_JOIN: [&str; 3] = ["safe_path", "SafePath", "relaxed_safe_join"];
const SAFE_JOIN: [&str; 3] = ["safe_path", "SafePath", "safe_join"];
const INTO_TRAIT: [&str; 3] = ["std", "convert", "Into"];
const IO_ERROR: [&str; 4] = ["std", "io", "error", "Error"];
const PATH_JOIN: [&str; 4] = ["std", "path", "Path", "join"];
const PATH_PARENT: [&str; 4] = ["std", "path", "Path", "parent"];

impl<'tcx> LateLintPass<'tcx> for SafePathLint {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &Expr<'_>) {
        if_chain! {
            if let ExprKind::MethodCall(method_name, method_span, args, _) = expr.kind;
            if let Some(method_def_id) = cx.typeck_results().type_dependent_def_id(expr.hir_id);
            let method_arg_span = expr.span.with_lo(method_span.lo());
            then {
                check_safe_join_opportunity(cx, expr, method_name.ident.name, method_span, args, method_def_id, method_arg_span);
                check_safe_parent_opportunity(cx, expr, method_name.ident.name, method_span, args, method_def_id, method_arg_span);
                check_safe_join_misapplication(cx, expr, method_name.ident.name, method_span, args, method_def_id, method_arg_span);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_safe_join_opportunity(
    cx: &LateContext<'_>,
    expr: &Expr<'_>,
    _method_name: Symbol,
    _method_span: Span,
    args: &[Expr<'_>],
    method_def_id: DefId,
    method_arg_span: Span,
) {
    if_chain! {
        if match_def_path(cx, method_def_id, &PATH_JOIN)
            || match_def_path(cx, method_def_id, &UTF8PATH_JOIN);
        if let [_, arg] = args;
        if constant_context(cx, cx.typeck_results())
            .expr(arg)
            .is_none();
        if let Some(arg_snippet) = snippet_opt(cx, arg.span);
        then {
            let module_local_def_id = cx.tcx.parent_module(expr.hir_id);
            let file_or_module = if module_local_def_id.is_top_level_module() {
                "file"
            } else {
                "module"
            };
            let import_msg = if module_uses_safe_path(cx, module_local_def_id) {
                String::new()
            } else {
                format!("add `use safe_path::SafePath;` to the {} and ", file_or_module)
            };
            if enclosing_body_can_return_io_error(cx, expr.hir_id) {
                span_lint_and_help(
                    cx,
                    SAFE_JOIN_OPPORTUNITY,
                    method_arg_span,
                    "join of a non-constant path",
                    None,
                    &format!(
                        "{}use `safe_join({})?` or `relaxed_safe_join({})?`",
                        import_msg, arg_snippet, arg_snippet
                    ),
                );
            } else {
                span_lint_and_help(
                    cx,
                    SAFE_JOIN_OPPORTUNITY,
                    method_arg_span,
                    "join of a non-constant path",
                    None,
                    &format!(
                        "{}adjust the surrounding code so that `safe_join({})?` or \
                        `relaxed_safe_join({})?` can be used",
                        import_msg, arg_snippet, arg_snippet
                    ),
                );
            };
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_safe_parent_opportunity(
    cx: &LateContext<'_>,
    expr: &Expr<'_>,
    _method_name: Symbol,
    _method_span: Span,
    args: &[Expr<'_>],
    method_def_id: DefId,
    method_arg_span: Span,
) {
    if_chain! {
        if match_def_path(cx, method_def_id, &PATH_PARENT)
            || match_def_path(cx, method_def_id, &UTF8PATH_PARENT);
        if let [arg] = args;
        if constant_context(cx, cx.typeck_results())
            .expr(arg)
            .is_none();
        then {
            let module_local_def_id = cx.tcx.parent_module(expr.hir_id);
            let file_or_module = if module_local_def_id.is_top_level_module() {
                "file"
            } else {
                "module"
            };
            let import_msg = if module_uses_safe_path(cx, module_local_def_id) {
                String::new()
            } else {
                format!("add `use safe_path::SafePath;` to the {} and ", file_or_module)
            };
            if enclosing_body_can_return_io_error(cx, expr.hir_id) {
                span_lint_and_help(
                    cx,
                    SAFE_JOIN_OPPORTUNITY,
                    method_arg_span,
                    "parent of a non-constant path",
                    None,
                    &format!(
                        "{}use `safe_parent()?` or `relaxed_safe_parent()?`",
                        import_msg
                    ),
                );
            } else {
                span_lint_and_help(
                    cx,
                    SAFE_JOIN_OPPORTUNITY,
                    method_arg_span,
                    "parent of a non-constant path",
                    None,
                    &format!(
                        "{}adjust the surrounding code so that `safe_parent()?` or \
                        `relaxed_safe_parent()?` can be used",
                        import_msg
                    ),
                );
            };
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_safe_join_misapplication(
    cx: &LateContext<'_>,
    _expr: &Expr<'_>,
    method_name: Symbol,
    method_span: Span,
    args: &[Expr<'_>],
    method_def_id: DefId,
    method_arg_span: Span,
) {
    let relaxed = match_def_path(cx, method_def_id, &RELAXED_SAFE_JOIN);
    if_chain! {
        if relaxed || match_def_path(cx, method_def_id, &SAFE_JOIN);
        if let [_, arg] = args;
        if let Some(Constant::Str(path)) = constant_context(cx, cx.typeck_results()).expr(arg);
        if Path::new(".").check_join_safety(path.as_ref(), relaxed).is_err();
        then {
            let (always_msg, root_msg) = if Path::new("/").check_join_safety(path.as_ref(), relaxed).is_err() {
                (" always", "")
            } else {
                ("", " if the receiver is not `/`")
            };
            span_lint_and_help(
                cx,
                SAFE_JOIN_MISAPPLICATION,
                method_arg_span,
                &format!("this call to `{}` will{} return an error{}", method_name, always_msg, root_msg),
                Some(method_span),
                "if such behavior is not intended, use `join`",
            );
        }
    }
}

fn module_uses_safe_path(cx: &LateContext<'_>, local_def_id: LocalDefId) -> bool {
    let module_items = cx.tcx.hir_module_items(local_def_id);
    module_items.items.iter().any(|item_id| {
        let item = cx.tcx.hir().item(*item_id);
        if let ItemKind::Use(path, _) = item.kind {
            match_path(path, &SAFE_JOIN_TRAIT)
        } else {
            false
        }
    })
}

fn enclosing_body_can_return_io_error(cx: &LateContext<'_>, hir_id: HirId) -> bool {
    let body_owner_hir_id = cx.tcx.hir().enclosing_body_owner(hir_id);
    let body_id = cx.tcx.hir().body_owned_by(body_owner_hir_id);
    let body = cx.tcx.hir().body(body_id);
    let body_ty = cx.typeck_results().expr_ty(&body.value);
    if_chain! {
        if let TyKind::Adt(adt_def, substs_ref) = &body_ty.kind();
        if match_def_path(cx, adt_def.did, &paths::RESULT);
        if let [_, generic_arg] = substs_ref.iter().collect::<Vec<_>>().as_slice();
        if let GenericArgKind::Type(error_ty) = generic_arg.unpack();
        if let Some(into_trait_id) = get_trait_def_id(cx, &INTO_TRAIT);
        if let Some(io_error_ty) = path_to_ty(cx, &IO_ERROR);
        if implements_trait(cx, io_error_ty, into_trait_id, &[GenericArg::from(error_ty)]);
        then {
            true
        } else {
            false
        }
    }
}

fn path_to_ty<'tcx>(cx: &LateContext<'tcx>, path: &[&str]) -> Option<Ty<'tcx>> {
    let res = path_to_res(cx, path);
    if let Res::Def(_, def_id) = res {
        let adt_def = cx.tcx.adt_def(def_id);
        let substs = cx.tcx.mk_substs(std::iter::empty::<GenericArg<'_>>());
        let ty = cx.tcx.mk_adt(adt_def, substs);
        Some(ty)
    } else {
        None
    }
}
