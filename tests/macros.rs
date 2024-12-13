pub mod sass {
    use sabry::scssy;

    scssy!(mixins_sass:sass "tests/assets/mixin-module.sass");
    scssy!(mixins_inline_sass:sass {"
    @mixin color($color: black)
        @if $color == black
            color: white
            background-color: green
        @else if $color == white
            color: red
            background-color: black
        @else
            color: #c6c6c6
            background-color: #00ffff
        &::after
            content: 'Colored'
            color: $color
            background-color: blue
            &:hover
                transform: scale(2)
        &:hover
            transform: translateX(10%)
"});
    scssy!(module_sass:sass "tests/assets/module-forward.sass");
    scssy!(module_inline_sass:sass {"
@forward 'mixins_a'
@forward 'mixins_c'
"});
    pub(crate) use mixins_inline_sass as mixins_inline_a;
    pub(crate) use mixins_sass as mixins_a;
    pub(crate) use module_inline_sass as module_inline_a;
    pub(crate) use module_sass as module_a;
}

pub mod scss {
    use sabry::scssy;

    scssy!(mixins_scss:scss "tests/assets/mixin-module.scss");
    scssy!(mixins_inline_scss:scss {"
    @mixin surface($color: black){
        @if $color == black{
            color: white;
            background-color: green;
        }@else if $color == white{
            color: red;
            background-color: black;
        }@else{
            color: #c6c6c6;
            background-color: #00ffff;
        }
        &::after{
            content: 'Colored';
            color: $color;
            background-color: blue;
            &:hover{
                transform: scale(2);
            }
        }
        &:hover{
            transform: translateX(10%);
        }
    }
"});
    scssy!(module_scss:scss "tests/assets/module-forward.scss");
    scssy!(module_inline_scss:scss {"
@forward 'mixins_a';
@forward 'mixins_c';
"});

    pub(crate) use mixins_inline_scss as mixins_inline_c;
    pub(crate) use mixins_scss as mixins_c;
    pub(crate) use module_inline_scss as module_inline_c;
    pub(crate) use module_scss as module_c;
}

#[test]
fn ensure_inline_matches_file() {
    assert_eq!(sass::mixins_a!(), sass::mixins_inline_a!());
    assert_eq!(scss::mixins_c!(), scss::mixins_inline_c!());

    assert_eq!(sass::module_a!(), sass::module_inline_a!());
    assert_eq!(scss::module_c!(), scss::module_inline_c!());
}

#[test]
fn syntax_contract() {
    assert_eq!(scss::mixins_inline_c!(syntax), "scss");
    assert_eq!(scss::mixins_c!(syntax), "scss");
    assert_eq!(scss::module_inline_c!(syntax), "scss");
    assert_eq!(scss::module_c!(syntax), "scss");

    assert_eq!(sass::mixins_a!(syntax), "sass");
    assert_eq!(sass::mixins_inline_a!(syntax), "sass");
    assert_eq!(sass::module_a!(syntax), "sass");
    assert_eq!(sass::module_inline_a!(syntax), "sass");
}

#[cfg(test)]
#[cfg(all(feature = "procmacro", feature = "build"))]
mod build_tests {
    use std::fs;

    use insta::{glob, Settings};
    use sabry::{
        buildmagic::SabryBuilder,
        config::{BehavHashCollision, BehavSassModCollision, SabryConfig},
        usey,
    };

    fn gen_config() -> SabryConfig {
        let mut config = SabryConfig::require().unwrap();
        config.css.bundle = Some("tests/sabry_output/bundle.css".into());
        config.css.scopes = Some("tests/sabry_output/scopes".into());
        config.css.minify = false;
        config.css.prelude = Some(vec!["tests/assets/prelude.css".into()]);

        config.sass.prelude = Some(vec![
            "tests/assets/prelude.sass".into(),
            "tests/assets/prelude.scss".into(),
        ]);
        config.sass.intermediate_dir = "tests/sabry_intermediate".into();
        config.sass.scanroot = "tests".into();
        config.sass.module_name_collision = BehavSassModCollision::Merge;

        config.hash.collision = BehavHashCollision::Error;
        config.hash.size = 16;
        config.hash.use_code_size = true;
        config.hash.use_code_text = true;
        config.hash.use_item_names = true;
        config.hash.use_scope_name = true;

        config.lightningcss.targets.android = Some("10".into());
        config.lightningcss.targets.chrome = Some("100".into());
        config.lightningcss.targets.edge = Some("80".into());
        config.lightningcss.targets.firefox = Some("100".into());
        config.lightningcss.targets.ie = Some("8".into());
        config.lightningcss.targets.ios_saf = Some("13.2".into());
        config.lightningcss.targets.opera = Some("80".into());
        config.lightningcss.targets.safari = Some("11".into());
        config.lightningcss.targets.samsung = Some("80".into());

        config
    }

    fn gen_builder() -> SabryBuilder {
        let config = gen_config();
        SabryBuilder::new(config)
    }

    #[test]
    fn compilation_with_buildy() {
        let mut builder = gen_builder();
        builder
            .build(usey!(
                super::sass::mixins_a!(),
                super::sass::module_a!(),
                super::scss::mixins_c!(),
                super::scss::module_c!()
            ))
            .unwrap();

        glob!("sabry_output/**/*.css", |path| {
            let generated = fs::read_to_string(path).unwrap();

            let mut settings = Settings::clone_current();
            settings.set_snapshot_path("sabry_output_snapshots");
            settings.set_prepend_module_to_snapshot(false);
            settings.set_omit_expression(true);
            settings.remove_info();
            settings.bind(|| {
                insta::assert_snapshot!(generated);
            });
        });
    }
}
