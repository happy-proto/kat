use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Mapping {
    Identity { source_start: usize },
    Decoded { source: Range<usize> },
    Synthetic,
}

#[derive(Clone, Debug)]
struct Segment {
    virtual_range: Range<usize>,
    mapping: Mapping,
}

/// A projection maps runs, not UTF-8 bytes. Synthetic runs have no source position.
#[derive(Clone, Debug, Default)]
pub(crate) struct SourceMap {
    segments: Vec<Segment>,
    len: usize,
}

impl SourceMap {
    pub(crate) fn identity(&mut self, source: Range<usize>) {
        self.append(
            source.len(),
            Mapping::Identity {
                source_start: source.start,
            },
        );
    }

    pub(crate) fn decoded(&mut self, len: usize, source: Range<usize>) {
        if source.is_empty() {
            self.synthetic(len);
        } else {
            self.append(len, Mapping::Decoded { source });
        }
    }

    pub(crate) fn synthetic(&mut self, len: usize) {
        self.append(len, Mapping::Synthetic);
    }

    fn append(&mut self, len: usize, mapping: Mapping) {
        if len == 0 {
            return;
        }
        if let Some(last) = self.segments.last_mut() {
            let contiguous = match (&last.mapping, &mapping) {
                (
                    Mapping::Identity { source_start: left },
                    Mapping::Identity {
                        source_start: right,
                    },
                ) => *left + last.virtual_range.len() == *right,
                _ => last.mapping == mapping,
            };
            if contiguous {
                last.virtual_range.end += len;
                self.len += len;
                return;
            }
        }
        self.segments.push(Segment {
            virtual_range: self.len..self.len + len,
            mapping,
        });
        self.len += len;
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn get(&self, offset: usize) -> Option<Range<usize>> {
        let index = self
            .segments
            .partition_point(|segment| segment.virtual_range.end <= offset);
        let segment = self.segments.get(index)?;
        match &segment.mapping {
            Mapping::Identity { source_start } => {
                let start = source_start + offset - segment.virtual_range.start;
                Some(start..start + 1)
            }
            Mapping::Decoded { source } => Some(source.clone()),
            Mapping::Synthetic => None,
        }
    }

    pub(crate) fn ranges(&self, range: Range<usize>) -> Vec<Range<usize>> {
        if range.is_empty() {
            return Vec::new();
        }
        let index = self
            .segments
            .partition_point(|segment| segment.virtual_range.end <= range.start);
        let mut result = Vec::new();
        let mut active: Option<Range<usize>> = None;
        for segment in self.segments[index..]
            .iter()
            .take_while(|segment| segment.virtual_range.start < range.end)
        {
            let overlap = range.start.max(segment.virtual_range.start)
                ..range.end.min(segment.virtual_range.end);
            let mapped = match &segment.mapping {
                Mapping::Identity { source_start } => Some(
                    (source_start + overlap.start - segment.virtual_range.start)
                        ..(source_start + overlap.end - segment.virtual_range.start),
                ),
                Mapping::Decoded { source } => Some(source.clone()),
                Mapping::Synthetic => None,
            };
            match (active.as_mut(), mapped) {
                (Some(current), Some(next))
                    if current.start <= next.start && next.start <= current.end =>
                {
                    current.end = next.end
                }
                (_, next) => {
                    if let Some(previous) = active.take() {
                        result.push(previous);
                    }
                    active = next;
                }
            }
        }
        if let Some(previous) = active {
            result.push(previous);
        }
        result
    }

    pub(crate) fn anchor(&self, offset: usize) -> Option<usize> {
        self.get(offset).map(|range| range.start).or_else(|| {
            (offset == self.len && offset > 0)
                .then(|| self.get(offset - 1).map(|range| range.end))
                .flatten()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_ranges_match_dense_mapping_at_every_boundary() {
        let mut map = SourceMap::default();
        map.identity(10..14);
        map.decoded(3, 14..20); // One decoded Unicode scalar occupies three UTF-8 bytes.
        map.synthetic(2);
        map.identity(30..34);
        let dense = [
            Some(10..11),
            Some(11..12),
            Some(12..13),
            Some(13..14),
            Some(14..20),
            Some(14..20),
            Some(14..20),
            None,
            None,
            Some(30..31),
            Some(31..32),
            Some(32..33),
            Some(33..34),
        ];
        for start in 0..dense.len() {
            for end in start + 1..=dense.len() {
                // Compare byte coverage independently of the segmented representation.
                let mut expected = dense[start..end]
                    .iter()
                    .flatten()
                    .flat_map(|range| range.clone())
                    .collect::<Vec<_>>();
                expected.sort_unstable();
                expected.dedup();
                let ranges = map.ranges(start..end);
                assert!(ranges.windows(2).all(|pair| pair[0].end <= pair[1].start));
                let actual = ranges.into_iter().flatten().collect::<Vec<_>>();
                assert_eq!(actual, expected, "{start}..{end}");
            }
        }
        assert_eq!(map.anchor(7), None);
        assert_eq!(map.anchor(13), Some(34));
    }

    #[test]
    fn synthetic_positions_are_distinct_from_real_source_offset_zero() {
        let mut map = SourceMap::default();
        map.synthetic(128);
        map.identity(0..4);
        assert_eq!(map.get(0), None);
        assert_eq!(map.anchor(127), None);
        assert!(map.ranges(0..128).is_empty());
        assert_eq!(map.get(128), Some(0..1));
        assert_eq!(
            map.ranges(127..130),
            std::iter::once(0..2).collect::<Vec<_>>()
        );
        assert!(map.ranges(129..129).is_empty());
    }

    #[test]
    fn large_identity_projection_uses_one_segment() {
        let mut map = SourceMap::default();
        map.identity(0..1_000_000);
        map.identity(1_000_000..2_000_000);
        assert_eq!(map.segments.len(), 1);
        assert_eq!(
            map.ranges(123..1999999),
            std::iter::once(123..1999999).collect::<Vec<_>>()
        );
    }
}
