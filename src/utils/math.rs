pub fn median<T>(numbers: &mut [T]) -> Option<T>
where
    T: PartialOrd + Clone,
{
    if numbers.is_empty() {
        return None;
    }

    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let len = numbers.len();
    if len % 2 == 0 {
        let mid = len / 2;
        let mid1 = &numbers[mid - 1];
        return Some(mid1.clone());
    } else {
        let mid = len / 2;
        return Some(numbers[mid].clone());
    }
}
