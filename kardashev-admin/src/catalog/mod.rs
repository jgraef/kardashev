pub mod bright_stars;
pub mod gaia;
pub mod gliese;
pub mod hyg;

#[allow(dead_code)]
fn collapse_whitespace(s: &str) -> String {
    let mut out = String::new();

    let mut chars = s.chars();
    'outer: while let Some(mut c) = chars.next() {
        out.push(c);
        while c.is_whitespace() {
            if let Some(c2) = chars.next() {
                c = c2;
            }
            else {
                break 'outer;
            }
        }
    }

    out
}
