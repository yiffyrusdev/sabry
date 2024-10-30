use leptos::prelude::*;
use sabry::styly;

styly!(s "src/routes/home.scss");

styly!(s2 {
    @use "theme";
    .card {
        @include theme.surface(secondary);
    }
    .pocwarn {
        @include theme.surface(secondary);
        text-transform: uppercase;
    }
});

/// Renders the home page of your application.
#[component]
pub fn Route() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <section class=s::table>
            <h1>"Welcome to Leptos!"</h1>
            <button class=s::btn on:click=on_click>"Ima CRAZY button, clicked: " {count} " times!"</button>
            <span class=s2::card>"DISCLAIMER:"</span>
            <p class=s2::pocwarn>"This isn't a design example! Its just proof of sabry's capabilities."</p>
            <p class=s2::pocwarn>"Plz dont make websites/apps which look like this page :D"</p>
        </section>
    }
}
