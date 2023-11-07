use std::{collections::BTreeMap, fmt::Debug, fs::File, io::Read, path::Path};

/// An histogram of bytes.
type Histogram = BTreeMap<u8, usize>;

// An histogram of dwords.
type DiHistogram = BTreeMap<(u8, u8), usize>;

/// Calculate the histogram of bytes of a given file.
pub fn calculate_histogram<P>(file: P) -> Histogram
where
    P: AsRef<Path> + Debug,
{
    let mut histogram = BTreeMap::new();
    let mut handle = File::open(&file).expect(&format!("Couldn't open file: {:?}", file));
    let mut buf = Vec::new();
    handle
        .read_to_end(&mut buf)
        .expect(&format!("Couldn't `read_to_end` on: {:?}", handle));
    for byte in buf {
        histogram.entry(byte).and_modify(|x| *x += 1).or_insert(1);
    }
    histogram
}

/// Calculate the dihistogram of dwords of a given file.
pub fn calculate_dihistogram<P>(file: P) -> DiHistogram
where
    P: AsRef<Path> + Debug,
{
    let mut dihistogram = BTreeMap::new();
    let mut handle = File::open(&file).expect(&format!("Couldn't open file: {:?}", file));
    let mut buf = Vec::new();
    handle
        .read_to_end(&mut buf)
        .expect(&format!("Couldn't `read_to_end` on: {:?}", handle));
    for dword in buf.windows(2) {
        dihistogram
            .entry((dword[0], dword[1]))
            .and_modify(|x| *x += 1)
            .or_insert(1);
    }
    dihistogram
}

/// Calculate the entropy from a given histogram.
pub fn calculate_entropy(histogram: &Histogram) -> f64 {
    let mut entropy = 0.0;
    let total: usize = histogram.values().sum();
    for (_byte, freq) in histogram {
        let relative_freq = (*freq as f64) / (total as f64);
        entropy += relative_freq.log2() * relative_freq;
    }
    -entropy
}

pub fn calculate_causal_entropy(histogram: &DiHistogram) -> f64 {
    let mut entropy = 0.0;
    let total: usize = histogram.values().sum();
    for (_byte, freq) in histogram {
        let relative_freq = (*freq as f64) / (total as f64);
        entropy += relative_freq.log2() * relative_freq;
    }
    -entropy
}

/// Get the top `count` most frequent byte(s) from the given histogram.
pub fn get_most_frequent_bytes(histogram: &Histogram, count: Option<u8>) -> Vec<(&u8, &usize)> {
    let mut vector: Vec<(&u8, &usize)> = histogram.into_iter().collect();
    vector.sort_by(|x, y| x.1.cmp(y.1).reverse());
    if let Some(count) = count {
        vector.into_iter().take(count.into()).collect()
    } else {
        vector.into_iter().take(255).collect()
    }
}
