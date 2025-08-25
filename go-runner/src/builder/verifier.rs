use gosyn::ast::FuncDecl;

use crate::prelude::*;

pub struct FuncVisitor {
    param_name: String,
    func_decl: gosyn::ast::FuncDecl,
}

impl FuncVisitor {
    pub fn verify_source_code(source: &str, benchmarks: &[String]) -> anyhow::Result<Vec<String>> {
        let file = gosyn::parse_source(source)?;
        Self::verify_file(&file, benchmarks)
    }

    pub fn verify_file(
        file: &gosyn::ast::File,
        benchmarks: &[String],
    ) -> anyhow::Result<Vec<String>> {
        let mut valid_benchmarks = Vec::new();
        for decl in &file.decl {
            let gosyn::ast::Declaration::Function(func_decl) = decl else {
                continue;
            };
            let func_name = &func_decl.name.name;
            if !benchmarks.contains(func_name) {
                continue;
            }

            if FuncVisitor::new(func_decl.clone())?.is_valid().is_err() {
                continue;
            };

            valid_benchmarks.push(func_name.clone());
        }

        Ok(valid_benchmarks)
    }

    pub fn new(func: FuncDecl) -> anyhow::Result<Self> {
        let param_name = Self::find_testing_b_param_name(&func)?;

        Ok(Self {
            param_name,
            func_decl: func,
        })
    }

    fn find_testing_b_param_name(func_decl: &gosyn::ast::FuncDecl) -> Result<String> {
        let params = &func_decl.typ.params;
        for param in &params.list {
            let type_expr = &param.typ;

            let gosyn::ast::Expression::TypePointer(pointer_type) = type_expr else {
                continue;
            };
            let gosyn::ast::Expression::Selector(selector) = pointer_type.typ.as_ref() else {
                continue;
            };

            let gosyn::ast::Expression::Ident(pkg) = selector.x.as_ref() else {
                continue;
            };

            // We need a testing.B parameter
            if pkg.name != "testing" || selector.sel.name != "B" {
                continue;
            }

            if let Some(first_name) = param.name.first() {
                return Ok(first_name.name.clone());
            }
        }

        bail!("Benchmark function does not have *testing.B parameter");
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        let Some(body) = &self.func_decl.body else {
            return Ok(());
        };

        self.is_valid_block(body)
    }

    fn is_valid_block(&self, block: &gosyn::ast::BlockStmt) -> anyhow::Result<()> {
        for stmt in &block.list {
            self.is_valid_stmt(stmt)?;
        }
        Ok(())
    }

    fn is_valid_stmt(&self, stmt: &gosyn::ast::Statement) -> anyhow::Result<()> {
        match stmt {
            gosyn::ast::Statement::Expr(expr_stmt) => self.is_valid_expr(&expr_stmt.expr),
            gosyn::ast::Statement::Assign(assign_stmt) => {
                for expr in &assign_stmt.right {
                    self.is_valid_expr(expr)?;
                }
                Ok(())
            }
            gosyn::ast::Statement::If(if_stmt) => {
                self.is_valid_expr(&if_stmt.cond)?;
                self.is_valid_block(&if_stmt.body)?;
                if let Some(else_stmt) = &if_stmt.else_ {
                    self.is_valid_stmt(else_stmt)?;
                }
                Ok(())
            }
            gosyn::ast::Statement::For(for_stmt) => {
                if let Some(condition) = &for_stmt.cond {
                    self.is_valid_stmt(condition)?;
                }
                if let Some(init) = &for_stmt.init {
                    self.is_valid_stmt(init)?;
                }
                if let Some(post) = &for_stmt.post {
                    self.is_valid_stmt(post)?;
                }
                self.is_valid_block(&for_stmt.body)
            }
            gosyn::ast::Statement::Block(block_stmt) => self.is_valid_block(block_stmt),
            _ => Ok(()),
        }
    }

    fn is_valid_expr(&self, expr: &gosyn::ast::Expression) -> anyhow::Result<()> {
        match expr {
            gosyn::ast::Expression::Call(call_expr) => {
                if let gosyn::ast::Expression::Selector(_) = call_expr.func.as_ref() {
                    for arg in &call_expr.args {
                        if self.uses_testing_ident(arg) {
                            bail!(
                                "testing.B parameter '{}' passed as argument to method call",
                                self.param_name
                            );
                        }
                    }
                } else {
                    for arg in &call_expr.args {
                        if self.uses_testing_ident(arg) {
                            bail!(
                                "testing.B parameter '{}' passed as argument to function call",
                                self.param_name
                            );
                        }
                    }
                }

                for arg in &call_expr.args {
                    self.is_valid_expr(arg)?;
                }
            }
            gosyn::ast::Expression::Operation(operation) => {
                self.is_valid_expr(&operation.x)?;
                if let Some(y) = &operation.y {
                    self.is_valid_expr(y)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn uses_testing_ident(&self, expr: &gosyn::ast::Expression) -> bool {
        if let gosyn::ast::Expression::Ident(ident) = expr {
            ident.name == self.param_name
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_benchmark() {
        let source = include_str!("../../testdata/verifier/valid_benchmark.go");
        let valid_benches =
            FuncVisitor::verify_source_code(source, &["BenchmarkValid".into()]).unwrap();

        assert_eq!(valid_benches.len(), 1);
        assert!(valid_benches.contains(&"BenchmarkValid".to_string()));
    }

    #[test]
    fn test_invalid_benchmark_function_call() {
        let source = include_str!("../../testdata/verifier/invalid_benchmark_function_call.go");
        let valid_benches =
            FuncVisitor::verify_source_code(source, &["BenchmarkInvalid".into()]).unwrap();

        assert!(valid_benches.is_empty());
    }

    #[test]
    fn test_valid_benchmark_method_calls() {
        let source = include_str!("../../testdata/verifier/valid_benchmark_methods.go");
        let valid_benches =
            FuncVisitor::verify_source_code(source, &["BenchmarkValidMethods".into()]).unwrap();

        assert_eq!(valid_benches.len(), 1);
        assert!(valid_benches.contains(&"BenchmarkValidMethods".to_string()));
    }

    #[test]
    fn test_multiple_benchmarks_mixed_validity() {
        let source = include_str!("../../testdata/verifier/mixed_validity_benchmarks.go");
        let valid_benches = FuncVisitor::verify_source_code(
            source,
            &["BenchmarkValid".into(), "BenchmarkInvalid".into()],
        )
        .unwrap();

        assert_eq!(valid_benches.len(), 1);
        assert!(valid_benches.contains(&"BenchmarkValid".to_string()));
    }
}
