use sabry::styly;

styly!(pub const bundlemod {
    @use "tokens";
    .someth {
        @include tokens.clickable;
        background-color: black;
    }
});

styly!(pub const filemod:sass "src/main.sass");

fn main(){
    println!("someth class codemod styles: {}", bundlemod::someth);
    println!("someth class filemod styles: {}", filemod::someth);
    println!("Thats how we used the styles! Good luck!!");
}
