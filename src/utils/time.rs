use std::time::Instant;

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct Timer {
    last_time_min: Instant,
    last_time_max: Instant,
    min: usize,
    max: usize,
    last_max: usize,
    last_min: usize,
    len: u64,
}

impl Timer {
    pub fn new(length: u64) -> Self {
        Timer {
            last_time_min: Instant::now(),
            last_time_max: Instant::now(),
            min: usize::MAX,
            max: usize::MIN,
            last_min: usize::MAX,
            last_max: usize::MIN,
            len: length,
        }
    }

    pub fn update_value(&mut self, v: usize) -> f64 {
        let elapsed_min = self.last_time_min.elapsed();
        let elapsed_max = self.last_time_max.elapsed();

        if v > self.last_max {
            self.last_max = v;
        } else if v < self.last_min {
            self.last_min = v;
        }

        if v > self.max {
            self.last_time_max = Instant::now();
            self.max = v;
        }

        if v < self.min {
            self.last_time_min = Instant::now();
            self.min = v;
        }

        if elapsed_min.as_secs() >= self.len {
            self.min = self.last_min;
            self.last_min = usize::MAX;
            self.last_time_min = Instant::now();
        }

        if elapsed_max.as_secs() >= self.len {
            self.max = self.last_max;
            self.last_max = usize::MIN;
            self.last_time_max = Instant::now();
        }

        if self.min == self.max {
            return 0.0;
        }

        println!("{:} {:}", self.min, self.max);

        (v - self.min) as f64 * 100.0 / (self.max - self.min) as f64
    }
}
