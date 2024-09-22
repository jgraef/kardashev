use std::{
    io::SeekFrom,
    path::Path,
    time::Instant,
};

use futures::{
    pin_mut,
    Future,
    FutureExt,
};
use image::{
    Rgb,
    RgbImage,
};
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use nalgebra::{
    Point3,
    Rotation3,
    Vector3,
};
use palette::LinSrgb;
use tokio::{
    fs::File,
    io::{
        AsyncRead,
        AsyncReadExt,
        AsyncSeekExt,
        AsyncWrite,
        AsyncWriteExt,
        BufReader,
        BufWriter,
    },
};

use crate::{
    gaia::{
        self,
        Data,
        HealPixRange,
    },
    utils::teff_color::TEFF_COLORS,
    Error,
};

struct Record {
    source_id: u64,
    healpix_range: HealPixRange,
    parallax: f64,
    longitude: f64,
    latitude: f64,
    t_eff: f32,
    apparent_magnitude: f32,
}

impl Record {
    pub fn from_gaia(record: &gaia::Record) -> Option<Self> {
        Some(Self {
            source_id: record.gaia_source.source_id,
            healpix_range: record.healpix_range,
            parallax: record.gaia_source.parallax?,
            longitude: record.gaia_source.l?,
            latitude: record.gaia_source.b?,
            t_eff: record.gaia_source.teff_gspphot?,
            apparent_magnitude: record.gaia_source.phot_g_mean_mag?,
        })
    }

    pub async fn write(&self, mut writer: impl AsyncWrite + Unpin) -> Result<(), Error> {
        writer.write_u64(self.source_id).await?;
        writer.write_u32(self.healpix_range.start).await?;
        writer.write_u32(self.healpix_range.end).await?;
        writer.write_f64(self.parallax).await?;
        writer.write_f64(self.longitude).await?;
        writer.write_f64(self.latitude).await?;
        writer.write_f32(self.t_eff).await?;
        writer.write_f32(self.apparent_magnitude).await?;
        Ok(())
    }

    pub async fn read(mut reader: impl AsyncRead + Unpin) -> Result<Self, Error> {
        let source_id = reader.read_u64().await?;
        let healpix_start = reader.read_u32().await?;
        let healpix_end = reader.read_u32().await?;
        let parallax = reader.read_f64().await?;
        let longitude = reader.read_f64().await?;
        let latitude = reader.read_f64().await?;
        let t_eff = reader.read_f32().await?;
        let apparent_magnitude = reader.read_f32().await?;
        Ok(Self {
            source_id,
            healpix_range: HealPixRange {
                start: healpix_start,
                end: healpix_end,
            },
            parallax,
            longitude,
            latitude,
            t_eff,
            apparent_magnitude,
        })
    }

    pub fn color(&self) -> LinSrgb {
        TEFF_COLORS
            .get(self.t_eff)
            .unwrap_or_else(|| LinSrgb::new(1.0, 1.0, 1.0))
    }

    /// in kilo parsec
    pub fn distance(&self) -> f64 {
        1.0 / self.parallax
    }

    pub fn absolute_magnitude(&self) -> f32 {
        self.apparent_magnitude - 5.0 * (self.distance().log10() as f32 + 2.0)
    }

    pub fn position(&self) -> Point3<f64> {
        let longitude = self.longitude.to_radians();
        let latitude = self.latitude.to_radians();

        let rotation = Rotation3::from_axis_angle(&Vector3::z_axis(), longitude)
            * Rotation3::from_axis_angle(&Vector3::x_axis(), latitude);

        Point3::from(rotation * (self.distance() * *Vector3::y_axis()))
    }
}

fn brightness(magnitude: f32, reference: f32) -> f32 {
    const BRIGHTNESS_FACTOR: f32 = 0.398107171; // 100.0f32.powf(-0.2f32);
    BRIGHTNESS_FACTOR
        .powf(magnitude - reference)
        .clamp(0.1, 1.0)
}

struct RecordReader {
    reader: BufReader<File>,
    num_records: u64,
    num_read: u64,
}

impl RecordReader {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = File::open(path).await?;
        let mut reader = BufReader::new(file);

        let num_records = reader.read_u64().await?;

        Ok(Self {
            reader,
            num_records,
            num_read: 0,
        })
    }

    pub async fn read_record(&mut self) -> Result<Option<Record>, Error> {
        if self.num_read >= self.num_records {
            return Ok(None);
        }

        let record = Record::read(&mut self.reader).await?;
        self.num_read += 1;

        Ok(Some(record))
    }

    pub fn num_records(&self) -> u64 {
        self.num_records
    }

    pub fn num_read(&self) -> u64 {
        self.num_read
    }
}

pub async fn export(
    output: impl AsRef<Path>,
    path: impl AsRef<Path>,
    limit_per_file: u64,
) -> Result<(), Error> {
    let data = Data::open(path).await?;
    let mut records = data.sequential();

    let (_, num_partitions) = records.progress();
    let progress_bar = ProgressBar::new(num_partitions as _);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{pos}/{len}] {spinner:.green} {wide_bar:.cyan/blue} ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    let output_file = File::create(output).await?;
    let mut output_writer = BufWriter::new(output_file);

    output_writer.write_u64(0).await?;
    let mut count = 0;
    let mut count_per_file = 0;

    while let Some(record) = records.read_record().await? {
        if let Some(record) = Record::from_gaia(&record) {
            record.write(&mut output_writer).await?;

            count += 1;
            count_per_file += 1;

            if count_per_file >= limit_per_file {
                count_per_file = 0;
                records.skip_file();
            }
        }

        let (progress, _) = records.progress();
        progress_bar.set_position(progress as _);
    }

    output_writer.seek(SeekFrom::Start(0)).await?;
    output_writer.write_u64(count).await?;

    output_writer.flush().await?;

    Ok(())
}

struct Canvas {
    image: RgbImage,
}

impl Canvas {
    fn new(width: u32, height: u32) -> Self {
        let image = RgbImage::from_pixel(width, height, Rgb([0; 3]));
        Self { image }
    }

    fn draw_particle_topdown(&mut self, record: &Record, radius: f64) {
        if record.parallax < 0.0 {
            return;
        }

        let image_size = std::cmp::min(self.image.width(), self.image.height()) as i32;
        let scale = 0.5 * (image_size as f64) / radius;

        let position = record.position();
        let x = (position.x * scale) as i32 + image_size / 2;
        let y = (position.y * scale) as i32 + image_size / 2;
        if x < 0 || x >= image_size || y < 0 || y >= image_size {
            return;
        }
        let x = x as u32;
        let y = y as u32;

        let color = record.color();
        //let brightness = brightness(record.absolute_magnitude(), 5.0);
        let brightness = 1.0;
        let pixel = Rgb([
            (color.red * brightness * 255.0) as u8,
            (color.green * brightness * 255.0) as u8,
            (color.blue * brightness * 255.0) as u8,
        ]);

        self.image.put_pixel(x, y, pixel);
    }

    fn draw_particle_skyview(&mut self, record: &Record) {
        use std::f64::consts::{
            FRAC_PI_2,
            PI,
            TAU,
        };

        let scale_x = (self.image.width() as f64) / TAU;
        let scale_y = (self.image.height() as f64) / PI;

        let x = ((PI - record.longitude.to_radians() + TAU) % TAU * scale_x) as u32;
        let y = ((FRAC_PI_2 - record.latitude.to_radians()) % PI * scale_y) as u32;

        let color = record.color();
        //let brightness = brightness(record.apparent_magnitude, 14.0);
        let brightness = 1.0;
        let pixel = Rgb([
            (color.red * brightness * 255.0) as u8,
            (color.green * brightness * 255.0) as u8,
            (color.blue * brightness * 255.0) as u8,
        ]);

        self.image.put_pixel(x, y, pixel);
    }
}

#[derive(Clone, Copy, Debug, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum View {
    TopDown,
    Sky,
}

impl View {
    const TOP_DOWN_RADIUS: f64 = 60.0; // in kilo parsec

    fn image_size(&self, width: u32) -> [u32; 2] {
        match self {
            Self::TopDown => [width, width],
            Self::Sky => [width, width / 2],
        }
    }

    fn draw_particle(&self, canvas: &mut Canvas, record: &Record) {
        match self {
            Self::TopDown => canvas.draw_particle_topdown(record, Self::TOP_DOWN_RADIUS),
            Self::Sky => canvas.draw_particle_skyview(record),
        }
    }
}

pub async fn render(
    output: impl AsRef<Path>,
    path: impl AsRef<Path>,
    view: View,
    width: u32,
) -> Result<(), Error> {
    async fn next_record(
        records: &mut RecordReader,
        abort_signal: impl Future<Output = ()> + Unpin,
    ) -> Result<Option<Record>, Error> {
        tokio::select! {
            _ = abort_signal => Ok(None),
            result = records.read_record() => result
        }
    }

    let mut records = RecordReader::open(path).await?;

    let progress_bar = ProgressBar::new(records.num_records());
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{pos}/{len}] {spinner:.green} {wide_bar:.cyan/blue} ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    let image_size = view.image_size(width);
    let mut canvas = Canvas::new(image_size[0], image_size[1]);

    let ctrl_c = tokio::signal::ctrl_c().map(|_| ());
    pin_mut!(ctrl_c);

    let t_start = Instant::now();

    while let Some(record) = next_record(&mut records, &mut ctrl_c).await? {
        view.draw_particle(&mut canvas, &record);
        progress_bar.set_position(records.num_read());
    }

    let time = t_start.elapsed();
    tracing::info!("rendering took {} s", time.as_secs());

    let output = output.as_ref();
    tracing::info!("writing image: {}", output.display());
    canvas.image.save(output)?;

    Ok(())
}
