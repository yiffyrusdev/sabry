use std::{
    collections::HashSet,
    convert::Infallible,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
    vec,
};

use sabry_intrnl::{
    compiler::{CompilerAdapter, SabryCompilerError},
    config::{manifest::ManifestError, BehavHashCollision, BehavSassModCollision, SabryConfig},
    scoper::{hash::ScopeHash, ArbitraryScope, ScopeError},
};
use sabry_procmacro_impl::impls::{styly, ArbitraryStyleSyntax};
use walkdir::WalkDir;

use crate::filevisit::{self, FileVisitError};

type ModuleName = String;
type ModuleCode = String;
type StyleModule = (ModuleName, ModuleCode);
type BuilderResult = Result<(), SabryBuildError>;

/// Entry point of sabrys build-magic
///
/// Example:
/// ```
/// # use sabry_build::buildmagic::buildy;
/// /* buildy call */
/// buildy(
///     vec![("mixins.scss".to_string(), "@mixin abc(){}".to_string())]
/// ).expect("Sabry failed to build CSS");
/// ```
///
/// Also you could use style-macros defined anywhere with `sabry::sassy!` procmacro:
/// ```ignore
/// buildy(
///     sabry::usey!(
///         crate1::mixins!(),
///         crate2::tokens!(),
///         crate3::mixins!()
///     )
/// ).expect("Sabry failed to build CSS");
/// ```
///
/// This function is intended to run at build time, however, you're free to use it however you please
pub fn buildy(inline_side_modules: impl IntoIterator<Item = StyleModule>) -> BuilderResult {
    println!("🧙: This is probably the stderr. Something went wrong:");

    let config = SabryConfig::require()?;
    let mut builder = SabryBuilder::new(config);

    println!("🧙 loading preludes");
    builder.load_preludes()?;

    println!("🧙 loading `buildy` modules");
    for (name, code) in inline_side_modules {
        builder.load_side_module(name, code)?;
    }

    println!("🧙 loading this crate");
    builder.load_styles_from_this_crate()?;

    println!("🧙 compiling CSS");
    builder.compile_everything()?;

    println!("🧙 writing an output");
    builder.generate_output()?;

    Ok(())
}

/// Entrypoint structure of sabrys build-magic.
///
/// You could construct CSS compilation process by yourself instead of using `buildy` function:
/// ```rust
/// # use sabry_intrnl::config::SabryConfig;
/// # use sabry_build::buildmagic::SabryBuilder;
/// let config = SabryConfig::require().expect("Config didnt load");
/// let builder = SabryBuilder::new(config);
/// ```
///
/// the [buildy] function will just do baseline usage of this structure for you
pub struct SabryBuilder {
    config: SabryConfig,
    css_compiler: CompilerAdapter,
    state: SabryBuildState,
}

//🧙
impl SabryBuilder {
    /// Construct new [SabryBuilder] from the config
    ///
    /// You can require [SabryConfig] automatically, with the [Result], by:
    ///
    /// ```
    /// # use sabry_intrnl::config::SabryConfig;
    /// SabryConfig::require().expect("Config didnt load");
    /// ```
    pub fn new(config: SabryConfig) -> Self {
        let css_compiler = CompilerAdapter::new(config.clone());
        Self {
            config,
            css_compiler,
            state: SabryBuildState::default(),
        }
    }

    /// Write all the loaded CSS
    ///
    /// - Bundle file (if configured)
    /// - Scope chunks (if out-dir is configured)
    pub fn generate_output(&mut self) -> BuilderResult {
        // warn on empty loaded_css_modules
        if self.state.loaded_css_modules.is_empty() {
            println!("🧙 sabry didn't compile any CSS. Perhaps the crate has no styles? Lets write some!\nAlso you should check that you do use `styly!` macro properly - at the top level as an item.");
        }

        if let Some(scope_dir) = &self.config.css.scopes {
            println!("🧙 writing CSS files for each of loaded scopes into {scope_dir}");

            fs::create_dir_all(scope_dir)?;
            for (scope, code) in &self.state.loaded_css_modules {
                let scope_path = format!("{}/{}.css", scope_dir, scope);
                fs::write(scope_path, code)?;
            }
        }

        if let Some(bundle_file) = &self.config.css.bundle {
            println!("🧙 writing merged CSS for the entire crate into {bundle_file}");

            let path = PathBuf::from_str(bundle_file)?;

            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }

            // remove the file to write new fresh tasty CSS
            let _ = fs::remove_file(&path);
            // and then reopen it
            let mut file = OpenOptions::new()
                .create_new(true)
                .append(true)
                .open(path)?;

            for (_, code) in &self.state.loaded_css_modules {
                write!(file, " {}", self.state.css_prelude)?;
                write!(file, " {code}")?;
            }
        }

        Ok(())
    }

    /// Compile all the loaded styles, SASS/SCSS/CSS, without actually writing them.
    pub fn compile_everything(&mut self) -> BuilderResult {
        // warn on empty known_side_modules and loaded_stylyses
        if self.state.known_side_modules.is_empty() {
            println!("🧙 sabry didn't load any side modules");
        }
        if self.state.loaded_stylyses.is_empty() {
            println!("🧙 sabry didn't load any usable styles");
        }

        // compile styly! macro parsed styles
        for styly in &self.state.loaded_stylyses {
            let scope = ArbitraryScope::from_source(
                styly.syntax.into(),
                styly.scope.clone(),
                styly.code.code(),
            )?
            .hashed(&self.config.hash)?;

            match self.config.hash.collision {
                BehavHashCollision::Ignore => {}
                BehavHashCollision::Error => {
                    if self.state.known_scope_hashes.contains(&scope.hash) {
                        return Err(SabryBuildError::HashCollision {
                            scope: scope.original_scope.name.to_string(),
                        });
                    }
                    self.state.known_scope_hashes.insert(scope.hash.clone());
                }
            }

            let css = self.css_compiler.compile_module(
                scope.original_scope.adapter().syntax.into(),
                &scope.hashed_code,
            )?;
            self.state
                .loaded_css_modules
                .push((scope.original_scope.name.to_string(), css));
        }

        // compile sass preludes into the CSS prelude
        for pre in &self.state.sass_prelude {
            let css = self.css_compiler.compile_module(pre.syntax.into(), &pre.code)?;
            self.state.css_prelude.push_str(&css);
        }

        Ok(())
    }

    /// Visit all the source files in the current crate and look for code that may affect building process:
    ///
    /// - `styly!` macro calls
    ///
    /// Only files with an **.rs** extension are visited.
    ///
    /// All the gathered info is loaded into state
    pub fn load_styles_from_this_crate(&mut self) -> BuilderResult {
        println!("🧙 scanning the crate");

        let root = WalkDir::new(&self.config.sass.scanroot);

        for entry in root {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                let entry_path = entry.path();
                let ext = entry_path.extension().unwrap_or_default();
                if ext == "rs" {
                    let visitor = filevisit::visit_file(entry_path)?;
                    self.state.loaded_stylyses.extend(visitor.found_stylys);
                }
            } else if metadata.is_symlink() {
                println!(
                    "🧙 sabry won't go through symlinks currently ('{}' is a symlink)",
                    entry.path().to_string_lossy()
                )
            }
        }

        Ok(())
    }

    /// Load up configured preludes and side modules
    pub fn load_preludes(&mut self) -> BuilderResult {
        // load sass modules from config
        let mut modules: Vec<StyleModule> = vec![];
        if let Some(sass_mods) = &self.config.sass.modules {
            for pre in sass_mods {
                let pre_path = PathBuf::from_str(pre)?;
                let pre_name = pre_path
                    .file_name()
                    .ok_or(SabryBuildError::FileName())?
                    .to_string_lossy()
                    .to_string();
                let code = fs::read_to_string(pre_path)?;
                modules.push((pre_name, code));
            }
        }
        for (name, code) in modules {
            self.load_side_module(name, code)?;
        }

        // load SASS preludes
        if let Some(sass_pres) = &self.config.sass.prelude {
            let mut sass_preludes: Vec<SassPreludeModule> = vec![];
            for pre in sass_pres {
                let pre_path = PathBuf::from_str(pre)?;

                let syntax = pre_path.extension().unwrap_or_default().to_str().unwrap_or_default();
                let syntax = match ArbitraryStyleSyntax::try_from(syntax) {
                    Ok(s) => s,
                    Err(_) => return Err(SabryBuildError::Another(format!("Unknown syntax for sass prelude {pre}")))
                };

                let code = fs::read_to_string(pre_path)?;
                sass_preludes.push(SassPreludeModule {syntax, code});
            }
            self.state.sass_prelude.extend(sass_preludes);
        }

        // load css preludes
        if let Some(css_pre) = &self.config.css.prelude {
            for pre in css_pre {
                let code = fs::read_to_string(pre)?;
                self.state.css_prelude.push_str(&code);
            }
        }

        Ok(())
    }

    /// Load up the SASS module by its `name` and fill the according file by its `code` as-is
    ///
    /// Module "loading" is simply a file creation. File is created in the configured `intermediate_dir`
    /// directory.
    ///
    /// Module `name` is not suffixed with syntax/extensions/whatever so it already should have necessary file extensions.
    ///
    /// This function will not overwrite the module file. Never.
    ///
    pub fn load_side_module(&mut self, name: ModuleName, code: ModuleCode) -> BuilderResult {
        println!("🧙 loading side module '{}'", &name);

        let module_file_path = format!("{}/{}", &self.config.sass.intermediate_dir, &name);
        let mut open_options = OpenOptions::new();
        let mut open_options = open_options.write(true);

        // prematurelly remove module file if the module is not yet known to be loaded
        if !self.state.known_side_modules.contains(&name) {
            let _ = fs::remove_file(&module_file_path);
        }

        open_options = if self.state.known_side_modules.contains(&name) {
            match &self.config.sass.module_name_collision {
                BehavSassModCollision::Error => {
                    return Err(SabryBuildError::ModuleCollision { module: name })
                }
                BehavSassModCollision::Merge => {
                    println!("🧙 sabry found duplicate module name '{name}': merging code, as configured");
                    open_options.append(true)
                }
            }
        } else {
            open_options.create(true)
        };

        fs::create_dir_all(&self.config.sass.intermediate_dir)?;
        let mut module_file = open_options.open(module_file_path)?;

        write!(module_file, "\n{code}\n")?;

        self.state.known_side_modules.insert(name);

        Ok(())
    }
}

#[derive(Default)]
pub struct SabryBuildState {
    /// HashSet of scope hashes known by builder
    /// Used to determine hash collision
    known_scope_hashes: HashSet<ScopeHash>,
    /// HashSet of module names known by builder
    /// Used to determine module name collision
    known_side_modules: HashSet<ModuleName>,
    /// styly! macro uses, parsed
    loaded_stylyses: Vec<styly::MacroSyntax>,
    /// CSS modules to form bundle/write separately
    loaded_css_modules: Vec<StyleModule>,
    /// CSS prelude to write into bundle
    /// Lives separately from [loaded_css_modules] to avoid name collision
    css_prelude: String,
    /// SASS preludes loaded from config
    /// Should be compiled into loaded_css_modules as well
    sass_prelude: Vec<SassPreludeModule>
}

/// Convenience struct for [SabryBuildState::sass_prelude]
pub struct SassPreludeModule {
    /// syntax for the prelude
    syntax: ArbitraryStyleSyntax,
    /// code of the prelude
    code: String
}

#[derive(Debug, thiserror::Error)]
pub enum SabryBuildError {
    #[error("Filesystem error")]
    Fs(#[from] io::Error),
    #[error("Failed to walk through the crate")]
    CrateWalk(#[from] walkdir::Error),
    #[error("Failed to load side SASS module")]
    LoadSass(),
    #[error(
        "Module with the similar name was already loaded, and sabry is configured to raise an error"
    )]
    ModuleCollision { module: ModuleName },
    #[error("Sabry cant understand the file")]
    FileVisit(#[from] FileVisitError),
    #[error("Syntax of style can not be parsed")]
    SyntaxError(#[from] ScopeError),
    #[error("Another scope has the same hash, and sabry is configured to raise an error. Try to adjust config, increase hash size, or change the style code")]
    HashCollision { scope: ModuleName },
    #[error("Something's wrong with file path")]
    Path(#[from] Infallible),
    #[error("File name wasnt determined properly")]
    FileName(),
    #[error("Failed to compile CSS")]
    CssCompile(#[from] SabryCompilerError),
    #[error("Failed to load config/manifest")]
    Manifest(#[from] ManifestError),
    #[error("Another error")]
    Another(String)
}
