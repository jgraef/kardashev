use palette::Srgba;

pub fn wgpu_buffer_size<T>() -> u64 {
    let unpadded_size: u64 = std::mem::size_of::<T>()
        .try_into()
        .expect("failed to convert usize to u64");
    let align_mask = wgpu::COPY_BUFFER_ALIGNMENT - 1;
    let padded_size = ((unpadded_size + align_mask) & !align_mask).max(wgpu::COPY_BUFFER_ALIGNMENT);
    padded_size
}

pub fn color_to_wgpu(color: Srgba<f64>) -> wgpu::Color {
    wgpu::Color {
        r: color.red,
        g: color.green,
        b: color.blue,
        a: color.alpha,
    }
}

pub fn color_to_array<T: Copy>(color: Srgba<T>) -> [T; 4] {
    [color.red,
    color.green,
    color.blue,
    color.alpha]
}