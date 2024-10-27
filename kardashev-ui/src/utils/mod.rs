pub mod any_cache;
pub mod futures;
pub mod small_linear_map;
pub mod thread_local_cell;
pub mod time;
pub mod web_fs;

pub fn human_size<T: humansize::ToF64 + humansize::Unsigned>(value: T) -> String {
    humansize::format_size(value, humansize::BINARY)
}
