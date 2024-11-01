/// Convenience unification entrypoint
/// to make friends with all the different syntax-consumers
#[derive(Debug, Clone, Copy)]
pub enum OneSyntaxToRuleThemAll {
    Sass,
    Scss
}

impl Default for OneSyntaxToRuleThemAll {
    fn default() -> Self {
        Self::Scss
    }
}

impl TryFrom<&str> for OneSyntaxToRuleThemAll {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sass" => Ok(Self::Sass),
            "scss" => Ok(Self::Scss),
            _ => Err(())
        }
    }
}

impl TryFrom<raffia::Syntax> for OneSyntaxToRuleThemAll {
    type Error = ();
    fn try_from(value: raffia::Syntax) -> Result<Self, Self::Error> {
        match value {
            raffia::Syntax::Sass => Ok(Self::Sass),
            raffia::Syntax::Scss => Ok(Self::Scss),
            _ => Err(())
        }
    }
}

impl TryFrom<grass::InputSyntax> for OneSyntaxToRuleThemAll {
    type Error = ();
    fn try_from(value: grass::InputSyntax) -> Result<Self, Self::Error> {
        match value {
            grass::InputSyntax::Sass => Ok(Self::Sass),
            grass::InputSyntax::Scss => Ok(Self::Scss),
            _ => Err(())
        }
    }
}

impl From<OneSyntaxToRuleThemAll> for raffia::Syntax {
    fn from(value: OneSyntaxToRuleThemAll) -> Self {
        match value {
            OneSyntaxToRuleThemAll::Sass => Self::Sass,
            OneSyntaxToRuleThemAll::Scss => Self::Scss
        }
    }
}

impl From<OneSyntaxToRuleThemAll> for grass::InputSyntax {
    fn from(value: OneSyntaxToRuleThemAll) -> Self {
        match value {
            OneSyntaxToRuleThemAll::Scss => Self::Scss,
            OneSyntaxToRuleThemAll::Sass => Self::Sass
        }
    }
}
