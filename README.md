# 🧙🏻 SABRY - Syntactically Awesome, But RustY

**Y**et another **R**usty **B**oilerplate-free **A**gnostic **S**tyling crate, which brings your SASS/SCSS style into Rust. Written by a fox this time.

\* sabry isn't "syntactically awesome", it refers to SASS abbr expansion.

> **Project status** - early, good enough for my team's non-critical "for-fun" pre-production, on demand. WIP features have no ETA. I'm pretty happy with ergonomics and taste of the crate, and I'll do my best to keep DX the way it is between minor versions - but there's no guarantee on backwards-compatibility and no refunds if something breaks.
>
> "master" branch is what's currently on crates.io
>
> each version ever been on crates.io (except the latest release) is available in "r_x.x.x" branch
>
> the latest-trashy state is in the "window" branch

[![Crates.io](https://img.shields.io/crates/v/sabry.svg)](https://crates.io/crates/sabry)
[![Docs.rs](https://img.shields.io/docsrs/sabry/latest.svg)](https://docs.rs/sabry)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/yiffyrusdev/sabry/blob/master/LICENSE)

At first, I'll show how this crate "tastes". With SABRY, its in your power to:

<table>
<tr>
<td>

Write arbitrary SASS/SCSS code and ship it *as a crate* however you please: with modules, with feature flags, etc. Say "bye" to manual copying, cli-tools and consts.

```rust,ignore
sassy!(tokens {@mixin colored{color: red;}});
```

</td>
<td>
📦
</td>
</tr>
<tr>
<td>

Code your SASS in separate files to get proper syntax highlighting, or do sass-in-the-rust to keep things in one place

```rust,ignore
styly!(component {.btn {color: green;}});
styly!(extras "src/components/extras.scss");
```

</td>
<td>
:3
</td>
</tr>
<tr>
<td>

Depend on styles with cargo, at build time, which brings all the rusty sweeties in: versions, updates, cratesio, local registries, workspaces, etc.

```toml
[build-dependencies]
tgk_brandstyle = {version = "0.0.1", features = ["utils"]}
```

</td>
<td>
🦀
</td>
</tr>
<tr>
<td>

`@use` your style-crates in sass code *naturally*

```rust,ignore
styly!(breadbadge {
    @use 'tokens';
    .scope {@include tokens.badge(primary);}
})
```

</td>
<td>
🪄
</td>
</tr>
<tr>
<td>

Keep things as private and modular as you wish

```rust,ignore
styly!(pub cats "src/sections/cats/style.scss");
styly!(dogs {#howl {border: none;}})

/// something like render function
{
    <div class={cats::meow}>
        <ul id={dogs::thehowl}></ul>
    </div>
}
```

</td>
<td>
^_^
</td>
</tr>
<tr>
<td>

Compile all the sweet SASS/SCSS into the optimized CSS bundle, ship it in CSS shunks or even include the compiled style into the binary

```rust,ignore
styly!(cssbundle {.c1 {color: white;}})
styly!(const binary {.c2 {color: black;}})
```

</td>
<td>
🏗️
</td>
</tr>
</table>

Sabry will gladly:

- scope and hash your styles, so they won't conflict, with amazing [raffia](https://crates.io/crates/raffia) parser
- compile SASS/SCSS into CSS with [grass](https://crates.io/crates/grass)
- optimize produced CSS with [lightningcss](https://crates.io/crates/lightningcss)
- prepare ready CSS bundle and put it wherever you wish
- include CSS into the build artifact - if you really want it

Also, just about everything is pub-available in this crate - Sabry is ready for experiments.


## Usage

Feel free to check out examples:

|[crate of styles](./examples//define-styles/)|[style usage](./examples/use-styles/)|[leptos-axum with sabry](./examples/leptos-axum)|
|-|-|-|

### Create a crate full of arbitrary SASS

The only need is the dependency

```toml
#Cargo.toml

[dependencies]
sabry = {version = "0.0.1"}
```
And a proc-macro
```rust,ignore
// lib.rs
use sabry::sassy;

sassy!(mixins "assets/mixins.scss");
sassy!(styles {
    $primary-color: black;
    @mixin colored($col: primary) {
        @if $col == primary {
            color: $primary-color;
        } @else {
            color: $col;
        }
    }
});
```
Now you can build and see two ready for export macros: `mixins!` and `styles!`.
These are usefull on their own, as invocation of `mixins!()` or `styles!()` - both shall give you the code literal.

However there's more sweet use case for them, which is covered below.

### Write scoped styles

Depend on sabry
```toml
# Cargo.toml

[dependencies]
sabry = {version = "0.0.1"}
```
And create a style scope wherever you want:
```rust,ignore
// breadbadgelist.rs
use sabry::styly;

styly!(const styles {
    .badges {
        display: flex;
        &__list {
            display: flex;
        }
    }
    #wolf {
        color: white;
    }
});

fn render() -> HtmlElement {
    div().class(styles::badges).chain(
        ul().class(styles::_list(styles::badges))
    ).chain(
        span().id(styles::thewolf)
    )
}
```
> That `const` usage is covered in the [following](#constant-styly-scopes) section. In this example we dont invoke sabry build-magic, so, to be as close to real life as possible, I used a const.

Every selector, if that does make sense, now available for you as a member of `styles` scope. In this example - `styles::badges`, `styles::thewolf` and `styles::_list()`. More about scoping and member names you can read [here](#styly-scopes).

### Use styles earlier created in another crate

The combination of previous two, with some additional work to do and some extra sugar to enjoy.

Sabry is needed as both dependency and build-dependency.
To be able to compile all styles sabry also needs the *build* feature flag:
```toml
# Cargo.toml

[dependencies]
sabry = {version = "0.0.1"}

[build-dependencies]
sabry = {version = "0.0.1", features = ["build"]}
```
> If you do use some non-default feature flags make sure to keep them in sync between sabry-dependency and sabry-build-dependency.

Then you have to tell sabry when code should be compiled. We'll do this in *build.rs* file.
```rust,ignore
// build.rs
use shared_styles::{mixins, styles};

fn main(){
    sabry::buildy(
        sabry::usey!(
            mixins!(),
            styles!()
        )
    ).expect("Failed to build sabry styles");
}
```
`buildy` is the entry function of sabry build-time process. The handy `usey!` macro will do just proper handling of our style-macros for it.

Now lets get back to the code and use the mixin defined in another crate:
```rust,ignore
// breadbadgelist.rs
use sabry::styly;

styly!(styles {
    @use "mixins";
    .badges {
        display: flex;
        &__list {
            display: flex;
        }
    }
    #wolf {
        @include mixins.colored(white);
    }
});

fn render() -> HtmlElement {
    div().class(styles::badges).chain(
        ul().class(styles::_list(styles::badges))
    ).chain(
        span().id(styles::thewolf)
    )
}
```
So the `mixins!` macro we just passed to the `usey!` macro inside of `buildy` function call is now accessible with simple and natural `@use "mixins"` SASS rule!

## Configuration

Sabry configuration lives in `[package.metadata.sabry]` table of the manifest file.

All configurations are optional, but with default configuration sabry won't produce any CSS files.

Full example, close to defaults:
```toml
# Cargo.toml

[package.metadata.sabry]
css.bundle = "target/static/style.css"
css.bundle_prelude = ["assets/prelude.css"]
css.scopes = "target/statis/scopes/"
css.minify = true

sass.intermediate_dir = "target/.sabry/sass"
sass.module_name_collision = "merge"
sass.modules = ["assets/sass/mod1.scss"]
sass.scanroot = "src"

hash.size = 6
hash.collision = "error"
hash.use_scope_name = true
hash.use_code_size = true
hash.use_item_names = false
hash.use_code_text = false

[package.metadata.sabry.lightningcss.targets]
chrome = "120"
safari = "13.2"
ie = "6"
```

### `sabry.css`

**bundle** *(no default)* - file path ro write CSS bundle into relative to project root

**bundle_prelude** *(no default)* - collection of CSS files, relative to the project root, which content will be inserted before the compiled style into the *bundle* file if any

**scopes** *(no default)* - dir path to put separate CSS for every scope into relative to project root

**minify** *(default true)* - print compressed CSS output and do the lightningcss thing

### `sabry.sass`

**intermediate_dir** *(default "target/.sabry/sass")* - file to put SASS/SCSS modules into so they are available with `@use` in code

**scanroot** *(default "src")* - root directory to start scanning "rs" files from. Used in build function

**modules** *(no default)* - collection of SASS/SCSS files, relative to the project root, which should be available as modules as well

**module_name_collision** *(default "merge")* - how to handle similary named modules.

*merge* - merge content

*error* - break building process with an error

### `sabry.hash`

**size** *(default 6)* - size of hash in bytes. Feel free to increase/decrease.

**use_scope_name** *(default true)* - wether to use scope identifier to calculate hash

**use_code_size** *(default true)* - wether to use scope code size to calculate hash

**use_item_names** *(default false)* - wether to use all scoped item idents to calculate hash

**use_code_text** *(default false)* - wether to use the scope code text to calculate hash

**collision** *(default "ignore")* - how to handle similarity of generated hashes

*ignore* - dont do anything

*error* - break building process with an error

### `sabry.lightningcss.targets`

Does require `css.minify` to be *true*.

Empty by default.

Available keys: chrome, firefox, edge, safari, saf_ios, samsung, android, ie

Value - minimal browser version to support in "M.m.p" format, where:

- *M* - major
- *m* - minor
- *p* - patch

For example `{ie = "9", saf_ios = "13.2"}` will try to generate CSS supported on both IE 9 and Safari-on-ios 13.2

## Detailed guide

### Style definition with `sassy!`

The `sassy!` macro is available with *procmacro* feature which is enabled by default.

It does accept the following syntax: `$name(:$syntax)? ({ $code })|($filename)`, where

- *$name* is any identifier valid for `macro_rules!`
- *$syntax* is either `sass` or `scss`
- *$code* is valid arbitrary style code in specified syntax
- *$filename* is a string literal which contains path to the file relative to package root

Examples:
```rust,ignore
sassy!(module1 {$primary-color: red;});
sassy!(module2 "src/assets/module2.scss");
sassy!(module3:sass "src/assets/module3.sass");
// works, but there are catches.
sassy!(module4:sass {
    =colored
        color: red
});
```

You may omit the syntax specifier - sabry uses SCSS as the default one.

The given code to `sassy!` is not checked to be valid code in given syntax (wip).

> SASS support inside of rust files is experimental. If you do want to use SASS tabbed syntax - consider to use files path instead of sass-in-rust option.

### Scoping with `styly!`

The `styly!` macro is available with *procmacro* feature flag which is enabled by default.

It does accept the following syntax: `pub? const? $ident(:$syntax)? ({ $code })|($filename)`, where

- *pub* is explained [here](#public-styly-scopes)
- *const* is explained [here](#constant-styly-scopes)
- *$ident* is any identifier valid for `mod`
- *$syntax* is either `sass` or `scss`
- *$code* is arbitrary style code valid with given syntax
- *$filename* is a string literal which contains path to the file relative to package root

Examples
```rust,ignore
styly!(fox {.fur {color: red; &-dark {color: black;}}})
styly!(pub fox {.fur {color: red; &-dark {color: black;}}})
styly!(pub const fox:sass {
    .fur
        color: red
        &-dark
            color: black
})
```
Every of those calls will produce the styling scope as a module. Differences are explained right below.

In general the scope does look like this:
```rust,ignore
const FOX: &str = "J9k_s9";
mod fox {
    pub const fur: &str = "J9k_s9 fur";
    pub fn _dark(c: &str) -> String {format!("{c}-dark")}
}
```

#### Styly scopes

Styly macro itself does not generate the scope. It is done in the `sabry_intrnl::scoper`. However, as a result, you will have the following:

- `const` with the UPPER_CASE name of the scope, which contains its hash
- `mod` with exact scope name which is the scope you're going to use
    - for simple selectors, like `.fur` you will have a const members with hashed original selectors to use wherever you need a class/id/etc.
    - for parent-selectors like `&-dark` you will have a non-const function, which will create matching the selector from any other.

You can read more about scoping and hashing in the [scoping](#scoping) section.

#### Public styly scopes

By default generated `mod` is private. You can make both mod and wrapper style constant public by adding the `pub` to macro call:

```rust,ignore
styly!(pub whatever "src/assets/whatever.scss");
```

#### Constant styly scopes

As you've seen above, scope doe not contain any style code by itself. That's the use case i advise mostly.

However you could still compile styles into the artifact by simply adding the `const` to the macro call:

```rust,ignore
styly!(const scope "src/assets/scope.scss");
```
Which results in following:
```rust,ignore
const SCOPE: &str = /* scope hash */;
const SCOPE_CSS: &str = /* compiled from src/assets/scope.scss */;
mod scope {/* selector collection */}
```

> **There is a catch** | with the `const` modifier macro must compile CSS at compile-time. That results in several game changers:
>
> *First*. You could avoid the "build magic". Sabry will just compile given styles with procmacro at compile time.
>
> *Second*. If you `@use` something inside of constant-flavored scope, you can only success if sabry *did the build magic before compilation of that macro call*. So you still can compile the styles into the artifact and enjoy mixins from other crates, but, in general, you are going to receive some false-positives from editor.
>
> Worth of notice: sabry will still include const-flavored styles into the CSS bundle during build time.

### Building with `buildy` and `usey!`

The `buildy` function is available with *build* feature which needs to be enabled explicitly.

This function accepts an iterator of pairs: (file_name, code) in form of `(String, String)` type. File name should have an extension, so grass can infere syntax during CSS compilation.

Each of those pairs is processed as a file which sabry needs to write into
the configured `intermediate_dir` and then passed into the CSS compiler.

You could, for example, define the module "mixin_a":
```rust,ignore
buildy(vec![("mixin_a".to_string(), "@mixin a(){}".to_string())]);
```

However there's a usey macro, which handles this for you:
```rust,ignore
buildy(
    usey!(mixins!(), utils!())
);
```

The `usey!` macro accepts the following syntax:
`#($macro,)*`, where

- *$macro* is a macro which handles three expansions:
    - `() => { $code }`
    - `(name) => { $name }`
    - `(syntax) => { $syntax }`

Where for the *$macro*:

- *$code* is a source code of style as a string literal
- *$name* is a name of module without extension as a string literal
- *$syntax* is either "sass" or "scss"


Exactly this kind of macros is produced by [`sassy!`](#style-definition-with-sassy).

### Scoping

Sabry handles scoping by restriction or mutation of existing selectors
with the hash. Hash is calculated for the entire scope by the [`styly!`](#scoping-with-styly) macro.

Currently the following types of selectors are scoped:

- class
- id
- SASS parent selectors (see below, there is a catch)
- tagname (see below, there is a catch)

Sabry does not make difference between top-level and nested selectors,
also selector complexity isn't taken into account: sabry simply walks through all compound selectors
and apply scoping for supported ones.

Different selector types are scoped differently:

- class selectors are restricted with scope hash: `.class` -> `.HASH.class`
- id selector are mutated with scope hash: `#id` -> `#id-HASH`
- tagname selectors are changed into indirect descendance: `div` -> `.HASH div`

> **There is a catch** | currently tagname selectors are scoped with indirect descendancy
> hash restriction. That means - if you do use tagnames in scoped stylesheets, you have to wrap your markdown
> with `SCOPE` class generated by `styly!` macro.
>
> Also if there are nested tagname selectors - sabry won't take this into account and you'll
> have to wrap them with `SCOPE` class as well

As for **SASS parent selectors**: they are currently handled in different way. Instead of
walking up the syntax tree sabry just creates function member for the scope and leave the rest to grass:

```rust,ignore
styly!(scope {
    .cls1 {
        &-dark {}
    }
});
```
is something like
```rust,ignore
const SCOPE: &str = "Ut8CskJ";
mod scope {
    pub const cls1: &str = "Ut8CskJ cls1";
    pub fn _dark(c: &str) -> String {format!("{c}-dark")}
}
```

So you still can get the scoped variant of `cls1-dark` class: `scope::_dark(scope::cls1)`.

This isn't very handy, also isn't strict enough, and is a high-priority subject to change. wip.

## WIP

*(sorted by my own priority)*, "dones" are excluded

- [] Crates of styled components - currently the only way to create them seems to be const styly.
    - [] CSS support
- [] Currently the crate causes "dependency inheritance" infection. We cant get rid of it, however should be doable to at least get rid of flag inheritance
- [] There are some strange parsing errors, seems like a bug, however very hard to reproduce. Have to investigate. Maybe do more tests.
- [] Experience with cargo-leptos is fine, and we do use it, however its a bit "raughy". Need to do something about it.

## Contributions

Any contributions are always welcome!

If you find this crate usefull, wanna stick with it in some project, but do miss some
features - feel free to submit a PR.

> If you'd like to fork for the PR - please, use the "window" branch, not the "master".

If you encounter any bugs/problems or have use case where things dont work as they should
for the latest version - please, open an issue!

If you'd like to lend a paw - feel free to check the [WIP](#wip) section out, or to search for "TODO" comments.

## MSRV

Sabry passes its own tests on 1.82 nightly.

There *were* some strange parsing issues reports on *stable* channel, seems to be fixed. idk.

## License

MIT.
