use std::ops::Range;

use norm::{
    Metric,
    fzf::{FzfParser, FzfV2},
};

use super::Entry;

pub(super) struct Match {
    pub(super) index: usize,
    pub(super) ranges: Vec<Range<usize>>,
}

pub(super) struct FilenameFilter {
    pub(super) query: String,
    pub(super) editing: bool,
    pub(super) matches: Vec<Match>,
    matcher: FzfV2,
    parser: FzfParser,
}

impl FilenameFilter {
    pub(super) fn new(entries: &[Entry]) -> Self {
        let mut filter = Self {
            query: String::new(),
            editing: false,
            matches: Vec::new(),
            matcher: FzfV2::new(),
            parser: FzfParser::new(),
        };
        filter.rebuild(entries);
        filter
    }

    pub(super) fn rebuild(&mut self, entries: &[Entry]) {
        if self.query.is_empty() {
            self.matches = (0..entries.len())
                .map(|index| Match {
                    index,
                    ranges: Vec::new(),
                })
                .collect();
            return;
        }
        let query = self.parser.parse(&self.query);
        let mut ranked = entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let mut ranges = Vec::new();
                self.matcher
                    .distance_and_ranges(query, &entry.label(), &mut ranges)
                    .map(|distance| (distance, Match { index, ranges }))
            })
            .collect::<Vec<_>>();
        // Preserve the normal directory/name order for equally good matches.
        ranked.sort_by_key(|(distance, item)| (*distance, item.index));
        self.matches = ranked.into_iter().map(|(_, item)| item).collect();
    }
}
