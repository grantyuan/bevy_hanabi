use syn::{Expr, visit_mut::{VisitMut, self}, Lit, LitInt, parse_quote, LitFloat};

pub struct NumberReplace;

impl VisitMut for NumberReplace {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        if let Expr::Lit(expr) = &node {
            match &expr.lit {                
                Lit::Int(vl) => {
                    let mut digits = vl.base10_digits().to_string();
                    if vl.suffix() == "f32" || vl.suffix() =="f64" {
                        digits = digits + ".0";
                        let unsuffixed: LitFloat = syn::parse_str(digits.as_str()).unwrap();
                        *node = parse_quote!(#unsuffixed);
                    }else{
                        let unsuffixed: LitInt = syn::parse_str(digits.as_str()).unwrap();
                        *node = parse_quote!(#unsuffixed);
                    }
                   
                },
                Lit::Float(vl) => {
                    let mut digits = vl.base10_digits().to_string();
                    if !digits.contains(".") {
                        digits = digits + ".0";
                    }
                    let unsuffixed: LitFloat = syn::parse_str(digits.as_str()).unwrap();
                    *node = parse_quote!(#unsuffixed);
                },
                _=>(),
            };
        }

        // Delegate to the default impl to visit nested expressions.
        visit_mut::visit_expr_mut(self, node);
    }
}



#[cfg(test)]
mod test {
    use quote::*;
    use syn::visit_mut::VisitMut;

    use crate::wgsl_syntex_tools::NumberReplace;
    
    #[test]
    fn test_marco() {
        let mut defined = quote!();
       
        let values = 0..3;
        let x = 32.0f32;
        
        values
            .enumerate()
            .for_each(|f| {
                let index = f.0;
                let value = f.1*2;
                let ident_v = format_ident!("{}", "val");
               
                let mut qu = quote! {
                    if (#ident_v >= #index && #ident_v < #value){
                        println("value inside {},{},{} {}",#index,#value,#ident_v, #x);
                    }
                };

                if index < 2 {
                    qu.append_all(quote!{
                        else
                    });
                }
                defined.append_all(qu);                
            });

        let mut syntex_tree = syn::parse2(defined).unwrap();
        NumberReplace.visit_expr_mut(&mut syntex_tree);
     
        let code = quote!(#syntex_tree);
        println!("{}", &code);
        assert!(!code.to_string().contains("usize"));
    }
}

