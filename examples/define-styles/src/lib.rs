use sabry::sassy;

sassy!(style "src/style.scss");

sassy!(utils {
    @mixin flex($direction, $align, $justify, $gap: .5rem) {
        display: flex;
        list-style: none;
        gap: $gap;
        flex-direction: $direction;

        @if $align == start{
            align-items: flex-start;
        }
        @else if $align == end{
            align-items: flex-end;
        }
        @else{
            align-items: $align;
        }

        @if $justify == start {
            justify-content: flex-start;
        }
        @else if $justify == end {
            justify-content: flex-end;
        }
        @else {
            justify-content: $justify;
        }
    }
});

sassy!(theme {
    $thm-surface-primary: #2b2a29;
    $thm-surface-secondary: #6C6C6C;
    $thm-surface-tertiary: #4C4C4C;
    $thm-surface-accent: #B0CB1F;

    $thm-text-primary: #ffffff;
    $thm-text-secondary: #ffffff;
    $thm-text-tertiary: #ffffff;
    $thm-text-accent: #B0CB1F;

    @mixin surface($style: primary) {
        @if $style == primary {
            @include txt(primary);
            background-color: $thm-surface-primary;
        }
        @else if $style == secondary {
            @include txt(secondary);
            background-color: $thm-surface-secondary;
        }
        @else if $style == accent {
            @include txt;
            background-color: $thm-surface-accent;
        }
    }

    @mixin txt($style: primary) {
        @if $style == primary {
            font-weight: 500;
            color: $thm-text-primary;
        }
        @else if $style == secondary {
            font-weight: 400;
            color: $thm-text-secondary;
        }
        @else if $style == tertiary {
            font-weight: 400;
            color: $thm-text-tertiary;
        }
        @else if $style == accent {
            font-weight: 500;
            color: $thm-text-accent;
        }
    }
});

sassy!(tokens {
    @use "theme";

    @mixin clickable($style: accent, $scale: 1.2, $clickscale: 0.9){
        @if $style == accent{
            @include theme.surface(accent);
        }
        @else if $style == secondary {
            @include theme.surface(secondary);
        }

        cursor: pointer;
        outline: none;
        -webkit-tap-highlight-color: #00000000;
        transition: transform .07s, filter .07s;
        &:hover{
            transform: scale($scale);
        }
        &:focus-visible {
            outline: 2px solid #fff;
        }
        &:active {
            filter: brightness(.9);
            transform: scale($clickscale);
        }
    }
});
