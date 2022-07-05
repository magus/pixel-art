pub fn range(start: u32, end: u32) -> Box<dyn Iterator<Item = u32>> {
    let reverse = start > end;

    if reverse {
        return Box::new((end..start).rev());
    }

    return Box::new(start..end);
}
