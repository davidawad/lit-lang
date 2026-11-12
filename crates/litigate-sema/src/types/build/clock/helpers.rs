fn chain_expr(deadline: &str, end: DueEnd) -> crate::hir::Expr {
    let call = crate::hir::Expr {
        ty: Type::Unit,
        kind: crate::hir::ExprKind::Call {
            callee: "deadline".to_owned(),
            args: vec![crate::hir::Expr {
                ty: Type::Unit,
                kind: crate::hir::ExprKind::Ref {
                    target: deadline.to_owned(),
                },
                span: None,
            }],
        },
        span: None,
    };
    crate::hir::Expr {
        ty: Type::Date,
        kind: crate::hir::ExprKind::Field {
            base: Box::new(call),
            field: end.field().to_owned(),
        },
        span: None,
    }
}

/// The arguments of `e` when it is a `deadline(...)` call.
fn deadline_call(e: &Expr) -> Option<&Vec<Expr>> {
    match &e.kind {
        ExprKind::Call(f, args) if f.display() == "deadline" => Some(args),
        _ => None,
    }
}

/// The source text of `deadline(...)`'s arguments, for a fix.
fn args_text(args: &[Expr]) -> String {
    args.iter()
        .map(|a| match &a.kind {
            ExprKind::Ref(r) => r.display(),
            _ => "d".to_owned(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
