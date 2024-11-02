use std::{collections::HashSet, fs};

use sabry_procmacro::{sassy, usey};

sassy!(scmod:scss {"
    @mixin abc(){
        color: red;
    }
    .sel1{
        @include abc;
    }
"});
sassy!(samod:sass {"
    @mixin abc()
        color: green
    .sel1
        @include abc
"});

sassy!(scmicromod:scss "sabry_procmacro/tests/scmicro.scss");
sassy!(samicromod:sass "sabry_procmacro/tests/samicro.sass");

#[test]
fn sassy_contract() {
    assert_eq!(scmod!(syntax), "scss");
    assert_eq!(
        scmod!(),
        "
@mixin abc(){
    color: red;
}
.sel1{
    @include abc;
}
"
    );

    assert_eq!(scmicromod!(syntax), "scss");
    assert_eq!(
        scmicromod!(),
        fs::read_to_string("tests/scmicro.scss").unwrap()
    );

    assert_eq!(samod!(syntax), "sass");
    assert_eq!(
        samod!(),
        "
@mixin abc()
    color: green
.sel1
    @include abc
"
    );

    assert_eq!(samicromod!(syntax), "sass");
    assert_eq!(
        samicromod!(),
        fs::read_to_string("tests/samicro.sass").unwrap()
    );
}

#[test]
fn usey_contract() {
    use samicromod as aliasedmod;

    let _usey = usey!(aliasedmod!(), scmicromod!());
    let expect_modules =
        HashSet::from(["aliasedmod.sass".to_string(), "scmicromod.scss".to_string()]);
    let expect_codes = HashSet::from([
        fs::read_to_string("tests/samicro.sass").unwrap(),
        fs::read_to_string("tests/scmicro.scss").unwrap(),
    ]);

    let mut real_modules = HashSet::new();
    let mut real_codes = HashSet::new();

    _usey.iter().for_each(|(module, code)| {
        real_modules.insert(module.clone());
        real_codes.insert(code.clone());
    });

    assert_eq!(expect_modules, real_modules);
    assert_eq!(expect_codes, real_codes)
}
