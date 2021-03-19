use proc_macro::TokenStream;
use quote::*;

#[proc_macro_derive(Layers)]
pub fn layers(input: TokenStream) -> TokenStream {
    let syn::DeriveInput { ident, data, .. } = syn::parse_macro_input!(input);
    let data_enum = match data {
        syn::Data::Struct(s) => {
            return syn::Error::new_spanned(s.struct_token, "expected enum")
                .into_compile_error()
                .into();
        }
        syn::Data::Union(u) => {
            return syn::Error::new_spanned(u.union_token, "expected enum")
                .into_compile_error()
                .into();
        }
        syn::Data::Enum(e) => e,
    };
    if let Some(e) = data_enum
        .variants
        .iter()
        .filter_map(|v| match &v.fields {
            syn::Fields::Unit => None,
            syn::Fields::Unnamed(f) if f.unnamed.len() == 1 => None,
            _ => Some(syn::Error::new_spanned(
                &v.ident,
                "expected empty variant or another nested Layers enum",
            )),
        })
        .fold(None, |err_opt, next_err| {
            if let Some(mut e) = err_opt {
                syn::Error::combine(&mut e, next_err);
                Some(e)
            } else {
                Some(next_err)
            }
        })
    {
        return e.into_compile_error().into();
    }

    let count = data_enum.variants.len();
    let u_ty = data_enum.variants.iter().filter_map(|v| {
        if let syn::Fields::Unnamed(u) = &v.fields {
            let ty = &u.unnamed[0].ty;
            Some(ty)
        } else {
            None
        }
    });
    
    let ty = u_ty.clone();
    let variants_root_check = quote! {
        trait IsLayer {
            fn variant_cannot_derive_from_root() {}
        }
        impl<T: trait_layers::Layers> IsLayer for T {}
        trait IsRoot {
            fn variant_cannot_derive_from_root() {}
        }
        impl<T: trait_layers::Root> IsRoot for T {}
        #(<#ty>::variant_cannot_derive_from_root();)*
    };
    let root_check = data_enum
        .variants
        .first()
        .map(|v| match &v.fields {
            syn::Fields::Unnamed(_) => quote! {
                fn type_check<R: trait_layers::Root>() {}
                type_check::<#ident>();
                #variants_root_check
            },
            _ => variants_root_check,
        })
        .unwrap_or(quote!{});
    
    let mut u_add = quote! { 0 };
    let u_add = data_enum.variants.iter().map(|v| {
        let add = u_add.clone();
        if let syn::Fields::Unnamed(u) = &v.fields {
            let ty = &u.unnamed[0].ty;
            u_add = quote! { #u_add + <#ty as trait_layers::Layers>::COUNT - 1 };
        }
        add
    });
    let (us, ns): (Vec<_>, Vec<_>) = data_enum
        .variants
        .iter()
        .enumerate()
        .zip(u_add)
        .partition(|((_, v), _)| matches!(&v.fields, syn::Fields::Unnamed(_)));
    let (u, ui, ua) = {
        let (uui, ua): (Vec<_>, Vec<_>) = us.into_iter().unzip();
        let (ui, u): (Vec<_>, Vec<_>) = uui.into_iter().unzip();
        let u: Vec<_> = u.iter().map(|v| &v.ident).collect();
        (u, ui, ua)
    };
    let (n, ni, na) = {
        let (nni, na): (Vec<_>, Vec<_>) = ns.into_iter().unzip();
        let (ni, n): (Vec<_>, Vec<_>) = nni.into_iter().unzip();
        (n, ni, na)
    };

    let v_end = data_enum.variants.iter().scan(quote! { 0 }, |v_end, v| {
        *v_end = if let syn::Fields::Unnamed(u) = &v.fields {
            let ty = &u.unnamed[0].ty;
            quote! { #v_end + <#ty as trait_layers::Layers>::COUNT }
        } else {
            quote! { #v_end + 1 }
        };
        Some(v_end.clone())
    });
    let v_rng = v_end.clone().scan(quote! { 0 }, |st, v_end| {
        let mut v_start = v_end.clone();
        core::mem::swap(st, &mut v_start);
        Some(quote! { (#v_start)..(#v_end) })
    });
    let v_start = core::iter::once(quote! { 0 }).chain(v_end);
    let (us, ns): (Vec<_>, Vec<_>) = data_enum
        .variants
        .iter()
        .zip(v_start)
        .zip(v_rng)
        .partition(|((v, _), _)| matches!(&v.fields, syn::Fields::Unnamed(_)));
    let (u2, t2, us, ur) = {
        let (u2us, ur): (Vec<_>, Vec<_>) = us.into_iter().unzip();
        let (u2, us): (Vec<_>, Vec<_>) = u2us.into_iter().unzip();
        let t2 = u_ty.clone();
        let u2: Vec<_> = u2.iter().map(|v| &v.ident).collect();
        (u2, t2, us, ur)
    };
    let (n2, ns) = {
        let n2ns: Vec<_> = ns.into_iter().map(|(n2ns, _)| n2ns).collect();
        let (n2, ns): (Vec<_>, Vec<_>) = n2ns.into_iter().unzip();
        (n2, ns)
    };

    let output = quote! {
        unsafe impl trait_layers::Layers for #ident {
            const COUNT: usize = #count #(+ <#u_ty as trait_layers::Layers>::COUNT - 1)*;

            fn as_num(&self) -> usize {
                #root_check
                match &self {
                    #(Self::#u(nested) => trait_layers::Layers::as_num(nested) + #ui + #ua,)*
                    #(Self::#n => #ni + #na,)*
                    _ => loop {} // unsafe { ::core::hint::unreachable_unchecked!() } cannot be imported for some odd reason
                }
            }

            fn try_from_num(num: usize) -> Option<Self> {
                #(if (#ur).contains(&num) {
                    return <#t2 as trait_layers::Layers>::try_from_num(num - (#us)).map(|nested| Self::#u2(nested));
                })*
                #(if (#ns) == num {
                    return Some(Self::#n2);
                })*
                None
            }
        }
    };
    output.into()
}

#[proc_macro_derive(Root)]
pub fn root(input: TokenStream) -> TokenStream {
    let syn::DeriveInput { ident, .. } = syn::parse_macro_input!(input);
    let output = quote! {
        impl trait_layers::Root for #ident {}
    };
    output.into()
}