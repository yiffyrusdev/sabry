use raffia::{
    ast::{
        ClassSelector, CompoundSelector, IdSelector, NestingSelector, SimpleSelector, Statement,
        Stylesheet, TypeSelector,
    },
    ParserBuilder, Syntax,
};

/// Convenience wrapper for [Stylesheet]
pub struct StylesheetAdapter<'s> {
    pub syntax: Syntax,
    source: &'s str,
    stylesheet: Stylesheet<'s>,
}

impl<'s> StylesheetAdapter<'s> {
    pub fn new(source: &'s str, syntax: Syntax) -> Result<Self, raffia::error::Error> {
        Ok(Self {
            stylesheet: ParserBuilder::new(source)
                .ignore_comments()
                .syntax(syntax)
                .build()
                .parse()?,
            syntax,
            source,
        })
    }

    pub fn source(&self) -> &str {
        self.source
    }

    pub fn selectors(&self) -> Vec<CompoundSelector<'s>> {
        Self::selectors_of(self.stylesheet.statements.iter().cloned()).collect()
    }

    pub fn class_selectors(&self) -> Vec<ClassSelector<'s>> {
        self.selectors_by(|c| c.as_class().cloned())
    }

    pub fn id_selectors(&self) -> Vec<IdSelector<'s>> {
        self.selectors_by(|c| c.as_id().cloned())
    }

    pub fn type_selectors(&self) -> Vec<TypeSelector<'s>> {
        self.selectors_by(|c| c.as_type().cloned())
    }

    pub fn nesting_selectors(&self) -> Vec<NestingSelector<'s>> {
        self.selectors_by(|c| c.as_nesting().cloned())
    }

    /// Return collected [Vec] of owned values modulo `T`, filtered by `F`.
    ///
    /// This is preferred way to apply custom filtering on selectors over
    /// `selectors().iter().filter_map()`
    /// as it does not allocate intermediate vector
    pub fn selectors_by<T, F>(&self, form: F) -> Vec<T>
    where
        F: FnMut(SimpleSelector<'s>) -> Option<T>,
    {
        Self::selectors_of(self.stylesheet.statements.iter().cloned())
            .flat_map(|cs| cs.children)
            .filter_map(form)
            .collect()
    }

    fn selectors_of(
        c: impl Iterator<Item = Statement<'s>>,
    ) -> impl Iterator<Item = CompoundSelector<'s>> {
        // Hello. Its Dan.
        // Im not proud of this,
        // but more effective implementation
        // was SO unreadable and SO infected with lifetimes
        // I suck at those things

        // For everyone trying to make this MORE OPTIMAL:
        // increment the hour-counter and post your name here:
        // 4 - Danik
        // 5 - Yiffy
        // 8 - Danik
        c.filter_map(|s| s.as_qualified_rule().cloned())
            .flat_map(|q| {
                let thissels = q
                    .selector
                    .selectors
                    .iter()
                    .flat_map(|sel| &sel.children)
                    .filter_map(|sc| sc.as_compound_selector())
                    .cloned();

                let blockstmt = q.block.statements.iter().cloned();
                Self::selectors_of(blockstmt)
                    .chain(thissels)
                    .collect::<Vec<_>>()
            })
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use raffia::Spanned;

    use super::StylesheetAdapter;

    #[test]
    fn sels() {
        let source_scss = "
.r1{
  .r2.r3 {
    color: red;
  }
  &-rod {
    color: red;
    &[value] {
      color: red;
    }
  }
}
div#id2 {
  & > span.seled {
    color: red;
    & + ul#id2[attr] {
      color: red;
    }
    & + .uled[attr] {
      color: red;
    }
  }
  &::after{
    color: red;
  }
}
        ";
        let source_sass = "
.r1
    .r2.r3
        color: red
    &-rod
        color: red
        &[value]
            color: red
div#id2
    & > span.seled
        color: red
        & + ul#id2[attr]
            color: red
        & + .uled[attr]
            color: red
    &::after
        color: red
        ";
        let expect_classes = HashSet::from(["r2", "r3", "r1", "uled", "seled"]);
        let expect_ids = HashSet::from(["id2", "id2"]);
        let expect_tags = HashSet::from(["ul", "div", "span"]);

        //scss
        let adp = StylesheetAdapter::new(source_scss, raffia::Syntax::Scss).unwrap();

        let classes = adp
            .class_selectors()
            .iter()
            .filter_map(|c| c.name.as_literal().map(|l| l.raw))
            .collect::<HashSet<_>>();

        let ids = adp
            .id_selectors()
            .iter()
            .filter_map(|i| i.name.as_literal().map(|l| l.raw))
            .collect::<HashSet<_>>();

        let tags = adp
            .type_selectors()
            .iter()
            .filter_map(|t| t.as_tag_name())
            .filter_map(|t| t.name.name.as_literal().map(|l| l.raw))
            .collect::<HashSet<_>>();

        assert_eq!(expect_classes, classes);
        assert_eq!(expect_ids, ids);
        assert_eq!(expect_tags, tags);

        //sass
        let adp = StylesheetAdapter::new(source_sass, raffia::Syntax::Sass).unwrap();

        let classes = adp
            .class_selectors()
            .iter()
            .filter_map(|c| c.name.as_literal().map(|l| l.raw))
            .collect::<HashSet<_>>();

        let ids = adp
            .id_selectors()
            .iter()
            .filter_map(|i| i.name.as_literal().map(|l| l.raw))
            .collect::<HashSet<_>>();

        let tags = adp
            .type_selectors()
            .iter()
            .filter_map(|t| t.as_tag_name())
            .filter_map(|t| t.name.name.as_literal().map(|l| l.raw))
            .collect::<HashSet<_>>();

        assert_eq!(expect_classes, classes);
        assert_eq!(expect_ids, ids);
        assert_eq!(expect_tags, tags);
    }

    #[test]
    fn rename_class() {
        let scss = ".cls1{color:red; .cls2{color:green;}}";
        let newname = "cls203-jg7ihgjhftyfhjh";
        let expect = ".cls1{color:red; .cls203-jg7ihgjhftyfhjh{color:green;}}";

        let adp = StylesheetAdapter::new(scss, raffia::Syntax::Scss).unwrap();
        let classes = adp.class_selectors();
        let target_class = classes[0].clone();

        let span = target_class.name.span();
        let renamed = format!("{}{}{}", &scss[..span.start], newname, &scss[span.end..]);
        assert_eq!(renamed, expect);
    }
}
