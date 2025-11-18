/// Calculate reading time in minutes from word count
/// Always rounds up to the nearest minute
pub fn estimate_reading_time(word_count: usize, words_per_minute: usize) -> usize {
    if word_count == 0 {
        return 0;
    }

    // Round up using div_ceil
    word_count.div_ceil(words_per_minute)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_words() {
        assert_eq!(estimate_reading_time(0, 200), 0);
    }

    #[test]
    fn test_rounds_up() {
        assert_eq!(estimate_reading_time(1, 200), 1);
        assert_eq!(estimate_reading_time(199, 200), 1);
        assert_eq!(estimate_reading_time(200, 200), 1);
        assert_eq!(estimate_reading_time(201, 200), 2);
        assert_eq!(estimate_reading_time(400, 200), 2);
        assert_eq!(estimate_reading_time(401, 200), 3);
    }

    #[test]
    fn test_different_wpm() {
        assert_eq!(estimate_reading_time(250, 250), 1);
        assert_eq!(estimate_reading_time(251, 250), 2);
        assert_eq!(estimate_reading_time(150, 150), 1);
        assert_eq!(estimate_reading_time(300, 150), 2);
    }
}
