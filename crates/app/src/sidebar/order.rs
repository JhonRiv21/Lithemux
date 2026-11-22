//! Moving one item of an ordered list: what a drag or «move up» asks for.
//! The result is the whole new order, which the use cases of `core` apply.

/// `ids` with `id` at `to`, the rest keeping their order. Unchanged when
/// `id` is not there.
pub fn moved(ids: &[String], id: &str, to: usize) -> Vec<String> {
    let mut order: Vec<String> = ids.iter().filter(|other| *other != id).cloned().collect();
    if order.len() == ids.len() {
        return order;
    }
    order.insert(to.min(order.len()), id.to_string());
    order
}

/// `ids` with `id` one place earlier (`-1`) or later (`1`); it stays at
/// the end it is already at.
pub fn shifted(ids: &[String], id: &str, by: isize) -> Vec<String> {
    let Some(from) = ids.iter().position(|other| other == id) else {
        return ids.to_vec();
    };
    let last = ids.len() as isize - 1;
    moved(ids, id, (from as isize + by).clamp(0, last) as usize)
}

/// `ids` with `id` taking the place of `target`: what dropping one card on
/// another means.
pub fn dropped_on(ids: &[String], id: &str, target: &str) -> Vec<String> {
    match ids.iter().position(|other| other == target) {
        Some(place) => moved(ids, id, place),
        None => ids.to_vec(),
    }
}

/// `all` with the items named in `shown` taking, among themselves, the
/// order `shown` gives them. The others stay in their places.
pub fn with_shown_reordered(all: &[String], shown: &[String]) -> Vec<String> {
    let mut next = shown.iter();
    all.iter()
        .map(|id| {
            if shown.contains(id) {
                next.next().unwrap_or(id).clone()
            } else {
                id.clone()
            }
        })
        .collect()
}
