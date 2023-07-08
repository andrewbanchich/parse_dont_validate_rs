fn main() {
    let array: [u8; 0] = [];
    let h = head(&array);

    dbg!(h);
}

// Won't compile!
// fn head<T>(slice: &[T]) -> &T {
//     match slice.get(0) {
//         Some(v) => v,
//     }
// }

// Will compile!
fn head<T>(slice: &[T]) -> Option<&T> {
    slice.get(0)
}
