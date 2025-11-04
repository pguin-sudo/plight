use unit_interval::UnitInterval;

use super::led_color::LedColor;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LedSequence {
    led_colors: Vec<LedColor>,
}

impl LedSequence {
    pub fn new(len: usize) -> Self {
        LedSequence {
            led_colors: vec![LedColor::default(); len],
        }
    }

    // Set

    pub fn set_color(&mut self, color: LedColor) {
        self.led_colors = vec![color; self.led_colors.len()];
    }

    pub fn set_colors(&mut self, led_colors: &[LedColor]) {
        self.led_colors = led_colors.to_vec();
    }

    pub fn set_sequence(&mut self, led_sequence: LedSequence) {
        self.led_colors = led_sequence.led_colors;
    }

    // Change

    pub fn adjusted_halfs_value(mut self, levels: (UnitInterval<f64>, UnitInterval<f64>)) -> Self {
        let mid = self.led_colors.len() / 2;
        let (first, second) = self.led_colors.split_at_mut(mid);

        first.iter_mut().for_each(|c| *c = *c * levels.1);
        second.iter_mut().for_each(|c| *c = *c * levels.0);

        self
    }

    // Default

    pub fn len(&self) -> usize {
        self.led_colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.led_colors.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&LedColor> {
        self.led_colors.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut LedColor> {
        self.led_colors.get_mut(index)
    }
}

impl IntoIterator for LedSequence {
    type Item = LedColor;
    type IntoIter = std::vec::IntoIter<LedColor>;

    fn into_iter(self) -> Self::IntoIter {
        self.led_colors.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut LedSequence {
    type Item = &'a mut LedColor;
    type IntoIter = std::slice::IterMut<'a, LedColor>;

    fn into_iter(self) -> Self::IntoIter {
        self.led_colors.iter_mut()
    }
}

impl<'a> IntoIterator for &'a LedSequence {
    type Item = &'a LedColor;
    type IntoIter = std::slice::Iter<'a, LedColor>;

    fn into_iter(self) -> Self::IntoIter {
        self.led_colors.iter()
    }
}

impl FromIterator<LedColor> for LedSequence {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = LedColor>,
    {
        LedSequence {
            led_colors: (Vec::from_iter(iter)),
        }
    }
}
