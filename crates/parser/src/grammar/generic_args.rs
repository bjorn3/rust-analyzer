use super::*;

pub(super) fn opt_generic_arg_list(p: &mut Parser, colon_colon_required: bool) {
    let m;
    if p.at(T![::]) && p.nth(2) == T![<] {
        m = p.start();
        p.bump(T![::]);
        p.bump(T![<]);
    } else if !colon_colon_required && p.at(T![<]) && p.nth(1) != T![=] {
        m = p.start();
        p.bump(T![<]);
    } else {
        return;
    }

    while !p.at(EOF) && !p.at(T![>]) {
        generic_arg(p);
        if !p.at(T![>]) && !p.expect(T![,]) {
            break;
        }
    }
    p.expect(T![>]);
    m.complete(p, GENERIC_ARG_LIST);
}

// test generic_arg
// type T = S<i32>;
fn generic_arg(p: &mut Parser) {
    match p.current() {
        LIFETIME_IDENT => lifetime_arg(p),
        T!['{'] | T![true] | T![false] | T![-] => const_arg(p),
        k if k.is_literal() => const_arg(p),
        // test associated_type_bounds
        // fn print_all<T: Iterator<Item, Item::Item, Item::<true>, Item: Display, Item<'a> = Item>>(printables: T) {}

        // test macro_inside_generic_arg
        // type A = Foo<syn::Token![_]>;
        IDENT if [T![<], T![=], T![:]].contains(&p.nth(1)) && !p.nth_at(1, T![::]) => {
            let m = p.start();
            name_ref(p);
            opt_generic_arg_list(p, false);
            match p.current() {
                T![=] => {
                    p.bump_any();
                    if types::TYPE_FIRST.contains(p.current()) {
                        // test assoc_type_eq
                        // type T = StreamingIterator<Item<'a> = &'a T>;
                        types::type_(p);
                    } else {
                        // test assoc_const_eq
                        // fn foo<F: Foo<N=3>>() {}
                        // const TEST: usize = 3;
                        // fn bar<F: Foo<N={TEST}>>() {}
                        const_arg(p);
                    }
                    m.complete(p, ASSOC_TYPE_ARG);
                }
                // test assoc_type_bound
                // type T = StreamingIterator<Item<'a>: Clone>;
                T![:] if !p.at(T![::]) => {
                    generic_params::bounds(p);
                    m.complete(p, ASSOC_TYPE_ARG);
                }
                _ => {
                    let m = m.complete(p, PATH_SEGMENT).precede(p).complete(p, PATH);
                    let m = paths::type_path_for_qualifier(p, m);
                    m.precede(p).complete(p, PATH_TYPE).precede(p).complete(p, TYPE_ARG);
                }
            }
        }
        _ => type_arg(p),
    }
}

// test lifetime_arg
// type T = S<'static>;
fn lifetime_arg(p: &mut Parser) {
    let m = p.start();
    lifetime(p);
    m.complete(p, LIFETIME_ARG);
}

pub(super) fn const_arg_expr(p: &mut Parser) {
    // The tests in here are really for `const_arg`, which wraps the content
    // CONST_ARG.
    match p.current() {
        // test const_arg_block
        // type T = S<{90 + 2}>;
        T!['{'] => {
            expressions::block_expr(p);
        }
        // test const_arg_literal
        // type T = S<"hello", 0xdeadbeef>;
        k if k.is_literal() => {
            expressions::literal(p);
        }
        // test const_arg_bool_literal
        // type T = S<true>;
        T![true] | T![false] => {
            expressions::literal(p);
        }
        // test const_arg_negative_number
        // type T = S<-92>;
        T![-] => {
            let lm = p.start();
            p.bump(T![-]);
            expressions::literal(p);
            lm.complete(p, PREFIX_EXPR);
        }
        _ => {
            // This shouldn't be hit by `const_arg`
            let lm = p.start();
            paths::use_path(p);
            lm.complete(p, PATH_EXPR);
        }
    }
}

// test const_arg
// type T = S<92>;
pub(super) fn const_arg(p: &mut Parser) {
    let m = p.start();
    const_arg_expr(p);
    m.complete(p, CONST_ARG);
}

fn type_arg(p: &mut Parser) {
    let m = p.start();
    types::type_(p);
    m.complete(p, TYPE_ARG);
}
