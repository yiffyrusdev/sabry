use leptos::prelude::*;
use sabry::styly;

styly!(style "src/routes/home.scss");

/// Renders the home page of your application.
#[component]
pub fn Route() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {class = STYLE,
        <section class=style::table>
            <h1>"Welcome to Leptos!"</h1>
            <button class=style::btn on:click=on_click>"Ima CRAZY button, clicked: " {count} " times!"</button>
            <button class=style::_dark(style::btn) on:click=on_click>"Ima CRAZY button, clicked: " {count} " times!"</button>
            <span class=style::card>"DISCLAIMER:"</span>
            <p class=style::pocwarn>"This isn't a design example! Its just proof of sabry's capabilities."</p>
            <p class=style::pocwarn>"Plz dont make websites/apps which look like this page :D"</p>
        </section>
    }
}
