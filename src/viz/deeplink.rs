//! Hash-based deep links from the notes into a specific screen.
//!
//! GitHub Pages serves a static site, so the target is encoded in the URL
//! fragment instead of a path that needs a server rewrite. Example:
//! `https://rvoidex7.github.io/r7crypto/#/learn/nist-frequency`.

use super::Tab;

/// A lesson inside the Learn tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lesson {
    XorCipher,
    ShannonEntropy,
    NistFrequency,
}

/// The screen a fragment asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub tab: Tab,
    pub lesson: Option<Lesson>,
}

/// Parse a `window.location.hash` value. Unknown or partial fragments are
/// ignored so a stale link still opens the app instead of failing to start.
pub fn parse(fragment: &str) -> Option<Target> {
    let path = fragment.strip_prefix("#/")?;
    let mut parts = path.split('/').filter(|part| !part.is_empty());
    let section = parts.next()?;

    let target = match section {
        "use" => Target {
            tab: Tab::Use,
            lesson: None,
        },
        "test" => Target {
            tab: Tab::Test,
            lesson: None,
        },
        "benchmark" => Target {
            tab: Tab::Benchmark,
            lesson: None,
        },
        "avalanche" => Target {
            tab: Tab::Avalanche,
            lesson: None,
        },
        "learn" => {
            let lesson = match parts.next()? {
                "xor-cipher" => Lesson::XorCipher,
                "shannon-entropy" => Lesson::ShannonEntropy,
                "nist-frequency" => Lesson::NistFrequency,
                _ => return None,
            };
            Target {
                tab: Tab::Learn,
                lesson: Some(lesson),
            }
        }
        _ => return None,
    };

    // Reject trailing junk such as `#/learn/nist-frequency/typo` rather than
    // silently treating it as a valid link.
    if parts.next().is_some() {
        return None;
    }
    Some(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learn_fragments_select_the_exact_lesson() {
        assert_eq!(
            parse("#/learn/nist-frequency"),
            Some(Target {
                tab: Tab::Learn,
                lesson: Some(Lesson::NistFrequency),
            })
        );
        assert_eq!(
            parse("#/learn/shannon-entropy"),
            Some(Target {
                tab: Tab::Learn,
                lesson: Some(Lesson::ShannonEntropy),
            })
        );
        assert_eq!(
            parse("#/learn/xor-cipher"),
            Some(Target {
                tab: Tab::Learn,
                lesson: Some(Lesson::XorCipher),
            })
        );
    }

    #[test]
    fn every_tab_has_a_fragment() {
        for (fragment, tab) in [
            ("#/use", Tab::Use),
            ("#/test", Tab::Test),
            ("#/benchmark", Tab::Benchmark),
            ("#/avalanche", Tab::Avalanche),
        ] {
            assert_eq!(parse(fragment), Some(Target { tab, lesson: None }));
        }
    }

    #[test]
    fn malformed_fragments_are_ignored() {
        for fragment in [
            "",
            "#",
            "#/",
            "#/learn",
            "#/learn/unknown",
            "#/bogus",
            "#/test/extra",
        ] {
            assert_eq!(parse(fragment), None, "fragment: {fragment}");
        }
    }
}
