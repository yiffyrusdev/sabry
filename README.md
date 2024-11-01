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

```rust
sabry::sassy!(tokens {"@mixin colored{color: red;}"});
```

</td>
</tr>
<tr>
<td>

Get the *(almost)* proper autocompletion for your scope from rust-analyzer

```rust
sabry::styly!(scope {".cls1{} #id1{}"});
/* autocompleted for you */
scope::cls1;
scope::theid1;
```

</td>
</tr>
<tr>
<td>

Code your SASS in separate files to get proper syntax highlighting, or do sass-in-the-rust to keep things in one place

```rust
sabry::styly!(component {".btn {color: green;}"});
sabry::styly!(extras "tests/assets/mixins.scss");
```

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
</tr>
<tr>
<td>

`@use` your style-crates in sass code *naturally*

```rust
sabry::styly!(breadbadge {"
    @use 'tokens';
    .scope {@include tokens.badge(primary);}
"});
```

</td>
</tr>
<tr>
<td>

Keep things as private and modular as you wish

```rust
sabry::styly!(pub cats "tests/assets/mixins.scss");
sabry::styly!(dogs {"#howl {border: none;}"});
```
```rust,ignore
/// something like render function
{
    <div class={cats::meow}>
        <ul id={dogs::thehowl}></ul>
    </div>
}
```

</td>
</tr>
<tr>
<td>

Compile all the sweet SASS/SCSS into the optimized CSS bundle, ship it in CSS shunks or even include the compiled style into the binary

```rust
sabry::styly!(cssbundle {".c1 {color: white;}"});
sabry::styly!(const binary {".c2 {color: black;}"});
```

</td>
</tr>
</table>

Sabry will gladly:

- scope and hash your styles, so they won't conflict, with amazing [raffia](https://crates.io/crates/raffia) parser
- compile SASS/SCSS into CSS with [grass](https://crates.io/crates/grass)
- optimize produced CSS with [lightningcss](https://crates.io/crates/lightningcss)
- prepare ready CSS bundle and put it wherever you wish
- include CSS into the build artifact - if you really want it

Also, just about everything is pub-available in this crate (with *internals* feature flag) - Sabry is ready for experiments.

## Usage

Feel free to check out examples:

|[crate of styles](https://github.com/yiffyrusdev/sabry/tree/master/examples/define-styles)|[style usage](https://github.com/yiffyrusdev/sabry/tree/master/examples/use-styles)|[leptos-axum with sabry](https://github.com/yiffyrusdev/sabry/tree/master/examples/leptos-axum)|
|-|-|-|

### Create a crate full of arbitrary SASS

The only need is the dependency

```toml
#Cargo.toml

[dependencies]
sabry = {version = "0.0.1"}
```
And a proc-macro
```rust
// lib.rs
use sabry::sassy;

sassy!(mixins "tests/assets/mixins.scss");
sassy!(styles {"
    $primary-color: black;
    @mixin colored($col: primary) {
        @if $col == primary {
            color: $primary-color;
        } @else {
            color: $col;
        }
    }
"});
```
*\* Unlike most of other crates that do sass-in-the-rust, sabry currently does not allow unquoted sass/scss. You still have to write it in a string-quotes. Unquoted sass/scss is reserved for the future, where we shall introduce variable injection.*

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
```rust
// breadbadgelist.rs
use sabry::styly;

styly!(const styles {"
    .badges {
        display: flex;
        &__list {
            display: flex;
        }
    }
    #wolf {
        color: white;
    }
"});

/* Now you can use all scoped selectors: */
let badges_scoped_class = styles::badges;
let wolf_scoped_id = styles::thewolf;
let badges__list_scoped_class = styles::___list(styles::badges);
```
> That `const` usage is covered in the [following](#constant-styly-scopes) section. In this example we dont invoke sabry build-magic, so, to be as close to real life as possible, I used a const.

Every selector, if that does make sense, now available for you as a member of `styles` scope. In this example - `styles::badges`, `styles::thewolf` and `styles::_list()`. More about scoping and member names you can read [here](#styly-scopes).

### Use styles earlier created in another crate

> **Tip** | If you use [leptos](https://github.com/leptos-rs/leptos) - check [this](#leptos-specials) section as well

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
```rust
// breadbadgelist.rs
use sabry::styly;

styly!(styles {"
    // 'mixins' is available beacuse we did call mixins!() macro in the example below
    @use 'mixins';
    .badges {
        display: flex;
        &__list {
            display: flex;
        }
    }
    #wolf {
        @include mixins.colored(white);
    }
"});
```
So the `mixins!` macro we just passed to the `usey!` macro inside of `buildy` function call is now accessible with simple and natural `@use "mixins"` SASS rule!

### Leptos specials

With [leptos](https://github.com/leptos-rs/leptos) framework you can do the trick to apply some class to most of HtmlElements for the entire component/island:

```rust
use leptos::prelude::*;

#[component]
fn component() -> impl IntoView {
    view! {class="cls1", /* <- here */
        <h1>"Head"</h1>
        <p>"text"</p>
    }
}
```
... and have "cls1" on both `h1` and `p` auto-assigned. Isn't that cool?

So, if you're on leptos, and don't mind to take this approach, I'd highly recommend turning the *lepty-scoping* feature on:

```toml
# Cargo.toml

[dependencies]
sabry = {version = "0.0.1", features = ["lepty-scoping"]}

[build-dependencies]
sabry = {version = "0.0.1", features = ["build", "lepty-scoping"]}
```

```rust
use leptos::prelude::*;
use sabry::styly;

styly!(scope {"
    // assuming we have tokens!() macro generated by `sassy!`, which is handled by `buildy`
    @use 'tokens';
    h1 {
        @include tokens.sectionhead();
        img {
            @include tokens.sectionimg();
        }
    }
    .breadcumbs {
        @include tokens.badgelist();
        &__item {
            @include tokens.badge(secondary);
        }
    }
"});

#[component]
fn component() -> impl IntoView {
    view! {class=SCOPE,
        <h1>
            "Head"
            <img src="whatever"/>
        </h1>
        <ul class=scope::breadcumbs>
            <li class=scope::___item(scope::breadcumbs)>
                "Home page"
            </li>
        </ul>
    }
}
```

That will perform *much* better:

- All scope members - like `scope::breadcumbs` - wont contain repeating scope hash, just the original class/id selector
    - So instead of scope members (which [aren't](https://github.com/yiffyrusdev/sabry/issues/2) autocompleted for sass-in-the-rust yet) you could just write bare selectors, as you do, for example, with svelte: `<li class="breadcumbs__item">`
- You will not encounter [the catch](#scoping) with nested tagname selectors

Also, this isn't really exclusive leptos-supporting feature. It just changes scoping behavour.

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
```rust
use sabry::sassy;

sassy!(module1 {"$primary-color: red;"});
sassy!(module2 "tests/assets/mixins.scss");
sassy!(module3:sass "tests/assets/mixins.sass");
// works, but there are catches.
sassy!(module4:sass {"
    =colored($col: primary)
        @if $col == primary
            color: white
        @else
            color: red
"});
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
```rust
use sabry::styly;

styly!(private_fox {".fur {color: red; &-dark {color: black;}}"});
styly!(pub public_fox {".fur {color: red; &-dark {color: black;}}"});
styly!(pub const pub_compiletime_fox:sass {"
    .fur
        color: red
        &-dark
            color: black
"});
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

```rust
sabry::styly!(pub whatever "tests/assets/mixins.scss");
```

#### Constant styly scopes

As you've seen above, scope doe not contain any style code by itself. That's the use case i advise mostly.

However you could still compile styles into the artifact by simply adding the `const` to the macro call:

```rust
sabry::styly!(const scope "tests/assets/mixins.scss");
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
>
> **However** If you are able to apply some class to all html elements you have - like `view!{class=CLASS...}` with Leptos - you could use *lepty-scoping* feature flag for sabry and get rid of this catch! See more [here](#leptos-specials).

As for **SASS parent selectors**: they are currently handled in different way. Instead of
walking up the syntax tree sabry just creates function member for the scope and leave the rest to grass:

```rust
sabry::styly!(scope {"
    .cls1 {
        &-dark {}
    }
"});
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

#### Scope member naming rules

Not any valid CSS selector is a valid rust identifier. In general this section should not be needed, as you should receive autocompletion from the editor. *However* it doesn't seem to work properly. Check the [wip](#wip) section out.

## Notable feature flags

**build** - turns on the `sabry::buildy` function, along with the entire `sabry_build` crate where it lives

**internals** - exposes majority of internal stuff for you to experiment or build own workflow

**lepty-scoping** - overhauls the scope generation logic, best suitable for the leptos. Check out the [section](#leptos-specials) and an [example](https://github.com/yiffyrusdev/sabry/tree/master/examples/leptos-axum)

## WIP

*(sorted by my own priority)*, "dones" are excluded

- [x] Somehow achieve the autocompletion for scopes. The problem is explained in details [here](https://github.com/yiffyrusdev/sabry/issues/2)
    - [ ] some weird unrelated stuff I can see in autocompletion
- [ ] Crates of styled components - currently the only way to create them seems to be const styly.
    - [ ] CSS support
- [ ] Currently the crate causes "dependency inheritance" infection. We cant get rid of it, however should be doable to at least get rid of flag inheritance
- [ ] Use sass-in-rust without quoted styles for variable injection:
    ```rust, ignore
    let myvar = "content";
    styly!(scope {
        .c::after {
            content: somehow(myvar);
        }
    })
    ```
- [ ] There are some strange parsing errors, seems like a bug, however very hard to reproduce. Have to investigate. Maybe do more tests.
- [ ] Experience with cargo-leptos is fine, and we do use it, however its a bit "raughy". Need to do something about it.

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
