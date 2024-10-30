use define_styles::{theme, tokens, utils as utils_dup};
use sabry::{sassy, usey};

// This module will have the same name as define_styles_example::utils
// and sabry is configured to merge them
sassy!(utils {
    @mixin nonclickable(){
        display: none;
    }
});

fn main(){
    /* Declare modules to use with sabry */
    sabry::buildy(
        usey!(
            theme!(),
            tokens!(),
            utils_dup!(),
            utils!()
        )
    ).expect("Failed to build sabry styles");
}
