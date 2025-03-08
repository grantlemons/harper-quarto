use harper_core::{Mask, Masker, Span};
use itertools::Itertools;

pub struct QuartoMasker;

fn isolate_document(source: &[char]) -> Option<Vec<Span>> {
    struct SourceLine {
        dashes: bool,
        span: Span,
    }

    let lines = source
        .into_iter()
        .enumerate()
        .collect_vec()
        .split(|(_, c)| **c == '\n')
        // removes empty lines as well
        .filter_map(|l| {
            Some(SourceLine {
                dashes: l.iter().map(|(_, c)| *c).all(|&c| c == '-'),
                span: Span::new(l.first()?.0, l.last()?.0 + 1),
            })
        })
        .collect_vec();

    let first_line_dashes = lines.first()?.dashes;
    let res = if first_line_dashes {
        lines.splitn(3, |l| l.dashes).nth(2)?.iter()
    } else {
        lines.iter()
    };

    Some(res.map(|l| l.span).collect())
}

impl Masker for QuartoMasker {
    fn create_mask(&self, source: &[char]) -> harper_core::Mask {
        let mut mask = Mask::new_blank();

        if let Some(lines) = isolate_document(source) {
            lines.into_iter().for_each(|span| {
                mask.push_allowed(span);
            });
        }

        mask.merge_whitespace_sep(source);
        mask
    }
}

#[cfg(test)]
mod tests {
    use harper_core::Masker;
    use itertools::Itertools;

    use super::QuartoMasker;

    #[test]
    fn mask_no_header() {
        let src = "# Line
---
line
# Line
---
line
line
";
        let expected = src.trim_end().chars().collect_vec();
        let src = src.chars().collect_vec();

        let masker = QuartoMasker;
        let mask = masker.create_mask(&src);

        let allowed = mask.iter_allowed(&src).next().expect("No allowed blocks").1;
        assert_eq!(allowed, expected);
    }

    #[test]
    fn mask_newline_before_header() {
        let src = "
---
Header contents
---
line
line
";
        let expected = src[25..].trim_end().chars().collect_vec();
        let src = src.chars().collect_vec();

        let masker = QuartoMasker;
        let mask = masker.create_mask(&src);

        let allowed = mask.iter_allowed(&src).next().expect("No allowed blocks").1;
        assert_eq!(allowed, expected);
    }

    #[test]
    fn mask_header() {
        let src = "---
Header contents
---
line
line
";
        let expected = src[24..].trim_end().chars().collect_vec();
        let src = src.chars().collect_vec();

        let masker = QuartoMasker;
        let mask = masker.create_mask(&src);

        let allowed = mask.iter_allowed(&src).next().expect("No allowed blocks").1;
        assert_eq!(allowed, expected);
    }

    #[test]
    fn mask_false_header() {
        let src = "This document contains text!
---
False header contents
---
line
line
";
        let expected = src.trim_end().chars().collect_vec();
        let src = src.chars().collect_vec();

        let masker = QuartoMasker;
        let mask = masker.create_mask(&src);

        let allowed = mask.iter_allowed(&src).next().expect("No allowed blocks").1;
        assert_eq!(allowed, expected);
    }
}
