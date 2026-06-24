pub struct FoldingEngine;

impl FoldingEngine {
    pub fn new() -> Self {
        FoldingEngine
    }

    pub fn folding_ranges(&self, text: &str) -> Vec<(u32, u32)> {
        let mut ranges = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut stack: Vec<(u32, u32)> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let open_count = line.chars().filter(|&c| c == '{').count();
            let close_count = line.chars().filter(|&c| c == '}').count();

            for _ in 0..open_count {
                stack.push((i as u32, 0));
            }

            for _ in 0..close_count.min(stack.len()) {
                if let Some((start, _)) = stack.pop() {
                    if i as u32 > start {
                        ranges.push((start, i as u32));
                    }
                }
            }
        }

        ranges
    }
}

impl Default for FoldingEngine {
    fn default() -> Self {
        Self::new()
    }
}
