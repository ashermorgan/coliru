/// Checks if a list of tags matches a list of tag rules
///
/// Rules and tags are both specified as strings. Each rule contains one or more
/// desired tags separated by `|`. A list of tags matches a list of rules if
/// each rule contains at least one tag that appears in the list of tags. The
/// results of rules prefixed with `^` will be negated. Any list of tags will
/// match an empty list of tag rules. An empty list of tags will only match an
/// empty list of tag rules.
///
/// ```
/// let rules = ["linux|macos", "system", "^work"];
/// let tags_1 = ["macos", "system", "user"];
/// let tags_2 = ["linux", "system", "work"];
/// assert_eq!(tags_match(&rules, &tags_1), true);
/// assert_eq!(tags_match(&rules, &tags_2), false);
/// ```
pub fn tags_match<S: AsRef<str>>(rules: &[S], tags: &[S]) -> bool {
    for rule in rules.iter() {
        let mut _rule = rule.as_ref();
        let is_negated = _rule.chars().nth(0) == Some('^');
        if is_negated {
            _rule = &_rule[1..]; // Strip leading '^'
        }

        let tag_found = _rule.split("|").any(|subrule| {
            tags.iter().any(|tag| {
                tag.as_ref() == subrule
            })
        });

        if tag_found == is_negated {
            return false
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_match_empty_parameters() {
        let tags_1 = [];
        let tags_2 = ["linux", "user"];
        assert_eq!(tags_match(&tags_1, &tags_1), true);
        assert_eq!(tags_match(&tags_1, &tags_2), true);
        assert_eq!(tags_match(&tags_2, &tags_1), false);
    }

    #[test]
    fn tags_match_one_match() {
        let tags_1 = ["linux"];
        let tags_2 = ["linux", "windows"];

        assert_eq!(tags_match(&tags_1.clone(), &tags_1.clone()), true);
        assert_eq!(tags_match(&tags_1.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&tags_2.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&tags_2.clone(), &tags_2.clone()), true);
    }

    #[test]
    fn tags_match_two_matches() {
        let tags_1 = ["linux", "user"];
        let tags_2 = ["linux", "user", "windows"];

        assert_eq!(tags_match(&tags_1.clone(), &tags_1.clone()), true);
        assert_eq!(tags_match(&tags_1.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&tags_2.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&tags_2.clone(), &tags_2.clone()), true);
    }

    #[test]
    fn tags_match_negated() {
        let rules = ["^linux"];
        let tags_1 = ["linux"];
        let tags_2 = ["windows"];
        let tags_3 = ["macos"];
        let tags_4 = ["linux", "macos"];

        assert_eq!(tags_match(&rules.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&rules.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&rules.clone(), &tags_3.clone()), true);
        assert_eq!(tags_match(&rules.clone(), &tags_4.clone()), false);
    }

    #[test]
    fn tags_match_negated_two_rules() {
        let rules_1 = ["^linux", "^user"];
        let rules_2 = ["^linux", "user"];
        let tags_1 = ["linux", "system"];
        let tags_2 = ["windows", "user"];
        let tags_3 = ["macos", "system"];
        let tags_4 = ["linux", "macos", "user"];

        assert_eq!(tags_match(&rules_1.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&rules_1.clone(), &tags_2.clone()), false);
        assert_eq!(tags_match(&rules_1.clone(), &tags_3.clone()), true);
        assert_eq!(tags_match(&rules_1.clone(), &tags_4.clone()), false);
        assert_eq!(tags_match(&rules_2.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&rules_2.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&rules_2.clone(), &tags_3.clone()), false);
        assert_eq!(tags_match(&rules_2.clone(), &tags_4.clone()), false);
    }

    #[test]
    fn tags_match_union() {
        let rules = ["linux|macos"];
        let tags_1 = ["linux"];
        let tags_2 = ["macos"];
        let tags_3 = ["linux", "macos"];
        let tags_4 = ["windows"];

        assert_eq!(tags_match(&rules.clone(), &tags_1.clone()), true);
        assert_eq!(tags_match(&rules.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&rules.clone(), &tags_3.clone()), true);
        assert_eq!(tags_match(&rules.clone(), &tags_4.clone()), false);
    }

    #[test]
    fn tags_match_union_two_rules() {
        let rules_1 = ["linux|macos", "user|system"];
        let rules_2 = ["linux|macos", "user"];
        let tags_1 = ["user", "linux"];
        let tags_2 = ["system", "macos"];
        let tags_3 = ["user", "linux", "macos"];
        let tags_4 = ["system", "windows"];

        assert_eq!(tags_match(&rules_1.clone(), &tags_1.clone()), true);
        assert_eq!(tags_match(&rules_1.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&rules_1.clone(), &tags_3.clone()), true);
        assert_eq!(tags_match(&rules_1.clone(), &tags_4.clone()), false);
        assert_eq!(tags_match(&rules_2.clone(), &tags_1.clone()), true);
        assert_eq!(tags_match(&rules_2.clone(), &tags_2.clone()), false);
        assert_eq!(tags_match(&rules_2.clone(), &tags_3.clone()), true);
        assert_eq!(tags_match(&rules_2.clone(), &tags_4.clone()), false);
    }

    #[test]
    fn tags_match_union_negated() {
        let rules = ["^linux|macos"];
        let tags_1 = ["linux"];
        let tags_2 = ["macos"];
        let tags_3 = ["linux", "macos"];
        let tags_4 = ["windows"];

        assert_eq!(tags_match(&rules.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&rules.clone(), &tags_2.clone()), false);
        assert_eq!(tags_match(&rules.clone(), &tags_3.clone()), false);
        assert_eq!(tags_match(&rules.clone(), &tags_4.clone()), true);
    }

    #[test]
    fn tags_match_union_negated_two_rules() {
        let rules_1 = ["^linux|macos", "^user"];
        let rules_2 = ["^linux|macos", "user|system"];
        let rules_3 = ["^linux|macos", "user"];
        let tags_1 = ["linux", "macos", "system"];
        let tags_2 = ["windows", "user"];
        let tags_3 = ["windows", "system"];

        assert_eq!(tags_match(&rules_1.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&rules_1.clone(), &tags_2.clone()), false);
        assert_eq!(tags_match(&rules_1.clone(), &tags_3.clone()), true);
        assert_eq!(tags_match(&rules_2.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&rules_2.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&rules_2.clone(), &tags_3.clone()), true);
        assert_eq!(tags_match(&rules_3.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&rules_3.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&rules_3.clone(), &tags_3.clone()), false);
    }
}
