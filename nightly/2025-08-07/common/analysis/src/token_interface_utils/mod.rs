extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_hir::FnRetTy;
use rustc_hir::QPath;
use rustc_hir::Ty;
use rustc_hir::TyKind;

// Used to check if the parameters for a function match the data types for a specific token interface function.
pub fn check_params(
    fn_params: &[Ty],
    expected_types: Vec<String>,
    fn_return: FnRetTy,
    expected_return: Option<String>,
) -> bool {
    let mut param_types: Vec<String> = Vec::new();
    for i in fn_params {
        if let TyKind::Path(QPath::Resolved(_, path)) = i.kind {
            let param_type = path.segments[0].ident.to_string();
            param_types.push(param_type.clone());
        }
    }
    if expected_return.is_none() {
        if let FnRetTy::DefaultReturn(_) = fn_return {
            return param_types == expected_types;
        }
    } else {
        if let FnRetTy::Return(ty) = fn_return {
            if let TyKind::Path(qpath) = &ty.kind {
                if let QPath::Resolved(_, path) = qpath {
                    if let Some(first_segment) = path.segments.first() {
                        return first_segment.ident.to_string() == expected_return.unwrap()
                            && param_types == expected_types;
                    }
                }
            }
        }
    }
    false
}

// Used to verify if a function matches a token interface standard function.
pub fn verify_token_interface_function(
    fn_name: String,
    fn_params: &[Ty],
    fn_return: FnRetTy,
) -> bool {
    let function = fn_name.split("::").last().unwrap();
    let (types, expected_return): (Vec<String>, Option<String>) = match function {
        "allowance" => (
            ["Env", "Address", "Address"]
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            Some("i128".to_string()),
        ),
        "approve" => (
            ["Env", "Address", "Address", "i128", "u32"]
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            None,
        ),
        "balance" => (
            ["Env", "Address"].iter().map(|&s| s.to_string()).collect(),
            Some("i128".to_string()),
        ),
        "transfer" => (
            ["Env", "Address", "Address", "i128"]
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            None,
        ),
        "transfer_from" => (
            ["Env", "Address", "Address", "Address", "i128"]
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            None,
        ),
        "burn" => (
            ["Env", "Address", "i128"]
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            None,
        ),
        "burn_from" => (
            ["Env", "Address", "Address", "i128"]
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            None,
        ),
        "decimals" => (
            ["Env"].iter().map(|&s| s.to_string()).collect(),
            Some("u32".to_string()),
        ),
        "name" => (
            ["Env"].iter().map(|&s| s.to_string()).collect(),
            Some("String".to_string()),
        ),
        "symbol" => (
            ["Env"].iter().map(|&s| s.to_string()).collect(),
            Some("String".to_string()),
        ),
        _ => return false,
    };
    check_params(fn_params, types, fn_return, expected_return)
}
