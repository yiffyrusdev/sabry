use leptos::prelude::*;
use leptos_meta::Style;
use sabry::styly;

#[component]
pub fn Aside() -> impl IntoView {
    let showme = RwSignal::new(true);

    view!{class=SCOPE,
        <Style>{SCOPE_CSS}</Style>
        <Show when=move|| *showme.read()>
            <aside>
                <h1>"Welcome to 'sabried' leptos!"</h1>
                <p>"This is the 'Aside' component"</p>
                <p>"which we declared in another crate"</p>
                <p>"styled with sabry too!"</p>
                <br/>
                <button class="btn" on:click=move|_| *showme.write() = false>
                    "Click me to hide aside!"
                </button>
            </aside>
        </Show>
    }
}

styly!(const scope:scss {"
    @use 'theme';
    @use 'utils';
    @use 'tokens';

    aside {
        @include theme.surface(secondary);
        @include utils.flex(column, center, center);

        position: fixed;
        top: 0;
        left: 0;
        bottom: 0;
        width: max-content;
        z-index: 1000;
    }

    p {
        @include theme.txt(primary);
        font-size: 1rem;
    }

    .btn {
        @include theme.surface(accent);
        @include tokens.clickable();
    }
"});
