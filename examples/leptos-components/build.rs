use define_styles::{theme, utils, tokens};

fn main(){
    sabry::buildy(
        sabry::usey!(
            utils!(),
            theme!(),
            tokens!()
        )
    ).expect("Failed to build with sabry");
}
