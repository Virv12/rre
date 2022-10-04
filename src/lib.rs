use std::{iter::Peekable, str::Chars};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

fn postfix(s: &str) -> String {
    fn a(it: &mut Peekable<Chars>, out: &mut String) {
        match it.peek() {
            Some(&c @ 'a'..='z') => {
                it.next();
                out.push(c);
            }
            Some(&'*') => {}
            Some(&')') | Some(&'|') | None => {
                out.push('`');
            }
            Some(&'(') => {
                it.next();
                c(it, out);
                assert_eq!(it.next(), Some(')'));
            }
            _ => unreachable!(),
        }

        loop {
            match it.peek() {
                Some(&'*') => {
                    it.next();
                    out.push('*');
                }
                _ => break,
            }
        }
    }

    fn b(it: &mut Peekable<Chars>, out: &mut String) {
        a(it, out);
        while matches!(it.peek(), Some(&('a'..='z' | '('))) {
            a(it, out);
            out.push('.');
        }
    }

    fn c(it: &mut Peekable<Chars>, out: &mut String) {
        b(it, out);
        while matches!(it.peek(), Some(&'|')) {
            it.next();
            b(it, out);
            out.push('|');
        }
    }

    let mut it = s.chars().peekable();
    let mut out = String::new();
    c(&mut it, &mut out);
    assert_eq!(it.next(), None);
    out
}

#[proc_macro]
pub fn regex(input: TokenStream) -> TokenStream {
    let re = parse_macro_input!(input as LitStr).value();
    assert!(re.chars().all(|c| c.is_ascii_lowercase()
        || c == '*'
        || c == '|'
        || c == '('
        || c == ')'));

    let pf = postfix(&re);

    let n = 2 * pf.chars().filter(|&c| c != '.').count();

    let mut v = vec![0u32; n * n];

    let mut free = 0;
    let mut stk = Vec::new();
    for c in pf.chars() {
        match c {
            '|' => {
                let s = free;
                let t = free + 1;
                free += 2;
                let (a, b) = stk.pop().unwrap();
                let (c, d) = stk.pop().unwrap();
                v[s * n + a] |= 1;
                v[s * n + c] |= 1;
                v[b * n + t] |= 1;
                v[d * n + t] |= 1;
                stk.push((s, t));
            }
            '.' => {
                let (a, b) = stk.pop().unwrap();
                let (c, d) = stk.pop().unwrap();
                v[d * n + a] |= 1;
                stk.push((c, b));
            }
            '*' => {
                let s = free;
                let t = free + 1;
                free += 2;
                let (a, b) = stk.pop().unwrap();
                v[s * n + a] |= 1;
                v[s * n + t] |= 1;
                v[b * n + a] |= 1;
                v[b * n + t] |= 1;
                stk.push((s, t));
            }
            c @ ('`' | 'a'..='z') => {
                let s = free;
                let t = free + 1;
                free += 2;
                v[s * n + t] |= 1 << (c as i32 - '`' as i32);
                stk.push((s, t));
            }
            _ => unreachable!(),
        }
    }

    assert_eq!(stk.len(), 1);

    for _ in 0..n {
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    if v[i * n + j] & 1 != 0 {
                        v[i * n + k] |= v[j * n + k];
                    }
                    if v[j * n + k] & 1 != 0 {
                        v[i * n + k] |= v[i * n + j];
                    }
                }
            }
        }
    }

    let (s, t) = stk.pop().unwrap();

    let tokens: TokenStream = quote! {
        |s: &str| -> bool {
            let arr = [ #(#v),* ];
            if s.is_empty() {
                return arr[#s * #n + #t] & 1 != 0;
            }

            let mut state = [false; #n];
            state[#s] = true;

            for c in s.chars() {
                assert!(matches!(c,'a'..='z'));

                let mut nstate = [false; #n];
                for i in 0..#n {
                    for j in 0..#n {
                        if state[i] && arr[i * #n + j] >> (c as i32 - '`' as i32) & 1 != 0 {
                            nstate[j] = true;
                        }
                    }
                }
                state = nstate;
            }

            state[#t]
        }
    }
    .into();

    tokens
}
