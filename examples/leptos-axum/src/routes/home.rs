use leptos::prelude::*;
use sabry::styly;

/// Renders the home page of your application.
#[component]
pub fn Route() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {class = STYLE,
        <leptos_components::Aside/> // lets use our component
        <section class="table">
            <h2>"Welcome to Leptos"</h2>
            <h2>
                "This head is styled with lepty-scopes feature"
                <br/>
                <span>"And this child too!"</span>
            </h2>

            <button class="btn" on:click=on_click>"Ima CRAZY button, clicked: " {count} " times!"</button>
            <button class="btn btn-dark" on:click=on_click>"Ima CRAZY button, clicked: " {count} " times!"</button>
            <span class="card">"DISCLAIMER:"</span>
            <p class="pocwarn">"This isn't a design example! Its just proof of sabry's capabilities."</p>
            <p class="pocwarn">"Plz dont make websites/apps which look like this page :D"</p>
        </section>
    }
}

styly!(style {"
    @use 'tokens';
    @use 'utils';
    @use 'theme';

    h2 {
        margin-bottom: 1rem;
        @include theme.surface(primary);
        @include theme.txt(primary);
        span {
            @include theme.txt(accent);
        }
    }

    .btn {
        @include tokens.clickable(accent);
        &-dark {
            @include tokens.clickable(secondary, 0.8, 1.1);
        }
    }

    .table {
        @include utils.flex(column, center, center);
        @include theme.surface(primary);
    }

    .card {
        @include theme.surface(secondary);
    }
    .pocwarn {
        @include theme.surface(secondary);
        text-transform: uppercase;
    }
"});
