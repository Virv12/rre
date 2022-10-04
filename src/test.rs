#![feature(test)]

use rre::regex;

extern crate test;

fn main() {
    let regex = regex!("a|b*");
    dbg!(regex("a"));
    dbg!(regex("b"));
    dbg!(regex("bb"));
    dbg!(regex("aa"));
    dbg!(regex(""));
}

use test::Bencher;

#[bench]
fn bench1(b: &mut Bencher) {
    let regex = regex!("aaaaaaaaaa(a|)(a|)(a|)(a|)(a|)(a|)(a|)(a|)(a|)(a|)");
    b.iter(|| {
        let s = test::black_box("aaaaaaaaaa");
        regex(s)
    });
}

#[bench]
fn bench2(b: &mut Bencher) {
    let re = regex::Regex::new("aaaaaaaaaa(a|)(a|)(a|)(a|)(a|)(a|)(a|)(a|)(a|)(a|)").unwrap();
    b.iter(|| {
        let s = test::black_box("aaaaaaaaaa");
        re.is_match(s);
    });
}
