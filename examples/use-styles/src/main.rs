use sabry::styly;

// Sabry now (0.0.1-alpha.3) disallows unquoted styles
// you won't benefit from unqouted styles though, as you won't get syntax highlighting
// Also, unqouted styles are reserved for future
styly!(pub const bundlemod {"
    @use 'tokens';
    .someth {
        @include tokens.clickable;
        background-color: black;
        &__dark {
            background-color: white;
        }
    }
    #fox {
        color: red;
    }
"});

// relative path is available because 'nightly' feature flag is set.
// If you don't - this will be "src/main.sass"
styly!(pub const filemod:sass "./main.sass");

fn main() {
    println!("someth class codemod styles: {}", bundlemod::someth);
    println!("someth class filemod styles: {}", filemod::someth);
    println!(
        "someth-dark selector will be: {}",
        bundlemod::___dark(bundlemod::someth)
    );
    println!("#fox id will be: {}", bundlemod::thefox);
    println!("Bundles const CSS will be: {}", BUNDLEMOD_CSS);
    println!("Bundles const CSS from file will be: {}", FILEMOD_CSS); // false-positive from relative file.
    println!("Thats how we used the styles! Good luck!!");
}
