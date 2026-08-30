use k2f_core::GlyphPosition;

pub fn unicode_for(cluster: u32, next: Option<u32>, chars: &[char]) -> String {
    if cluster == GlyphPosition::CLUSTER_NOT_SOURCE {
        return String::new();
    }
    let start = cluster as usize;
    if start >= chars.len() {
        return String::new();
    }
    let end = match next {
        Some(n) if n != GlyphPosition::CLUSTER_NOT_SOURCE => (n as usize).min(chars.len()),
        _ => chars.len(),
    };
    if end <= start {
        return String::new();
    }
    chars[start..end].iter().collect()
}

pub fn next_source_cluster(clusters: &[u32], index: usize) -> Option<u32> {
    let cur = *clusters.get(index)?;
    clusters[index + 1..]
        .iter()
        .copied()
        .find(|&c| c != GlyphPosition::CLUSTER_NOT_SOURCE && c > cur)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ligature_takes_until_next_cluster() {
        let chars: Vec<char> = "file".chars().collect();
        assert_eq!(unicode_for(0, Some(2), &chars), "fi");
        assert_eq!(unicode_for(2, Some(3), &chars), "l");
        assert_eq!(unicode_for(3, None, &chars), "e");
    }

    #[test]
    fn skips_decorative_clusters() {
        let chars: Vec<char> = "Hi".chars().collect();
        assert_eq!(
            unicode_for(GlyphPosition::CLUSTER_NOT_SOURCE, None, &chars),
            ""
        );
    }
}
