use base64::Engine;

use crate::config::SabryHashConfig;

use super::{apply_basic_rusty_member_gen_rules, ArbitraryScope};

/// Convenience wrapper for String-being-a-hash
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopeHash(String);

impl ScopeHash {
    pub fn new(scope: &ArbitraryScope, config: &SabryHashConfig) -> Self {
        let mut hasher = blake3::Hasher::new();

        if config.use_scope_name {
            hasher.update(scope.name.to_string().as_bytes());
        }
        if config.use_code_text {
            hasher.update(scope.adapter().source().as_bytes());
        }
        if config.use_code_size {
            hasher.update(&scope.adapter().source().len().to_ne_bytes());
        }
        if config.use_item_names {
            let classes = scope.adapter().class_selectors();
            let ids = scope.adapter().id_selectors();
            //# types/tags dont participate
            //# child-parent dont participate

            let merged_items = classes
                .iter()
                .map(|s| &s.name)
                .chain(ids.iter().map(|i| &i.name))
                .map(|ident| ident.as_literal())
                .filter_map(|mbl| mbl.map(|l| l.raw))
                .collect::<String>();

            hasher.update(merged_items.as_bytes());
        }

        let size = if config.size >= blake3::OUT_LEN {
            blake3::OUT_LEN
        } else {
            config.size
        };

        let hash = hasher.finalize();
        let hash = hash.as_bytes();

        //? This isn't right. "size" is size of hash in symbols.
        // naaah, its fine :D (Dan)
        let hash = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(&hash[..size]);
        let hash = apply_basic_rusty_member_gen_rules(&hash);

        Self(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[cfg(test)]
    pub fn test_init(v: String) -> Self {
        Self(v)
    }
}

#[cfg(test)]
mod test {
    use syn::Ident;

    use crate::{config::SabryHashConfig, scoper::ArbitraryScope, syntax::ostrta::OneSyntaxToRuleThemAll};

    use super::ScopeHash;

    #[test]
    fn hash_matches() {
        let source = ".cls1{color:red; &-dark{color: black} #id1 {color:green;} div {color:blue;}} .cls3#id2{color: black;}";

        let scope1 = ArbitraryScope::from_source(
            OneSyntaxToRuleThemAll::Scss,
            syn::parse_str::<Ident>("lasifudm").unwrap(),
            source,
        )
        .unwrap();
        let hash1 = ScopeHash::new(&scope1, &SabryHashConfig::default());

        let scope2 = ArbitraryScope::from_source(
            OneSyntaxToRuleThemAll::Scss,
            syn::parse_str::<Ident>("lasifudm").unwrap(),
            source,
        )
        .unwrap();
        let hash2 = ScopeHash::new(&scope2, &SabryHashConfig::default());

        assert_eq!(hash1, hash2);
    }
}
