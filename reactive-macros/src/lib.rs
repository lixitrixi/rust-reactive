use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprPath, Ident, parse_macro_input, visit::Visit};

/// Stores any variable identifiers found during traversal
struct IdentCollector {
    idents: Vec<Ident>,
}

impl<'ast> Visit<'ast> for IdentCollector {
    fn visit_expr_path(&mut self, i: &'ast ExprPath) {
        if i.path.segments.len() == 1 {
            self.idents.push(i.path.segments[0].ident.clone());
        }
        // syn::visit::visit_expr_path(self, i);
    }
}

#[proc_macro]
pub fn rv(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let mut collector = IdentCollector { idents: Vec::new() };
    collector.visit_expr(&expr);

    let idents = collector.idents;
    let output = quote! {::rust_reactive::Field::new((#(&#idents),*), |#(#idents),*| #expr)};

    TokenStream::from(output)
}
