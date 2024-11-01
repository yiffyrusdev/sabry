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

styly!(pub filemod:sass "src/main.sass");

fn main(){
    println!("someth class codemod styles: {}", bundlemod::someth);
    println!("someth class filemod styles: {}", filemod::someth);
    println!("someth-dark selector will be: {}", bundlemod::___dark(bundlemod::someth));
    println!("#fox id will be: {}", bundlemod::thefox);
    println!("Thats how we used the styles! Good luck!!");
}
