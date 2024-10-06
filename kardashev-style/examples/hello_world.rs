use kardashev_style::style;

#[style(path = "examples/hello_world.scss")]
struct StyleWorld;

#[style(path = "examples/hello_other.scss")]
struct StyleOther;

pub fn main() {
    println!("{}", StyleWorld::myclass);
    println!("{}", StyleWorld::my_other_class);
    println!("{}", StyleOther::this_is_renamed);
}
