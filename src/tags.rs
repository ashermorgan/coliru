/// Checks if a set of tags matches a set of rules
///
/// Rules and tags are both specified as strings. Each rule contains one or more
/// desired tags separated by `|`. A list of tags matches a set of rules if each
/// rule contains at least one tag that appears in the list of tags. The results
/// of rules prefixed with `^` will be negated. An empty list of rules will
/// match any list of tags.
///
/// ```
/// let rules = ["linux|macos", "system", "^work"].map(|x| x.to_string());
/// let tags_1 = ["macos", "system", "user"].map(|x| x.to_string());
/// let tags_2 = ["linux", "system", "work"].map(|x| x.to_string());
/// assert_eq!(tags_match(&rules, &tags_1), true);
/// assert_eq!(tags_match(&rules, &tags_2), false);
/// ```
pub fn tags_match(rules: &[String], tags: &[String]) -> bool {
    for rule in rules.iter() {
        let is_negated = rule.chars().nth(0) == Some('^');
        let _rule: &str;
        if is_negated {
            _rule = &rule[1..]; // Strip leading '^'
        } else {
            _rule = &rule;
        }

        let tag_found = _rule.split("|").any(|subrule| {
            tags.iter().any(|tag| {
                tag == subrule
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
        let tags_1 = ["linux", "user"].map(|x| x.to_string());
        assert_eq!(tags_match(&[], &[]), true);
        assert_eq!(tags_match(&[], &tags_1), true);
        assert_eq!(tags_match(&tags_1, &[]), false);
    }

    #[test]
    fn tags_match_one_match() {
        let tags_1 = ["linux"].map(|x| x.to_string());
        let tags_2 = ["linux", "windows"].map(|x| x.to_string());

        assert_eq!(tags_match(&tags_1.clone(), &tags_1.clone()), true);
        assert_eq!(tags_match(&tags_1.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&tags_2.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&tags_2.clone(), &tags_2.clone()), true);
    }

    #[test]
    fn tags_match_two_matches() {
        let tags_1 = ["linux", "user"].map(|x| x.to_string());
        let tags_2 = ["linux", "user", "windows"].map(|x| x.to_string());

        assert_eq!(tags_match(&tags_1.clone(), &tags_1.clone()), true);
        assert_eq!(tags_match(&tags_1.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&tags_2.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&tags_2.clone(), &tags_2.clone()), true);
    }

    #[test]
    fn tags_match_negated() {
        let rules = ["^linux"].map(|x| x.to_string());
        let tags_1 = ["linux"].map(|x| x.to_string());
        let tags_2 = ["windows"].map(|x| x.to_string());
        let tags_3 = ["macos"].map(|x| x.to_string());
        let tags_4 = ["linux", "macos"].map(|x| x.to_string());

        assert_eq!(tags_match(&rules.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&rules.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&rules.clone(), &tags_3.clone()), true);
        assert_eq!(tags_match(&rules.clone(), &tags_4.clone()), false);
    }

    #[test]
    fn tags_match_negated_two_rules() {
        let rules_1 = ["^linux", "^user"].map(|x| x.to_string());
        let rules_2 = ["^linux", "user"].map(|x| x.to_string());
        let tags_1 = ["linux", "system"].map(|x| x.to_string());
        let tags_2 = ["windows", "user"].map(|x| x.to_string());
        let tags_3 = ["macos", "system"].map(|x| x.to_string());
        let tags_4 = ["linux", "macos", "user"].map(|x| x.to_string());

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
        let rules = ["linux|macos"].map(|x| x.to_string());
        let tags_1 = ["linux"].map(|x| x.to_string());
        let tags_2 = ["macos"].map(|x| x.to_string());
        let tags_3 = ["linux", "macos"].map(|x| x.to_string());
        let tags_4 = ["windows"].map(|x| x.to_string());

        assert_eq!(tags_match(&rules.clone(), &tags_1.clone()), true);
        assert_eq!(tags_match(&rules.clone(), &tags_2.clone()), true);
        assert_eq!(tags_match(&rules.clone(), &tags_3.clone()), true);
        assert_eq!(tags_match(&rules.clone(), &tags_4.clone()), false);
    }

    #[test]
    fn tags_match_union_two_rules() {
        let rules_1 = ["linux|macos", "user|system"].map(|x| x.to_string());
        let rules_2 = ["linux|macos", "user"].map(|x| x.to_string());
        let tags_1 = ["user", "linux"].map(|x| x.to_string());
        let tags_2 = ["system", "macos"].map(|x| x.to_string());
        let tags_3 = ["user", "linux", "macos"].map(|x| x.to_string());
        let tags_4 = ["system", "windows"].map(|x| x.to_string());

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
        let rules = ["^linux|macos"].map(|x| x.to_string());
        let tags_1 = ["linux"].map(|x| x.to_string());
        let tags_2 = ["macos"].map(|x| x.to_string());
        let tags_3 = ["linux", "macos"].map(|x| x.to_string());
        let tags_4 = ["windows"].map(|x| x.to_string());

        assert_eq!(tags_match(&rules.clone(), &tags_1.clone()), false);
        assert_eq!(tags_match(&rules.clone(), &tags_2.clone()), false);
        assert_eq!(tags_match(&rules.clone(), &tags_3.clone()), false);
        assert_eq!(tags_match(&rules.clone(), &tags_4.clone()), true);
    }

    #[test]
    fn tags_match_union_negated_two_rules() {
        let rules_1 = ["^linux|macos", "^user"].map(|x| x.to_string());
        let rules_2 = ["^linux|macos", "user|system"].map(|x| x.to_string());
        let rules_3 = ["^linux|macos", "user"].map(|x| x.to_string());
        let tags_1 = ["linux", "macos", "system"].map(|x| x.to_string());
        let tags_2 = ["windows", "user"].map(|x| x.to_string());
        let tags_3 = ["windows", "system"].map(|x| x.to_string());

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
