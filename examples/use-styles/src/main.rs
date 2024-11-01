use sabry::styly;

styly!(pub const bundlemod {
    @use "tokens";
    .someth {
        @include tokens.clickable;
        background-color: black;
        &__dark {
            background-color: white;
        }
    }
});

fn main(){
    println!("someth class codemod styles: {}", bundlemod::someth);
    println!("someth class filemod styles: {}", filemod::someth);
    //println!("someth-dark selector will be: {}", bundlemod::_dark(bundlemod::someth));
    println!("Thats how we used the styles! Good luck!!");
}
