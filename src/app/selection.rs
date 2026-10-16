#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Selection {
    start: f64,
    end: f64,
    media_duration: f64,
}

impl Selection {
    pub fn full(media_duration: f64) -> Self {
        let media_duration = media_duration.max(0.0);
        Self {
            start: 0.0,
            end: media_duration,
            media_duration,
        }
    }

    pub fn start(&self) -> f64 {
        self.start
    }

    pub fn end(&self) -> f64 {
        self.end
    }

    pub fn media_duration(&self) -> f64 {
        self.media_duration
    }

    pub fn length(&self) -> f64 {
        self.end - self.start
    }

    pub fn is_usable(&self) -> bool {
        self.length() > f64::EPSILON
    }

    pub fn set_start(&mut self, seconds: f64) {
        self.start = seconds.clamp(0.0, self.end);
    }

    pub fn set_end(&mut self, seconds: f64) {
        self.end = seconds.clamp(self.start, self.media_duration);
    }
}

#[cfg(test)]
mod tests {
    use super::Selection;

    #[test]
    fn starts_with_the_whole_media() {
        let selection = Selection::full(10.0);
        assert_eq!(selection.start(), 0.0);
        assert_eq!(selection.end(), 10.0);
        assert_eq!(selection.length(), 10.0);
    }

    #[test]
    fn start_never_passes_the_end() {
        let mut selection = Selection::full(10.0);
        selection.set_end(4.0);
        selection.set_start(9.0);
        assert_eq!(selection.start(), 4.0);
        assert_eq!(selection.end(), 4.0);
        assert!(!selection.is_usable());
    }

    #[test]
    fn end_never_moves_before_the_start() {
        let mut selection = Selection::full(10.0);
        selection.set_start(6.0);
        selection.set_end(2.0);
        assert_eq!(selection.start(), 6.0);
        assert_eq!(selection.end(), 6.0);
    }

    #[test]
    fn values_outside_the_media_are_clamped() {
        let mut selection = Selection::full(10.0);
        selection.set_start(-5.0);
        assert_eq!(selection.start(), 0.0);
        selection.set_end(50.0);
        assert_eq!(selection.end(), 10.0);
    }
}
